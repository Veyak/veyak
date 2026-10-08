/// Multi-language scripting engine for pre-request and post-request hooks.
///
/// Supported languages:
/// - JavaScript (embedded QuickJS via `rquickjs`)
/// - Python (host `python3` runner)
/// - Go (host `go run` runner)
/// - Rust (host runner with `cargo` / `rustc`)
///
/// The scripting API exposes the `veyak` (and `vy` / `pm` aliases) namespace:
///
/// ```javascript
/// // JavaScript (QuickJS) example
/// const data = veyak.response.json();
/// veyak.environment.set("access_token", data.token);
/// veyak.test("Status 200", () => veyak.response.to.have.status(200));
/// ```
///
/// ```python
/// # Python example
/// data = veyak.response.json()
/// veyak.environment.set("access_token", data.get("token"))
/// veyak.test("Status 200", lambda: veyak.response.status == 200)
/// ```
use std::collections::HashMap;
use std::process::{Command, Stdio};

use rquickjs::{Context, Function, Object, Runtime};
use veyak_error::{AppError, AppResult};
use veyak_models::{ApiRequest, ApiResponse, ScriptLanguage, ScriptTestResult};

#[derive(Debug, Default, Clone)]
pub struct ScriptResult {
    /// Variables that the script wrote via `veyak.environment.set(k, v)`.
    pub env_updates: HashMap<String, String>,
    /// Console output lines (log / warn / error / print).
    pub console_output: Vec<String>,
    /// Test results from `veyak.test(name, fn)`.
    pub test_results: Vec<ScriptTestResult>,
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

    let lang = request
        .pre_request_language
        .unwrap_or(ScriptLanguage::Javascript);
    let result = execute_by_language(lang, &script, env_vars, Some(&request), None)?;
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

    let lang = request
        .post_request_language
        .unwrap_or(ScriptLanguage::Javascript);
    execute_by_language(lang, &script, env_vars, Some(request), Some(response))
}

/// Dispatcher to execute scripts according to the selected language.
fn execute_by_language(
    language: ScriptLanguage,
    script: &str,
    env_vars: &HashMap<String, String>,
    request: Option<&ApiRequest>,
    response: Option<&ApiResponse>,
) -> AppResult<ScriptResult> {
    match language {
        ScriptLanguage::Javascript => execute_quickjs(script, env_vars, request, response),
        ScriptLanguage::Python => execute_python(script, env_vars, request, response),
        ScriptLanguage::Go => execute_golang(script, env_vars, request, response),
        ScriptLanguage::Rust => execute_rust(script, env_vars, request, response),
    }
}

// ===========================================================================
// JavaScript Engine (QuickJS)
// ===========================================================================

