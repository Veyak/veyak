use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE, SET_COOKIE};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::time::{Duration, Instant};
use tauri::State;

use crate::db;
use veyak_error::{AppError, AppResult};
use veyak_models::{
    ApiKeyTarget, ApiRequest, ApiResponse, AppSettings, AuthType, BodyMode, CookieRow,
};

#[cfg(feature = "scripting")]
use crate::models::PluginHook;

/// Executes an `ApiRequest` end to end: runs `preRequest` plugin hooks,
/// interpolates `{{variables}}` from the active environment, applies
/// user settings (redirects, TLS validation, proxy, timeout) to the HTTP
/// client, sends the request, runs `postResponse` plugin hooks, records
/// a history entry, and returns the final `ApiResponse`.
#[tauri::command]
pub async fn send_request(
    state: State<'_, AppState>,
    request: ApiRequest,
) -> AppResult<ApiResponse> {
    let dd = &state.data_dir;

    let settings = db::settings::get_settings(dd)?;
    let active = db::app_state::get_active_state(dd)?;
    let active_env_id = active
        .collection_active_environments
        .get(&request.collection_id)
        .map(String::as_str)
        .or(active.active_environment_id.as_deref());
    let env_vars = db::environments::active_variable_map(dd, active_env_id)?;

    #[cfg(feature = "scripting")]
    {
        request = run_pre_request_hooks(dd, request, &env_vars)?;
    }

    let interpolated = interpolate_request(&request, &env_vars);

    // WebSocket requests should never go through the HTTP pipeline.
    if interpolated.method == veyak_models::HttpMethod::Ws {
        return Err(AppError::Invalid(
            "WebSocket requests cannot be sent via HTTP — use the WebSocket panel instead".into(),
        ));
    }

    let client = build_client(&settings)?;
    let url =
        build_url(&interpolated).map_err(|e| AppError::Invalid(format!("invalid URL: {e}")))?;

    let mut builder = client.request(interpolated.method.as_reqwest(), url);
    builder = apply_headers(builder, &interpolated)?;
    builder = apply_auth(builder, &interpolated);
    builder = apply_body(builder, &interpolated).await?;

    let started = Instant::now();
    let send_result = builder.send().await;
    let elapsed = started.elapsed();

    let response = send_result.map_err(|e| AppError::Http(describe_reqwest_error(&e)))?;

    let status = response.status();
    let status_text = status.canonical_reason().unwrap_or("Unknown").to_string();
    let headers = collect_headers(response.headers());
    let cookies = collect_cookies(response.headers());

    let bytes = response
        .bytes()
        .await
        .map_err(|e| AppError::Http(format!("failed to read response body: {e}")))?;
    let size_bytes = bytes.len() as u64;
    let body = String::from_utf8_lossy(&bytes).to_string();

    let api_response = ApiResponse {
        status: status.as_u16(),
        status_text,
        time_ms: elapsed.as_millis(),
        size_bytes,
        headers,
        cookies,
        body,
    };

    #[cfg(feature = "scripting")]
    {
        api_response =
            run_post_response_hooks(dd, &request, api_response, &env_vars, active_env_id)?;
    }

    db::history::add_entry(
        dd,
        Some(&request.id),
        interpolated.method,
        Some(&request.name),
        &interpolated.url,
        api_response.status,
        elapsed.as_millis(),
        None,
        None,
    )?;

    Ok(api_response)
}

use crate::state::AppState;

fn build_client(settings: &AppSettings) -> AppResult<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_millis(settings.timeout_ms))
        .user_agent(settings.user_agent.clone())
        // "Validate SSL certificates" off -> accept self-signed / invalid
        // certs. Named for the setting's effect, not reqwest's API.
        .danger_accept_invalid_certs(!settings.verify_ssl_certificates);

    builder = if settings.follow_redirects {
        builder.redirect(reqwest::redirect::Policy::limited(
            settings.max_redirects as usize,
        ))
    } else {
        builder.redirect(reqwest::redirect::Policy::none())
    };

    if let Some(proxy_url) = &settings.proxy_url {
        if !proxy_url.trim().is_empty() {
            let proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| AppError::Invalid(format!("invalid proxy URL: {e}")))?;
            builder = builder.proxy(proxy);
        }
    }

    builder
        .build()
        .map_err(|e| AppError::Other(format!("failed to build HTTP client: {e}")))
}

