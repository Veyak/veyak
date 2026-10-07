/// QuickJS-based scripting engine for pre-request and post-request hooks.
///
/// The scripting API is intentionally Postman-compatible:
///
/// ```javascript
/// // Pre-request example
/// pm.environment.set("token", pm.environment.get("cached_token") || "");
///
/// // Post-request example
/// const json = pm.response.json();
/// pm.environment.set("access_token", json.token);
/// pm.test("Status 200", () => pm.response.to.have.status(200));
/// ```
///
/// Globals injected into every script:
/// - `pm.environment.get(key)` / `pm.environment.set(key, value)`
/// - `pm.request`      – the outgoing request object
/// - `pm.response`     – the received response (only in post-request)
/// - `pm.test(name, fn)` – register a test assertion
/// - `console.log / warn / error` – output captured and returned in the result
use std::collections::HashMap;

use rquickjs::{Context, Function, Object, Runtime};
use veyak_error::{AppError, AppResult};
use veyak_models::{ApiRequest, ApiResponse};

#[derive(Debug, Default)]
pub struct ScriptResult {
    /// Variables that the script wrote via `pm.environment.set(k, v)`.
    pub env_updates: HashMap<String, String>,
    /// Console output lines (log / warn / error).
    pub console_output: Vec<String>,
    /// Test results from `pm.test(name, fn)`.
    pub test_results: Vec<TestResult>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub error: Option<String>,
}

/// Run the pre-request script attached to a request.
pub fn run_pre_request_script(
    request: ApiRequest,
    env_vars: &HashMap<String, String>,
) -> AppResult<(ApiRequest, ScriptResult)> {
    let script = match &request.pre_request_script {
        Some(s) if !s.trim().is_empty() => s.clone(),
        _ => return Ok((request, ScriptResult::default())),
    };

    let result = execute_script(&script, env_vars, Some(&request), None)?;
    Ok((request, result))
}

/// Run the post-request script attached to a request.
pub fn run_post_request_script(
    request: &ApiRequest,
    response: &ApiResponse,
    env_vars: &HashMap<String, String>,
) -> AppResult<ScriptResult> {
    let script = match &request.post_request_script {
        Some(s) if !s.trim().is_empty() => s.clone(),
        _ => return Ok(ScriptResult::default()),
    };

    execute_script(&script, env_vars, Some(request), Some(response))
}

/// The static pm bootstrap (no Rust format! interpolation — written as raw string
/// so the JS regex literal stays intact without conflicting with format! braces).
const PM_BOOTSTRAP_STATIC: &str = r#"
var pm = (function() {
    var environment = {
        _data: __envData__,
        get: function(key) { return __envData__[key] !== undefined ? String(__envData__[key]) : ""; },
        set: function(key, value) {
            __envData__[key] = value;
            __pm_set_env__(String(key), String(value));
        }
    };

    var variables = {
        replaceIn: function(template) {
            return String(template).replace(/\{\{(\w+)\}\}/g, function(_, k) {
                return __envData__[k] !== undefined ? String(__envData__[k]) : "";
            });
        }
    };

    var response = null;
    if (__responseData__ !== null) {
        response = Object.assign({}, __responseData__);
        response.json = function() {
            try { return JSON.parse(__responseData__.body || "{}"); } catch(e) { return {}; }
        };
        response.text = function() { return __responseData__.body || ""; };
        response.code = __responseData__.status;
        response.to = {
            have: {
                status: function(expected) {
                    if (__responseData__.status !== expected) {
                        throw new Error("Expected status " + expected + " but got " + __responseData__.status);
                    }
                }
            }
        };
    }

    return {
        environment: environment,
        variables: variables,
        request: __requestData__,
        response: response,
        test: function(name, fn) {
            __pm_test__(String(name), fn);
        }
    };
})();
"#;