const VEYAK_BOOTSTRAP_JS: &str = r#"
var veyak = (function() {
    var environment = {
        _data: __envData__,
        get: function(key) { return __envData__[key] !== undefined ? String(__envData__[key]) : ""; },
        set: function(key, value) {
            __envData__[key] = value;
            __veyak_set_env__(String(key), String(value));
        },
        has: function(key) { return __envData__[key] !== undefined; }
    };

    var variables = {
        get: function(key) { return environment.get(key); },
        set: function(key, value) { environment.set(key, value); },
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
        response.status = __responseData__.status;
        response.headers = __responseData__.headers || {};
        response.to = {
            have: {
                status: function(expected) {
                    if (__responseData__.status !== expected) {
                        throw new Error("Expected status " + expected + " but got " + __responseData__.status);
                    }
                },
                header: function(name) {
                    var lower = String(name).toLowerCase();
                    var found = Object.keys(response.headers).some(function(k) { return k.toLowerCase() === lower; });
                    if (!found) {
                        throw new Error("Expected header " + name + " to be present");
                    }
                }
            }
        };
    }

    function expect(actual) {
        return {
            to: {
                eql: function(expected) {
                    if (JSON.stringify(actual) !== JSON.stringify(expected)) {
                        throw new Error("Expected " + JSON.stringify(actual) + " to equal " + JSON.stringify(expected));
                    }
                },
                equal: function(expected) {
                    if (actual !== expected) {
                        throw new Error("Expected " + actual + " to equal " + expected);
                    }
                },
                be: {
                    a: function(type) {
                        if (typeof actual !== type) {
                            throw new Error("Expected type " + type + " but got " + (typeof actual));
                        }
                    },
                    true: function() {
                        if (actual !== true) throw new Error("Expected true but got " + actual);
                    },
                    false: function() {
                        if (actual !== false) throw new Error("Expected false but got " + actual);
                    },
                    null: function() {
                        if (actual !== null) throw new Error("Expected null but got " + actual);
                    }
                },
                include: function(sub) {
                    if (!String(actual).includes(String(sub))) {
                        throw new Error("Expected '" + actual + "' to include '" + sub + "'");
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
        expect: expect,
        test: function(name, fn) {
            __veyak_test__(String(name), fn);
        }
    };
})();

// Aliases for convenience
var vy = veyak;
var pm = veyak;
"#;

fn execute_quickjs(
    script: &str,
    env_vars: &HashMap<String, String>,
    request: Option<&ApiRequest>,
    response: Option<&ApiResponse>,
) -> AppResult<ScriptResult> {
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

    let data_injection = format!(
        "var __envData__ = {env_json};\
         var __requestData__ = {request_json};\
         var __responseData__ = {response_json};\n"
    );

    let rt = Runtime::new().map_err(|e| AppError::Script(format!("QuickJS runtime: {e}")))?;
    let ctx = Context::full(&rt).map_err(|e| AppError::Script(format!("QuickJS context: {e}")))?;

    let env_updates = std::sync::Arc::new(std::sync::Mutex::new(HashMap::<String, String>::new()));
    let console_lines = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let test_results = std::sync::Arc::new(std::sync::Mutex::new(Vec::<ScriptTestResult>::new()));

    let env_updates_c = env_updates.clone();
    let console_c = console_lines.clone();
    let test_results_c = test_results.clone();

    ctx.with(|ctx| -> rquickjs::Result<()> {
        // console logging
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

        // veyak.environment.set bridge
        let eu = env_updates_c.clone();
        let set_fn = Function::new(ctx.clone(), move |key: String, value: String| {
            if let Ok(mut g) = eu.lock() {
                g.insert(key, value);
            }
        })?;

        // veyak.test bridge
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
                    g.push(ScriptTestResult { name, passed, error });
                }
                Ok(())
            },
        )?;

        ctx.globals().set("__veyak_set_env__", set_fn)?;
        ctx.globals().set("__veyak_test__", test_fn)?;

        // Step 1: inject data
        ctx.eval::<(), _>(data_injection.as_str())?;
        // Step 2: load veyak bootstrap
        ctx.eval::<(), _>(VEYAK_BOOTSTRAP_JS)?;
        // Step 3: execute user script
        ctx.eval::<(), _>(script.to_string())?;

        Ok(())
    })
    .map_err(|e| AppError::Script(format!("JavaScript execution error: {e}")))?;

    let env_result = env_updates.lock().unwrap().clone();
    let console_result = console_lines.lock().unwrap().clone();
    let test_result = test_results.lock().unwrap().clone();

    Ok(ScriptResult {
        env_updates: env_result,
        console_output: console_result,
        test_results: test_result,
    })
}

// ===========================================================================
// Python Engine (python3)
// ===========================================================================

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExternalScriptOutput {
    #[serde(default)]
    env_updates: HashMap<String, String>,
    #[serde(default)]
    console_output: Vec<String>,
    #[serde(default)]
    test_results: Vec<ScriptTestResult>,
}

fn execute_python(
    script: &str,
    env_vars: &HashMap<String, String>,
    request: Option<&ApiRequest>,
    response: Option<&ApiResponse>,
) -> AppResult<ScriptResult> {
    let python_cmd = which_python().ok_or_else(|| {
        AppError::Script(
            "Python 3 is not found in PATH. Please install Python to run Python scripts.".into(),
        )
    })?;

    let env_json = serde_json::to_string(env_vars).unwrap_or_else(|_| "{}".into());
    let request_json = serde_json::to_string(&request).unwrap_or_else(|_| "None".into());
    let response_json = serde_json::to_string(&response).unwrap_or_else(|_| "None".into());

    let harness = format!(
        r#"
import sys, json

_env_data = json.loads({env_json:?})
_req_data = json.loads({request_json:?}) if {request_json:?} != "None" else None
_resp_data = json.loads({response_json:?}) if {response_json:?} != "None" else None

_env_updates = dict()
_console_output = []
_test_results = []

class Environment:
    def __init__(self, data):
        self._data = data
    def get(self, key, default=""):
        return str(self._data.get(key, default))
    def set(self, key, value):
        self._data[key] = str(value)
        _env_updates[str(key)] = str(value)
    def has(self, key):
        return key in self._data

class Response:
    def __init__(self, data):
        self._data = data or {{}}
        self.status = self._data.get("status", 0)
        self.code = self.status
        self.headers = self._data.get("headers", {{}})
        self.body = self._data.get("body", "")
    def json(self):
        try:
            return json.loads(self.body)
        except Exception:
            return {{}}
    def text(self):
        return self.body

class Veyak:
    def __init__(self):
        self.environment = Environment(_env_data)
        self.request = _req_data
        self.response = Response(_resp_data) if _resp_data else None
    def test(self, name, fn):
        try:
            res = fn()
            passed = bool(res) if res is not None else True
            _test_results.append({{"name": str(name), "passed": passed, "error": None if passed else "Assertion failed"}})
        except Exception as e:
            _test_results.append({{"name": str(name), "passed": False, "error": str(e)}})

veyak = Veyak()
vy = veyak
pm = veyak

# Redefine print to capture logs
_original_print = print
def custom_print(*args, **kwargs):
    msg = " ".join(str(a) for a in args)
    _console_output.append(f"[python] {{msg}}")
print = custom_print

try:
{indented_user_script}
except Exception as e:
    _console_output.append(f"[error] {{e}}")

result = {{
    "envUpdates": _env_updates,
    "consoleOutput": _console_output,
    "testResults": _test_results
}}
_original_print("__VEYAK_OUTPUT_START__" + json.dumps(result) + "__VEYAK_OUTPUT_END__")
"#,
        env_json = env_json,
        request_json = request_json,
        response_json = response_json,
        indented_user_script = indent_code(script, 4)
    );

    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join(format!("veyak_script_{}.py", veyak_db::new_id()));
    std::fs::write(&script_path, harness)
        .map_err(|e| AppError::Script(format!("Failed to write python temp script: {e}")))?;

    let output = Command::new(&python_cmd)
        .arg(&script_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| AppError::Script(format!("Failed to execute python: {e}")))?;

    let _ = std::fs::remove_file(&script_path);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    parse_external_output(&stdout, &stderr)
}

fn which_python() -> Option<String> {
    for cmd in &["python3", "python"] {
        if let Ok(out) = Command::new(cmd).arg("--version").output() {
            if out.status.success() {
                return Some(cmd.to_string());
            }
        }
    }
    None
}

// ===========================================================================
// Go Engine (go run)
// ===========================================================================

fn execute_golang(
    script: &str,
    env_vars: &HashMap<String, String>,
    request: Option<&ApiRequest>,
    response: Option<&ApiResponse>,
) -> AppResult<ScriptResult> {
    let go_cmd = which_go().ok_or_else(|| {
        AppError::Script("Go is not found in PATH. Please install Go to run Go scripts.".into())
    })?;

    let env_json = serde_json::to_string(env_vars).unwrap_or_else(|_| "{}".into());
    let request_json = serde_json::to_string(&request).unwrap_or_else(|_| "null".into());
    let response_json = serde_json::to_string(&response).unwrap_or_else(|_| "null".into());

    let harness = format!(
        r#"package main

import (
	"encoding/json"
	"fmt"
)

type ScriptTestResult struct {{
	Name   string  `json:"name"`
	Passed bool    `json:"passed"`
	Error  *string `json:"error"`
}}

type Environment struct {{
	data map[string]string
}}

var envUpdates = make(map[string]string)
var consoleOutput = []string{{}}
var testResults = []ScriptTestResult{{}}

func (e *Environment) Get(key string) string {{
	return e.data[key]
}}

func (e *Environment) Set(key, val string) {{
	e.data[key] = val
	envUpdates[key] = val
}}

type Response struct {{
	Status  int                    `json:"status"`
	Headers map[string]string      `json:"headers"`
	Body    string                 `json:"body"`
}}

func (r *Response) JSON() map[string]interface{{}} {{
	var res map[string]interface{{}}
	_ = json.Unmarshal([]byte(r.Body), &res)
	return res
}}

type Veyak struct {{
	Environment *Environment
	Request     interface{{}}
	Response    *Response
}}

func (v *Veyak) Log(msg string) {{
	consoleOutput = append(consoleOutput, fmt.Sprintf("[go] %s", msg))
}}

func (v *Veyak) Test(name string, passed bool) {{
	var errStr *string
	if !passed {{
		msg := "Assertion failed"
		errStr = &msg
	}}
	testResults = append(testResults, ScriptTestResult{{Name: name, Passed: passed, Error: errStr}})
}}

func main() {{
	var envData map[string]string
	_ = json.Unmarshal([]byte(`{env_json}`), &envData)

	var reqData interface{{}}
	_ = json.Unmarshal([]byte(`{request_json}`), &reqData)

	var respData *Response
	if `{response_json}` != "null" {{
		_ = json.Unmarshal([]byte(`{response_json}`), &respData)
	}}

	veyak := &Veyak{{
		Environment: &Environment{{data: envData}},
		Request:     reqData,
		Response:    respData,
	}}
	vy := veyak
	_ = vy

	// User Script Block
	runUserScript := func(v *Veyak) {{
		{user_script}
	}}
	runUserScript(veyak)

	out := map[string]interface{{}}{{
		"envUpdates":    envUpdates,
		"consoleOutput": consoleOutput,
		"testResults":   testResults,
	}}
	bytes, _ := json.Marshal(out)
	fmt.Println("__VEYAK_OUTPUT_START__" + string(bytes) + "__VEYAK_OUTPUT_END__")
}}
"#,
        env_json = env_json,
        request_json = request_json,
        response_json = response_json,
        user_script = script
    );

    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join(format!("veyak_script_{}.go", veyak_db::new_id()));
    std::fs::write(&script_path, harness)
        .map_err(|e| AppError::Script(format!("Failed to write Go temp script: {e}")))?;

    let output = Command::new(&go_cmd)
        .args(["run", script_path.to_str().unwrap_or_default()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| AppError::Script(format!("Failed to execute Go: {e}")))?;

    let _ = std::fs::remove_file(&script_path);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    parse_external_output(&stdout, &stderr)
}

fn which_go() -> Option<String> {
    if let Ok(out) = Command::new("go").arg("version").output() {
        if out.status.success() {
            return Some("go".to_string());
        }
    }
    None
}

// ===========================================================================
// Rust Engine
// ===========================================================================

fn execute_rust(
    script: &str,
    env_vars: &HashMap<String, String>,
    request: Option<&ApiRequest>,
    response: Option<&ApiResponse>,
) -> AppResult<ScriptResult> {
    let env_json = serde_json::to_string(env_vars).unwrap_or_else(|_| "{}".into());
    let request_json = serde_json::to_string(&request).unwrap_or_else(|_| "null".into());
    let response_json = serde_json::to_string(&response).unwrap_or_else(|_| "null".into());

    let temp_dir = std::env::temp_dir().join(format!("veyak_rust_{}", veyak_db::new_id()));
    let _ = std::fs::create_dir_all(&temp_dir);

    let cargo_toml = r#"[package]
name = "veyak_script_runner"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#;

    let main_rs = format!(
        r#"use std::collections::HashMap;
use serde::{{Serialize, Deserialize}};

#[derive(Serialize, Deserialize, Clone)]
struct ScriptTestResult {{
    name: String,
    passed: bool,
    error: Option<String>,
}}

struct VeyakContext {{
    pub env_data: HashMap<String, String>,
    pub env_updates: HashMap<String, String>,
    pub console_output: Vec<String>,
    pub test_results: Vec<ScriptTestResult>,
    pub request: serde_json::Value,
    pub response: Option<serde_json::Value>,
}}

impl VeyakContext {{
    pub fn response_json(&self) -> Option<serde_json::Value> {{
        let body_str = self.response.as_ref()?.get("body")?.as_str()?;
        serde_json::from_str(body_str).ok()
    }}
    pub fn get_env(&self, key: &str) -> String {{
        self.env_data.get(key).cloned().unwrap_or_default()
    }}
    pub fn set_env(&mut self, key: &str, val: &str) {{
        self.env_data.insert(key.to_string(), val.to_string());
        self.env_updates.insert(key.to_string(), val.to_string());
    }}
    pub fn log(&mut self, msg: &str) {{
        self.console_output.push(format!("[rust] {{msg}}"));
    }}
    pub fn test(&mut self, name: &str, passed: bool) {{
        self.test_results.push(ScriptTestResult {{
            name: name.to_string(),
            passed,
            error: if passed {{ None }} else {{ Some("Assertion failed".to_string()) }},
        }});
    }}
}}

fn main() {{
    let env_data: HashMap<String, String> = serde_json::from_str({env_json:?}).unwrap_or_default();
    let req_val: serde_json::Value = serde_json::from_str({request_json:?}).unwrap_or(serde_json::Value::Null);
    let resp_val: Option<serde_json::Value> = serde_json::from_str({response_json:?}).ok();

    let mut veyak = VeyakContext {{
        env_data,
        env_updates: HashMap::new(),
        console_output: Vec::new(),
        test_results: Vec::new(),
        request: req_val,
        response: resp_val,
    }};

    // User Rust code
    let mut run_script = |v: &mut VeyakContext| {{
        {user_script}
    }};
    run_script(&mut veyak);

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Output {{
        env_updates: HashMap<String, String>,
        console_output: Vec<String>,
        test_results: Vec<ScriptTestResult>,
    }}

    let out = Output {{
        env_updates: veyak.env_updates,
        console_output: veyak.console_output,
        test_results: veyak.test_results,
    }};
    println!("__VEYAK_OUTPUT_START__{{}}__VEYAK_OUTPUT_END__", serde_json::to_string(&out).unwrap());
}}
"#,
        env_json = env_json,
        request_json = request_json,
        response_json = response_json,
        user_script = script
    );

    let _ = std::fs::write(temp_dir.join("Cargo.toml"), cargo_toml);
    let src_dir = temp_dir.join("src");
    let _ = std::fs::create_dir_all(&src_dir);
    let _ = std::fs::write(src_dir.join("main.rs"), main_rs);

    let output = Command::new("cargo")
        .args(["run", "--quiet", "--manifest-path", temp_dir.join("Cargo.toml").to_str().unwrap()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| AppError::Script(format!("Failed to compile/execute Rust script: {e}")))?;

    let _ = std::fs::remove_dir_all(&temp_dir);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    parse_external_output(&stdout, &stderr)
}

// ===========================================================================
// Helpers
// ===========================================================================

fn indent_code(code: &str, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    code.lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("{pad}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_external_output(stdout: &str, stderr: &str) -> AppResult<ScriptResult> {
    if let Some(start) = stdout.find("__VEYAK_OUTPUT_START__") {
        let after_start = &stdout[start + "__VEYAK_OUTPUT_START__".len()..];
        if let Some(end) = after_start.find("__VEYAK_OUTPUT_END__") {
            let json_str = &after_start[..end];
            if let Ok(parsed) = serde_json::from_str::<ExternalScriptOutput>(json_str) {
                return Ok(ScriptResult {
                    env_updates: parsed.env_updates,
                    console_output: parsed.console_output,
                    test_results: parsed.test_results,
                });
            }
        }
    }

    if !stderr.trim().is_empty() {
        return Err(AppError::Script(format!("Script execution error: {stderr}")));
    }

    Ok(ScriptResult {
        env_updates: HashMap::new(),
        console_output: if stdout.trim().is_empty() {
            vec![]
        } else {
            vec![stdout.to_string()]
        },
        test_results: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use veyak_models::{AuthConfig, HttpMethod, RequestBody};

    fn dummy_request() -> ApiRequest {
        ApiRequest {
            id: "req_test".into(),
            collection_id: "col_test".into(),
            folder_id: None,
            sort_order: 0,
            name: "Test Request".into(),
            method: HttpMethod::Get,
            url: "https://api.example.com".into(),
            params: vec![],
            headers: vec![],
            cookies: vec![],
            auth: AuthConfig::default(),
            body: RequestBody::default(),
            pre_request_script: None,
            pre_request_language: None,
            post_request_script: None,
            post_request_language: None,
        }
    }

    fn dummy_response() -> ApiResponse {
        ApiResponse {
            status: 200,
            status_text: "OK".into(),
            time_ms: 50,
            size_bytes: 120,
            headers: std::collections::BTreeMap::new(),
            cookies: vec![],
            body: r#"{"token": "test_token_123", "user": {"id": 42}}"#.into(),
            console_output: None,
            test_results: None,
            env_updates: None,
        }
    }

    #[test]
    fn test_quickjs_pre_request_script() {
        let mut req = dummy_request();
        req.pre_request_script = Some(
            r#"
            veyak.environment.set("token", "pre_token_abc");
            console.log("Pre-request executed");
        "#
            .into(),
        );

        let env_vars = HashMap::new();
        let (updated_req, result) = run_pre_request_script(req, &env_vars).unwrap();

        assert_eq!(updated_req.id, "req_test");
        assert_eq!(
            result.env_updates.get("token").map(String::as_str),
            Some("pre_token_abc")
        );
        assert!(result.console_output.iter().any(|l| l.contains("Pre-request executed")));
    }

    #[test]
    fn test_quickjs_post_request_script_extract_json_and_tests() {
        let mut req = dummy_request();
        req.post_request_script = Some(
            r#"
            const data = veyak.response.json();
            veyak.environment.set("auth_token", data.token);
            veyak.test("Status is 200", () => {
                veyak.response.to.have.status(200);
            });
            veyak.test("Token is string", () => {
                veyak.expect(data.token).to.be.a("string");
            });
        "#
            .into(),
        );

        let resp = dummy_response();
        let env_vars = HashMap::new();
        let result = run_post_request_script(&req, &resp, &env_vars).unwrap();

        assert_eq!(
            result.env_updates.get("auth_token").map(String::as_str),
            Some("test_token_123")
        );
        assert_eq!(result.test_results.len(), 2);
        assert!(result.test_results[0].passed);
        assert!(result.test_results[1].passed);
    }

    #[test]
    fn test_python_post_request_script() {
        if which_python().is_none() {
            eprintln!("Skipping Python test: python3 not installed");
            return;
        }

        let mut req = dummy_request();
        req.post_request_language = Some(ScriptLanguage::Python);
        req.post_request_script = Some(
            r#"
data = veyak.response.json()
veyak.environment.set("py_token", data.get("token"))
veyak.test("Status code is 200", lambda: veyak.response.status == 200)
print("Extracted token from Python")
"#
            .into(),
        );

        let resp = dummy_response();
        let env_vars = HashMap::new();
        let result = run_post_request_script(&req, &resp, &env_vars).unwrap();

        assert_eq!(
            result.env_updates.get("py_token").map(String::as_str),
            Some("test_token_123")
        );
        assert_eq!(result.test_results.len(), 1);
        assert!(result.test_results[0].passed);
        assert!(result.console_output.iter().any(|l| l.contains("Extracted token from Python")));
    }

    #[test]
    fn test_go_post_request_script() {
        if which_go().is_none() {
            eprintln!("Skipping Go test: go not installed");
            return;
        }

        let mut req = dummy_request();
        req.post_request_language = Some(ScriptLanguage::Go);
        req.post_request_script = Some(
            r#"
            data := v.Response.JSON()
            if token, ok := data["token"].(string); ok {
                v.Environment.Set("go_token", token)
            }
            v.Test("Go status check", v.Response.Status == 200)
            v.Log("Go script executed successfully")
"#
            .into(),
        );

        let resp = dummy_response();
        let env_vars = HashMap::new();
        let result = run_post_request_script(&req, &resp, &env_vars).unwrap();

        assert_eq!(
            result.env_updates.get("go_token").map(String::as_str),
            Some("test_token_123")
        );
        assert_eq!(result.test_results.len(), 1);
        assert!(result.test_results[0].passed);
        assert!(result.console_output.iter().any(|l| l.contains("Go script executed successfully")));
    }

    #[test]
    fn test_rust_post_request_script() {
        let mut req = dummy_request();
        req.post_request_language = Some(ScriptLanguage::Rust);
        req.post_request_script = Some(
            r#"
            if let Some(resp) = v.response_json() {
                if let Some(token) = resp.get("token").and_then(|t| t.as_str()) {
                    v.set_env("rust_token", token);
                }
            }
            let is_200 = v.response.as_ref().map(|r| r["status"] == 200).unwrap_or(false);
            v.test("Rust status check", is_200);
            v.log("Rust script executed successfully");
"#
            .into(),
        );

        let resp = dummy_response();
        let env_vars = HashMap::new();
        let result = run_post_request_script(&req, &resp, &env_vars).unwrap();

        assert_eq!(
            result.env_updates.get("rust_token").map(String::as_str),
            Some("test_token_123")
        );
        assert_eq!(result.test_results.len(), 1);
        assert!(result.test_results[0].passed);
        assert!(result.console_output.iter().any(|l| l.contains("Rust script executed successfully")));
    }
}