/// Returns a copy of `request` with `{{variables}}` resolved in the URL,
/// enabled header/param values, and the raw body. Disabled rows are left
/// alone since they won't be sent anyway.
pub fn interpolate_request(request: &ApiRequest, vars: &HashMap<String, String>) -> ApiRequest {
    let mut next = request.clone();

    next.url = db::environments::interpolate(&next.url, vars);

    for row in next.headers.iter_mut().filter(|r| r.enabled) {
        row.value = db::environments::interpolate(&row.value, vars);
    }
    for row in next.params.iter_mut().filter(|r| r.enabled) {
        row.value = db::environments::interpolate(&row.value, vars);
    }
    if let Some(raw) = next.body.raw.as_mut() {
        *raw = db::environments::interpolate(raw, vars);
    }
    if let Some(bearer) = next.auth.bearer.as_mut() {
        bearer.token = db::environments::interpolate(&bearer.token, vars);
    }

    next
}

fn build_url(request: &ApiRequest) -> Result<reqwest::Url, url::ParseError> {
    let mut url = reqwest::Url::parse(&request.url)?;

    let mut api_key_query = None;
    if request.auth.auth_type == AuthType::ApiKey {
        if let Some(api_key) = &request.auth.api_key {
            if api_key.add_to == ApiKeyTarget::Query && !api_key.key.is_empty() {
                api_key_query = Some((api_key.key.clone(), api_key.value.clone()));
            }
        }
    }

    let has_params = request
        .params
        .iter()
        .any(|r| r.enabled && !r.key.is_empty());

    if has_params || api_key_query.is_some() {
        let mut pairs = url.query_pairs_mut();
        for row in request
            .params
            .iter()
            .filter(|r| r.enabled && !r.key.is_empty())
        {
            pairs.append_pair(&row.key, &row.value);
        }
        if let Some((k, v)) = api_key_query {
            pairs.append_pair(&k, &v);
        }
    }

    Ok(url)
}

fn apply_headers(
    mut builder: reqwest::RequestBuilder,
    request: &ApiRequest,
) -> AppResult<reqwest::RequestBuilder> {
    for row in request
        .headers
        .iter()
        .filter(|r| r.enabled && !r.key.is_empty())
    {
        let name = HeaderName::from_bytes(row.key.as_bytes())
            .map_err(|e| AppError::Invalid(format!("invalid header name '{}': {e}", row.key)))?;
        let value = HeaderValue::from_str(&row.value).map_err(|e| {
            AppError::Invalid(format!("invalid header value for '{}': {e}", row.key))
        })?;
        builder = builder.header(name, value);
    }

    if request.auth.auth_type == AuthType::ApiKey {
        if let Some(api_key) = &request.auth.api_key {
            if api_key.add_to == ApiKeyTarget::Header && !api_key.key.is_empty() {
                builder = builder.header(api_key.key.clone(), api_key.value.clone());
            }
        }
    }

    Ok(builder)
}

fn apply_auth(
    mut builder: reqwest::RequestBuilder,
    request: &ApiRequest,
) -> reqwest::RequestBuilder {
    match request.auth.auth_type {
        AuthType::Basic => {
            if let Some(basic) = &request.auth.basic {
                builder = builder.basic_auth(&basic.username, Some(&basic.password));
            }
        }
        AuthType::Bearer => {
            if let Some(bearer) = &request.auth.bearer {
                builder = builder.bearer_auth(&bearer.token);
            }
        }
        AuthType::ApiKey | AuthType::None => {}
    }
    builder
}

