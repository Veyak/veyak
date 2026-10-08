use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE, SET_COOKIE};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::time::{Duration, Instant};
use tauri::State;

use crate::db;
use crate::scripting;
use veyak_error::{AppError, AppResult};
use veyak_models::{
    ApiKeyTarget, ApiRequest, ApiResponse, AppSettings, AuthType, BodyMode, CookieRow,
};

/// Executes an `ApiRequest` end to end:
/// 1. Runs the per-request **pre-request script** (JavaScript via QuickJS).
/// 2. Interpolates `{{variables}}` from the active environment.
/// 3. Applies user settings (redirects, TLS, proxy, timeout).
/// 4. Sends the HTTP request.
/// 5. Runs the per-request **post-request script** to extract response data
///    into environment variables.
/// 6. Persists a history entry and returns the `ApiResponse`.
#[tauri::command]
pub async fn send_request(
    state: State<'_, AppState>,
    mut request: ApiRequest,
) -> AppResult<ApiResponse> {
    let dd = &state.data_dir;

    let settings = db::settings::get_settings(dd)?;
    let active = db::app_state::get_active_state(dd)?;

    let mut target_env_id = active
        .collection_active_environments
        .get(&request.collection_id)
        .cloned()
        .or(active.active_environment_id.clone());

    // If no environment is currently selected, check if one exists in the collection or create Default
    if target_env_id.is_none() && !request.collection_id.is_empty() {
        if let Ok(ws_id) = db::collections::find_collection_workspace(dd, &request.collection_id) {
            if let Ok(envs) = db::environments::list_environments(dd, &ws_id, &request.collection_id) {
                if let Some(first) = envs.first() {
                    target_env_id = Some(first.environment.id.clone());
                } else if let Ok(new_env) =
                    db::environments::create_environment(dd, &ws_id, &request.collection_id, "Default")
                {
                    let _ = db::app_state::set_active_environment(
                        dd,
                        Some(&request.collection_id),
                        Some(&new_env.id),
                    );
                    target_env_id = Some(new_env.id);
                }
            }
        }
    }

    let mut env_vars = db::environments::active_variable_map(dd, target_env_id.as_deref())?;

    let mut all_console_output = Vec::new();
    let mut all_test_results = Vec::new();
    let mut all_env_updates = HashMap::new();

    // ── Pre-request script ────────────────────────────────────────────────
    match scripting::run_pre_request_script(request.clone(), &env_vars) {
        Ok((updated_request, pre_result)) => {
            request = updated_request;
            for (k, v) in &pre_result.env_updates {
                env_vars.insert(k.clone(), v.clone());
                all_env_updates.insert(k.clone(), v.clone());
            }
            all_console_output.extend(pre_result.console_output);
            all_test_results.extend(pre_result.test_results);

            if !pre_result.env_updates.is_empty() {
                if let Some(ref env_id) = target_env_id {
                    if let Err(e) = apply_env_updates(dd, env_id, pre_result.env_updates) {
                        log::warn!("Failed to persist pre-request env updates: {e}");
                    }
                }
            }
        }
        Err(e) => {
            all_console_output.push(format!("[pre-request error] {e}"));
            log::warn!("Pre-request script error (continuing): {e}");
        }
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

    let mut api_response = ApiResponse {
        status: status.as_u16(),
        status_text,
        time_ms: elapsed.as_millis(),
        size_bytes,
        headers,
        cookies,
        body,
        console_output: None,
        test_results: None,
        env_updates: None,
    };

    // ── Post-request script ───────────────────────────────────────────────
    match scripting::run_post_request_script(&request, &api_response, &env_vars) {
        Ok(post_result) => {
            for (k, v) in &post_result.env_updates {
                all_env_updates.insert(k.clone(), v.clone());
            }
            all_console_output.extend(post_result.console_output);
            all_test_results.extend(post_result.test_results);

            if !post_result.env_updates.is_empty() {
                if let Some(ref env_id) = target_env_id {
                    if let Err(e) = apply_env_updates(dd, env_id, post_result.env_updates) {
                        log::warn!("Failed to persist post-request env updates: {e}");
                    }
                }
            }
        }
        Err(e) => {
            all_console_output.push(format!("[post-request error] {e}"));
            log::warn!("Post-request script error: {e}");
        }
    }

    if !all_console_output.is_empty() {
        api_response.console_output = Some(all_console_output);
    }
    if !all_test_results.is_empty() {
        api_response.test_results = Some(all_test_results);
    }
    if !all_env_updates.is_empty() {
        api_response.env_updates = Some(all_env_updates.into_iter().collect());
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


// ---------------------------------------------------------------------------
// Environment update helper — shared by pre/post-request script runners
// ---------------------------------------------------------------------------

pub fn apply_env_updates(
    dd: &veyak_db::DataDir,
    environment_id: &str,
    updates: HashMap<String, String>,
) -> veyak_error::AppResult<()> {
    if updates.is_empty() {
        return Ok(());
    }
    let (ws_id, col_id) = match db::environments::find_environment_location(dd, environment_id) {
        Ok(loc) => loc,
        Err(_) => return Ok(()), // environment no longer exists — silently skip
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
