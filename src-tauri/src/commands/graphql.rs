use crate::{db, state::AppState};
use std::collections::BTreeMap;
use tauri::{command, AppHandle, Emitter, Manager, State, Window};
use veyak_error::AppResult;
use veyak_graphql::{
    client::execute_graphql, introspection::fetch_schema, state::GraphQlSubscriptionHandle,
    subscription::start_subscription,
};
use veyak_models::{
    AuthConfig, AuthType, GraphQlRequest, GraphQlResponse, GraphQlSchema, GraphQlSubscriptionEvent,
    KeyValueRow,
};

/// Convert a `Vec<KeyValueRow>` into a `BTreeMap`, skipping disabled/empty rows.
fn rows_to_map(rows: &[KeyValueRow]) -> BTreeMap<String, String> {
    rows.iter()
        .filter(|r| r.enabled && !r.key.trim().is_empty())
        .map(|r| (r.key.clone(), r.value.clone()))
        .collect()
}

/// Inject auth credentials into the header map.
fn apply_auth_headers(auth: &AuthConfig, headers: &mut BTreeMap<String, String>) {
    match auth.auth_type {
        AuthType::Bearer => {
            if let Some(bearer) = &auth.bearer {
                if !bearer.token.is_empty() {
                    headers.insert(
                        "Authorization".to_string(),
                        format!("Bearer {}", bearer.token),
                    );
                }
            }
        }
        AuthType::Basic => {
            if let Some(basic) = &auth.basic {
                use base64::Engine;
                let encoded = base64::engine::general_purpose::STANDARD
                    .encode(format!("{}:{}", basic.username, basic.password));
                headers.insert("Authorization".to_string(), format!("Basic {encoded}"));
            }
        }
        AuthType::ApiKey => {
            if let Some(api_key) = &auth.api_key {
                use veyak_models::ApiKeyTarget;
                if api_key.add_to == ApiKeyTarget::Header && !api_key.key.is_empty() {
                    headers.insert(api_key.key.clone(), api_key.value.clone());
                }
            }
        }
        AuthType::None => {}
    }
}

/// Fetch and parse the GraphQL schema via the standard introspection query.
///
/// Returns a fully parsed [`GraphQlSchema`] containing all types, fields, args,
/// and enum values — used to populate the Docs drawer in the frontend.
#[command]
pub async fn graphql_introspect(
    state: State<'_, AppState>,
    url: String,
    headers: Vec<KeyValueRow>,
    auth: Option<AuthConfig>,
) -> AppResult<GraphQlSchema> {
    let settings = crate::db::settings::get_settings(&state.data_dir)?;
    let mut header_map = rows_to_map(&headers);
    if let Some(auth_cfg) = &auth {
        apply_auth_headers(auth_cfg, &mut header_map);
    }
    fetch_schema(&url, header_map, settings.verify_ssl_certificates).await
}

/// Execute a GraphQL query or mutation synchronously over HTTP POST.
///
/// All HTTP work, JSON parsing, and data/errors/extensions extraction
/// happens in Rust. The frontend receives a [`GraphQlResponse`] ready to render.
#[command]
pub async fn graphql_execute(
    state: State<'_, AppState>,
    request: GraphQlRequest,
) -> AppResult<GraphQlResponse> {
    let settings = crate::db::settings::get_settings(&state.data_dir)?;

    let mut headers = rows_to_map(&request.headers);
    apply_auth_headers(&request.auth, &mut headers);

    let variables = if request.variables.trim().is_empty() {
        None
    } else {
        serde_json::from_str::<serde_json::Value>(&request.variables).ok()
    };

    let res = execute_graphql(
        &request.url,
        &request.query,
        variables,
        request.operation_name.as_deref(),
        headers,
        settings.verify_ssl_certificates,
        settings.timeout_ms,
    )
    .await?;

    let _ = crate::db::history::add_entry(
        &state.data_dir,
        Some(&request.id),
        veyak_models::HttpMethod::Post,
        Some(&request.name),
        &request.url,
        res.status,
        res.time_ms as u128,
        None,
        Some(veyak_models::ApiResponse {
            status: res.status,
            status_text: res.status_text.clone(),
            time_ms: res.time_ms,
            size_bytes: res.size_bytes,
            headers: res.headers.clone(),
            cookies: Vec::new(),
            body: res.data.clone().or_else(|| res.errors.clone()).unwrap_or_default(),
            console_output: None,
            test_results: None,
            env_updates: None,
        }),
    );

    Ok(res)
}

/// Start a `graphql-ws` subscription.
///
/// Establishes the WebSocket, performs the protocol handshake, and spawns
/// a background Tokio task that drives the read loop. Events are forwarded
/// to the frontend via the Tauri `graphql://event` channel.
///
/// Returns the `connection_id` used to cancel the subscription later.
#[command]
pub async fn graphql_subscribe(
    state: State<'_, AppState>,
    window: Window,
    request: GraphQlRequest,
) -> AppResult<String> {
    let dd = &state.data_dir;

    let settings = db::settings::get_settings(dd)?;
    let mut headers = rows_to_map(&request.headers);
    apply_auth_headers(&request.auth, &mut headers);

    let variables = if request.variables.trim().is_empty() {
        None
    } else {
        serde_json::from_str::<serde_json::Value>(&request.variables).ok()
    };

    let handle = start_subscription(
        &request.url,
        &request.query,
        variables,
        request.operation_name,
        headers,
        settings,
    )
    .await?;

    let connection_id = request.id.clone();
    let cancel_token = handle.cancel_token.clone();
    let mut rx = handle.rx;
    let cid = connection_id.clone();
    let app_handle: AppHandle = window.app_handle().clone();

    // Register in state so graphql_unsubscribe can cancel it
    state
        .graphql_state
        .insert(GraphQlSubscriptionHandle {
            connection_id: connection_id.clone(),
            cancel_token,
        })
        .await;

    // Forward subscription events to the frontend via Tauri events
    let graphql_state = state.graphql_state.clone();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let timestamp = chrono::Utc::now().to_rfc3339();
            let is_terminal = event.event_type == "complete" || event.event_type == "error";

            let _ = app_handle.emit(
                "graphql://event",
                GraphQlSubscriptionEvent {
                    connection_id: cid.clone(),
                    event_type: event.event_type.clone(),
                    payload: event.payload,
                    timestamp,
                },
            );

            if is_terminal {
                break;
            }
        }
        // Clean up the state entry when the loop ends
        graphql_state.remove(&cid).await;
    });

    Ok(connection_id)
}

/// Cancel an active GraphQL subscription by its `connection_id`.
///
/// Sends the `graphql-ws` `complete` message through the cancellation token
/// and removes the handle from state. The background task will emit a final
/// `graphql://event` with `event_type: "complete"` and terminate.
#[command]
pub async fn graphql_unsubscribe(
    state: State<'_, AppState>,
    connection_id: String,
) -> AppResult<()> {
    state.graphql_state.cancel(&connection_id).await;
    Ok(())
}