async fn apply_body(
    mut builder: reqwest::RequestBuilder,
    request: &ApiRequest,
) -> AppResult<reqwest::RequestBuilder> {
    let mode = request.body.mode.unwrap_or(BodyMode::Json);
    match mode {
        BodyMode::Json => {
            if let Some(raw) = &request.body.raw {
                if !raw.trim().is_empty() {
                    serde_json::from_str::<Value>(raw)
                        .map_err(|e| AppError::Invalid(format!("body is not valid JSON: {e}")))?;
                    builder = builder
                        .header(CONTENT_TYPE, "application/json")
                        .body(raw.clone());
                }
            }
        }
        BodyMode::Raw => {
            if let Some(raw) = &request.body.raw {
                builder = builder.body(raw.clone());
            }
        }
        BodyMode::Urlencoded => {
            let pairs: Vec<(String, String)> = request
                .body
                .url_encoded
                .as_ref()
                .map(|rows| {
                    rows.iter()
                        .filter(|r| r.enabled && !r.key.is_empty())
                        .map(|r| (r.key.clone(), r.value.clone()))
                        .collect()
                })
                .unwrap_or_default();
            builder = builder.form(&pairs);
        }
        BodyMode::FormData | BodyMode::Multipart => {
            let mut form = reqwest::multipart::Form::new();

            if let Some(rows) = &request.body.form_data {
                for row in rows.iter().filter(|r| r.enabled && !r.key.is_empty()) {
                    form = form.text(row.key.clone(), row.value.clone());
                }
            }

            if let Some(files) = &request.body.files {
                for file in files {
                    let file_bytes = match tokio::fs::read(&file.path)
                        .await
                        .map_err(|e| format!("failed to read file '{}': {e}", file.name))
                    {
                        Ok(it) => it,
                        Err(err) => {
                            return Err(AppError::Other(format!(
                                "failed to read file '{}': {}",
                                file.name, err
                            )))
                        }
                    };

                    let part =
                        reqwest::multipart::Part::bytes(file_bytes).file_name(file.name.clone());
                    form = form.part(file.name.clone(), part);
                }
            }

            builder = builder.multipart(form);
        }
    }
    Ok(builder)
}

fn collect_headers(headers: &HeaderMap) -> BTreeMap<String, String> {
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    for (name, value) in headers.iter() {
        let value_str = value.to_str().unwrap_or("").to_string();
        out.entry(name.as_str().to_lowercase())
            .and_modify(|existing| {
                existing.push_str(", ");
                existing.push_str(&value_str);
            })
            .or_insert(value_str);
    }
    out
}

/// Pragmatic `Set-Cookie` parser for the response viewer's Cookies tab —
/// name/value/domain only, no full RFC 6265 attribute handling
/// (Expires/Max-Age/SameSite), which isn't needed just to display them.
fn collect_cookies(headers: &HeaderMap) -> Vec<CookieRow> {
    headers
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .map(|raw| {
            let mut parts = raw.split(';').map(str::trim);
            let (name, value) = parts
                .next()
                .and_then(|kv| kv.split_once('='))
                .unwrap_or(("", ""));

            let domain = parts
                .find(|attr| attr.to_ascii_lowercase().starts_with("domain="))
                .and_then(|attr| attr.split_once('='))
                .map(|(_, v)| v.to_string())
                .unwrap_or_default();

            CookieRow {
                id: uuid::Uuid::new_v4().to_string(),
                name: name.to_string(),
                value: value.to_string(),
                domain,
            }
        })
        .collect()
}

fn describe_reqwest_error(e: &reqwest::Error) -> String {
    if e.is_timeout() {
        "request timed out".to_string()
    } else if e.is_connect() {
        format!("connection failed: {e}")
    } else {
        e.to_string()
    }
}