/// Core script execution engine using QuickJS.
fn execute_script(
    script: &str,
    env_vars: &HashMap<String, String>,
    request: Option<&ApiRequest>,
    response: Option<&ApiResponse>,
) -> AppResult<ScriptResult> {
    // Serialise inputs to JSON for injection
    let env_json = serde_json::to_string(env_vars)
        .map_err(|e| AppError::Script(format!("serialise env: {e}")))?;
    let request_json = match request {
        Some(r) => serde_json::to_string(r)
            .map_err(|e| AppError::Script(format!("serialise request: {e}")))?,
        None => "null".to_string(),
    };
    let response_json = match response {
        Some(r) => serde_json::to_string(r)
            .map_err(|e| AppError::Script(format!("serialise response: {e}")))?,
        None => "null".to_string(),
    };

    // Build the data-injection snippet (only the variable-interpolated part)
    let data_injection = format!(
        "var __envData__ = {env_json};\
         var __requestData__ = {request_json};\
         var __responseData__ = {response_json};\n"
    );

    let rt = Runtime::new().map_err(|e| AppError::Script(format!("QuickJS runtime: {e}")))?;
    let ctx = Context::full(&rt).map_err(|e| AppError::Script(format!("QuickJS context: {e}")))?;

    // Shared mutable state accessed via Arc<Mutex<>>
    let env_updates = std::sync::Arc::new(std::sync::Mutex::new(HashMap::<String, String>::new()));
    let console_lines = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let test_results = std::sync::Arc::new(std::sync::Mutex::new(Vec::<TestResult>::new()));

    let env_updates_c = env_updates.clone();
    let console_c = console_lines.clone();
    let test_results_c = test_results.clone();

    ctx.with(|ctx| -> rquickjs::Result<()> {
        // ── console ────────────────────────────────────────────────────────
        let console_obj = Object::new(ctx.clone())?;
        for level in &["log", "warn", "error", "info"] {
            let cl = console_c.clone();
            let lvl = level.to_string();
            let f = Function::new(ctx.clone(), {
                let cl = cl.clone();
                let lvl = lvl.clone();
                move |args: rquickjs::function::Rest<rquickjs::Value>| {
                    let parts: Vec<String> = args
                        .0
                        .iter()
                        .map(|v| {
                            if let Some(s) = v.as_string() {
                                s.to_string().unwrap_or_default()
                            } else if let Some(n) = v.as_int() {
                                n.to_string()
                            } else if let Some(f) = v.as_float() {
                                f.to_string()
                            } else if let Some(b) = v.as_bool() {
                                b.to_string()
                            } else {
                                "[object]".to_string()
                            }
                        })
                        .collect();
                    let line = format!("[{}] {}", lvl, parts.join(" "));
                    if let Ok(mut g) = cl.lock() {
                        g.push(line);
                    }
                }
            })?;
            console_obj.set(*level, f)?;
        }
        ctx.globals().set("console", console_obj)?;

        // ── pm.environment.set (bridge to Rust) ────────────────────────────
        let eu = env_updates_c.clone();
        let set_fn = Function::new(ctx.clone(), move |key: String, value: String| {
            if let Ok(mut g) = eu.lock() {
                g.insert(key, value);
            }
        })?;

        // ── pm.test (bridge to Rust) ──────────────────────────────────────
        let tr = test_results_c.clone();
        let test_fn = Function::new(
            ctx.clone(),
            move |name: String, func: rquickjs::Function| -> rquickjs::Result<()> {
                let result = func.call::<(), ()>(());
                let (passed, error) = match result {
                    Ok(_) => (true, None),
                    Err(e) => (false, Some(e.to_string())),
                };
                if let Ok(mut g) = tr.lock() {
                    g.push(TestResult { name, passed, error });
                }
                Ok(())
            },
        )?;

        ctx.globals().set("__pm_set_env__", set_fn)?;
        ctx.globals().set("__pm_test__", test_fn)?;

        // Step 1: inject data variables
        ctx.eval::<(), _>(data_injection.as_str())?;
        // Step 2: build the pm global (static template with JS regex, no Rust format!)
        ctx.eval::<(), _>(PM_BOOTSTRAP_STATIC)?;
        // Step 3: execute user script
        ctx.eval::<(), _>(script.to_string())?;

        Ok(())
    })
    .map_err(|e| AppError::Script(format!("Script execution error: {e}")))?;

    let env_result = env_updates.lock().unwrap().clone();
    let console_result = console_lines.lock().unwrap().clone();
    let test_result: Vec<TestResult> = test_results.lock().unwrap().clone();

    Ok(ScriptResult {
        env_updates: env_result,
        console_output: console_result,
        test_results: test_result,
    })
}