// ---------------------------------------------------------------------
// Plugin hooks
// ---------------------------------------------------------------------

#[cfg(feature = "scripting")]
fn run_pre_request_hooks(
    dd: &DataDir,
    request: ApiRequest,
    env_vars: &HashMap<String, String>,
) -> AppResult<ApiRequest> {
    let plugins = db::plugins::list_enabled_plugins_with_source(dd)?;
    let mut current = request;

    for (manifest, source) in plugins {
        if !manifest.hooks.contains(&PluginHook::PreRequest) {
            continue;
        }

        let ctx = json!({ "request": current, "environment": env_vars });
        let result = run_hook_sync(source, "preRequest", ctx)?;

        if let Some(request_value) = result.get("request") {
            current = serde_json::from_value(request_value.clone()).map_err(|e| {
                AppError::Script(format!(
                    "plugin '{}' returned an invalid request shape: {e}",
                    manifest.id
                ))
            })?;
        }
    }

    Ok(current)
}

#[cfg(feature = "scripting")]
fn run_post_response_hooks(
    dd: &DataDir,
    request: &ApiRequest,
    response: ApiResponse,
    env_vars: &HashMap<String, String>,
    active_environment_id: Option<&str>,
) -> AppResult<ApiResponse> {
    let plugins = db::plugins::list_enabled_plugins_with_source(dd)?;
    let mut current = response;
    let mut env_updates: HashMap<String, String> = HashMap::new();

    for (manifest, source) in plugins {
        if !manifest.hooks.contains(&PluginHook::PostResponse) {
            continue;
        }

        let ctx = json!({ "request": request, "response": current, "environment": env_vars });
        let result = run_hook_sync(source, "postResponse", ctx)?;

        if let Some(response_value) = result.get("response") {
            current = serde_json::from_value(response_value.clone()).map_err(|e| {
                AppError::Script(format!(
                    "plugin '{}' returned an invalid response shape: {e}",
                    manifest.id
                ))
            })?;
        }

        // A plugin can hand back variables to persist into the active
        // environment — e.g. pulling a fresh access token out of a login
        // response so the next request picks it up automatically.
        if let Some(set_vars) = result
            .get("setEnvironmentVariables")
            .and_then(Value::as_object)
        {
            for (key, value) in set_vars {
                if let Some(value_str) = value.as_str() {
                    env_updates.insert(key.clone(), value_str.to_string());
                }
            }
        }
    }

    if !env_updates.is_empty() {
        if let Some(env_id) = active_environment_id {
            apply_environment_updates(dd, env_id, env_updates)?;
        }
    }

    Ok(current)
}

#[cfg(feature = "scripting")]
fn run_hook_sync(source: String, fn_name: &'static str, ctx: Value) -> AppResult<Value> {
    // Run the script on the current thread (plugins are expected to be
    // fast — the timeout guard has been dropped since we're now sync,
    // but the overall send_request is still behind Tauri's async runtime).
    crate::scripting::run_hook(&source, fn_name, &ctx)
}

#[cfg(feature = "scripting")]
fn apply_environment_updates(
    dd: &DataDir,
    environment_id: &str,
    updates: HashMap<String, String>,
) -> AppResult<()> {
    let (ws_id, col_id) = match db::environments::find_environment_location(dd, environment_id) {
        Ok(loc) => loc,
        Err(_) => return Ok(()),
    };
    let mut vars = db::environments::list_variables(dd, &ws_id, &col_id, environment_id)?;

    for (key, value) in &updates {
        match vars.iter_mut().find(|v| v.key == *key) {
            Some(existing) => existing.value = value.clone(),
            None => vars.push(veyak_models::EnvironmentVariable {
                id: String::new(),
                environmentid: environment_id.to_string(),
                key: key.clone(),
                value: value.clone(),
                enabled: true,
                is_secret: false,
            }),
        }
    }

    db::environments::replace_variables(dd, environment_id, &vars)?;
    Ok(())
}
