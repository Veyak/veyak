use serde_json::Value;
use std::str::FromStr;
use uuid::Uuid;
use veyak_error::{AppError, AppResult};
use veyak_models::{
    ApiKeyAuth, ApiKeyTarget, ApiRequest, AuthConfig, AuthType, BasicAuth, BearerAuth, BodyMode,
    EnvironmentVariable, GrpcMethodType, GrpcRequest, HttpMethod, KeyValueRow, RequestBody,
    RequestItem,
};

use crate::insomnia::v5_models::{InsomniaV5File, InsomniaV5Item};
use crate::types::{ImportData, ImportedCollection, ImportedEnvironment, ImportedFolder};

/// Returns true if the string content has indicators of the Insomnia v5 file format.
pub fn is_insomnia_v5(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return false;
    }

    if trimmed.contains("collection.insomnia.rest")
        || trimmed.contains("spec.insomnia.rest")
        || trimmed.contains("environment.insomnia.rest")
        || trimmed.contains("mock.insomnia.rest")
        || trimmed.contains("mcpClient.insomnia")
        || trimmed.contains("mcpclient.insomnia")
    {
        return true;
    }

    if let Ok(json) = serde_json::from_str::<Value>(trimmed) {
        if let Some(t) = json.get("type").and_then(|v| v.as_str()) {
            if is_insomnia_v5_type_str(t) {
                return true;
            }
        }
        if json.get("schema_version").is_some()
            && (json.get("collection").is_some()
                || json.get("environments").is_some()
                || json.get("routes").is_some())
        {
            return true;
        }
    } else if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(trimmed) {
        if let Some(t) = yaml.get("type").and_then(|v| v.as_str()) {
            if is_insomnia_v5_type_str(t) {
                return true;
            }
        }
        if yaml.get("schema_version").is_some()
            && (yaml.get("collection").is_some()
                || yaml.get("environments").is_some()
                || yaml.get("routes").is_some())
        {
            return true;
        }
    }

    false
}

pub fn is_insomnia_v5_type_str(t: &str) -> bool {
    let lower = t.to_ascii_lowercase();
    lower.contains("insomnia.rest")
        || lower.contains("insomnia/")
        || lower.starts_with("collection.insomnia")
        || lower.starts_with("spec.insomnia")
        || lower.starts_with("environment.insomnia")
        || lower.starts_with("mock.insomnia")
        || lower.starts_with("mcpclient.insomnia")
}

/// Parse content as an Insomnia v5 export (YAML or JSON).
pub fn import_insomnia_v5(content: &str) -> AppResult<ImportData> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(AppError::Invalid(
            "Empty content cannot be parsed as Insomnia v5".to_string(),
        ));
    }

    let root_val: Value = if let Ok(json) = serde_json::from_str::<Value>(trimmed) {
        json
    } else if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(trimmed) {
        serde_json::to_value(yaml)
            .map_err(|e| AppError::Invalid(format!("Failed to parse YAML content: {e}")))?
    } else {
        return Err(AppError::Invalid(
            "Failed to parse content as Insomnia v5 JSON or YAML".to_string(),
        ));
    };

    let root: InsomniaV5File = serde_json::from_value(root_val.clone()).map_err(|e| {
        AppError::Invalid(format!("Failed to parse Insomnia v5 file structure: {e}"))
    })?;

    let file_type = root.file_type.as_deref().unwrap_or("");
    let is_environment_file = file_type.starts_with("environment.insomnia.rest");

    let collection_name = root
        .name
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("Imported Insomnia Collection")
        .to_string();

    let mut folders = Vec::new();
    let mut requests = Vec::new();

    // 1. Process items in collection / requests / data / routes / mcpRequest
    let mut root_items = Vec::new();
    if let Some(col) = &root.collection {
        root_items.extend(col.clone());
    } else if let Some(reqs) = &root.requests {
        root_items.extend(reqs.clone());
    } else if let Some(data) = &root.data {
        // If data contains folders or requests (not environments)
        if !is_environment_file {
            root_items.extend(data.clone());
        }
    } else if let Some(routes) = &root.routes {
        // Mock server routes
        root_items.extend(routes.clone());
    } else if let Some(mcp) = &root.mcp_request {
        root_items.push(mcp.clone());
    }

    process_v5_items(&root_items, None, &mut folders, &mut requests);

    // 2. Process environments
    let environments = extract_environments_from_v5(&root, &root_val);

    let mut collections = Vec::new();
    if !is_environment_file || !requests.is_empty() || !folders.is_empty() {
        collections.push(ImportedCollection {
            name: collection_name,
            folders,
            requests,
            environments: environments.clone(),
        });
    }

    Ok(ImportData {
        collections,
        environments,
        warnings: Vec::new(),
    })
}

fn process_v5_items(
    items: &[Value],
    parent_folder_id: Option<&str>,
    folders: &mut Vec<ImportedFolder>,
    requests: &mut Vec<RequestItem>,
) {
    for raw_item in items {
        let norm_val = normalize_v5_item_value(raw_item);
        let Ok(item) = serde_json::from_value::<InsomniaV5Item>(norm_val.clone()) else {
            continue;
        };

        if is_folder(&item) {
            let folder_id = item
                .meta
                .as_ref()
                .and_then(|m| m.id.clone())
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| Uuid::new_v4().to_string());

            let folder_name = item
                .name
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| format!("Folder {}", folders.len() + 1));

            folders.push(ImportedFolder {
                id: folder_id.clone(),
                parent_id: parent_folder_id.map(String::from),
                name: folder_name,
                sort_order: folders.len() as i64,
            });

            if let Some(children) = &item.children {
                process_v5_items(children, Some(&folder_id), folders, requests);
            } else if let Some(reqs) = &item.requests {
                process_v5_items(reqs, Some(&folder_id), folders, requests);
            }
        } else {
            let req_item =
                convert_v5_item_to_request(&item, parent_folder_id, requests.len() as i64);
            requests.push(req_item);
        }
    }
}

/// Normalizes named map formats like `{ "Request Name": { "method": "POST", ... } }`
/// into standard `{ "name": "Request Name", "method": "POST", ... }`.
fn normalize_v5_item_value(val: &Value) -> Value {
    let Some(obj) = val.as_object() else {
        return val.clone();
    };

    if obj.len() == 1 {
        let (key, inner) = obj.iter().next().unwrap();
        if let Some(inner_obj) = inner.as_object() {
            let known_properties = [
                "name",
                "meta",
                "method",
                "url",
                "children",
                "requests",
                "body",
                "headers",
                "parameters",
                "authentication",
                "scripts",
                "settings",
                "metadata",
                "protoMethodName",
                "protoFileId",
                "reflectionApi",
                "eventListeners",
                "statusCode",
                "mimeType",
                "transportType",
                "queryparams",
                "pathParameters",
            ];
            if !known_properties.contains(&key.as_str()) {
                let mut merged = inner_obj.clone();
                if !merged.contains_key("name") {
                    merged.insert("name".to_string(), Value::String(key.clone()));
                }
                return Value::Object(merged);
            }
        }
    }

    val.clone()
}

fn is_folder(item: &InsomniaV5Item) -> bool {
    if item.children.is_some() || item.requests.is_some() {
        return true;
    }
    // If it has no URL, no HTTP method, and no gRPC / event / mock properties, it is treated as a folder
    item.method.is_none()
        && item.url.is_none()
        && item.proto_method_name.is_none()
        && item.proto_file_id.is_none()
        && item.reflection_api.is_none()
        && item.event_listeners.is_none()
        && item.status_code.is_none()
        && item.transport_type.is_none()
}

fn convert_v5_item_to_request(
    item: &InsomniaV5Item,
    folder_id: Option<&str>,
    sort_order: i64,
) -> RequestItem {
    let name = item
        .name
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Untitled".to_string());
    let url = item.url.clone().unwrap_or_default();

    // Check if gRPC
    let is_grpc = item.proto_method_name.is_some()
        || item.proto_file_id.is_some()
        || item.reflection_api.is_some()
        || (item.metadata.is_some() && item.method.is_none());

    if is_grpc {
        let (service, method) = parse_grpc_service_and_method(item.proto_method_name.as_deref());
        let metadata = parse_v5_key_value_rows(item.metadata.as_ref());
        let auth = parse_v5_auth(item.authentication.as_ref());
        let message = match &item.body {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Object(obj)) => obj
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            _ => String::new(),
        };
        let use_reflection = item
            .reflection_api
            .as_ref()
            .and_then(|r| r.enabled)
            .unwrap_or(true);

        return RequestItem::Grpc(GrpcRequest {
            id: Uuid::new_v4().to_string(),
            collection_id: String::new(),
            folder_id: folder_id.map(String::from),
            name,
            sort_order,
            url,
            service,
            method,
            method_type: GrpcMethodType::Unary,
            metadata,
            auth,
            message,
            use_reflection,
            proto_file_ids: item.proto_file_id.clone().into_iter().collect(),
        });
    }

    // Check if WebSocket / SocketIO
    let id_str = item
        .meta
        .as_ref()
        .and_then(|m| m.id.as_deref())
        .unwrap_or("");
    let is_ws = id_str.starts_with("ws-req")
        || id_str.starts_with("socketio-req")
        || url.starts_with("ws://")
        || url.starts_with("wss://")
        || item.event_listeners.is_some()
        || item.method.as_deref() == Some("WS");

    let method = if is_ws {
        HttpMethod::Ws
    } else {
        let method_str = item.method.as_deref().unwrap_or("GET");
        HttpMethod::from_str(method_str).unwrap_or(HttpMethod::Get)
    };

    let params = parse_v5_parameters(item.parameters.as_ref(), item.query_params.as_ref());
    let headers = parse_v5_key_value_rows(item.headers.as_ref());
    let auth = parse_v5_auth(item.authentication.as_ref());
    let body = parse_v5_body(item.body.as_ref(), item.mime_type.as_deref());

    RequestItem::Http(ApiRequest {
        id: Uuid::new_v4().to_string(),
        collection_id: String::new(),
        folder_id: folder_id.map(String::from),
        sort_order,
        name,
        method,
        url,
        params,
        headers,
        cookies: Vec::new(),
        auth,
        body,
        pre_request_script: None,
        post_request_script: None,
    })
}

fn parse_grpc_service_and_method(proto_method_name: Option<&str>) -> (String, String) {
    let Some(full) = proto_method_name else {
        return (String::new(), String::new());
    };

    let trimmed = full.trim().trim_start_matches('/');
    if let Some((svc, meth)) = trimmed.split_once('/') {
        (svc.to_string(), meth.to_string())
    } else if let Some((svc, meth)) = trimmed.rsplit_once('.') {
        (svc.to_string(), meth.to_string())
    } else {
        (String::new(), trimmed.to_string())
    }
}

fn parse_v5_key_value_rows(rows_opt: Option<&Vec<Value>>) -> Vec<KeyValueRow> {
    let mut rows = Vec::new();
    let Some(items) = rows_opt else { return rows };

    for item in items {
        if let Some(obj) = item.as_object() {
            if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                let value = match obj.get("value") {
                    Some(Value::String(s)) => s.clone(),
                    Some(Value::Number(n)) => n.to_string(),
                    Some(Value::Bool(b)) => b.to_string(),
                    _ => String::new(),
                };
                let disabled = obj
                    .get("disabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                rows.push(KeyValueRow {
                    id: Uuid::new_v4().to_string(),
                    key: name.to_string(),
                    value,
                    enabled: !disabled,
                });
            } else if obj.len() == 1 {
                let (key, val) = obj.iter().next().unwrap();
                let (value, disabled) = match val {
                    Value::String(s) => (s.clone(), false),
                    Value::Number(n) => (n.to_string(), false),
                    Value::Bool(b) => (b.to_string(), false),
                    Value::Object(inner) => {
                        let v = match inner.get("value") {
                            Some(Value::String(s)) => s.clone(),
                            Some(Value::Number(n)) => n.to_string(),
                            Some(Value::Bool(b)) => b.to_string(),
                            _ => String::new(),
                        };
                        let d = inner
                            .get("disabled")
                            .and_then(|x| x.as_bool())
                            .unwrap_or(false);
                        (v, d)
                    }
                    _ => (val.to_string(), false),
                };
                rows.push(KeyValueRow {
                    id: Uuid::new_v4().to_string(),
                    key: key.clone(),
                    value,
                    enabled: !disabled,
                });
            }
        }
    }

    rows
}

fn parse_v5_parameters(
    params_opt: Option<&Vec<Value>>,
    query_params_opt: Option<&Value>,
) -> Vec<KeyValueRow> {
    let mut rows = parse_v5_key_value_rows(params_opt);

    if let Some(qp) = query_params_opt {
        if let Some(arr) = qp.as_array() {
            let qp_rows = parse_v5_key_value_rows(Some(arr));
            rows.extend(qp_rows);
        }
    }

    rows
}

fn parse_v5_body(body_opt: Option<&Value>, default_mime: Option<&str>) -> RequestBody {
    let mut req_body = RequestBody::default();
    let Some(body_val) = body_opt else {
        return req_body;
    };

    if let Some(text) = body_val.as_str() {
        if !text.is_empty() {
            let is_json = default_mime == Some("application/json")
                || serde_json::from_str::<Value>(text).is_ok();
            req_body.mode = Some(if is_json {
                BodyMode::Json
            } else {
                BodyMode::Raw
            });
            req_body.raw = Some(text.to_string());
        }
        return req_body;
    }

    if let Some(obj) = body_val.as_object() {
        let mime = obj
            .get("mimeType")
            .and_then(|v| v.as_str())
            .or(default_mime)
            .unwrap_or("");
        let text = obj.get("text").and_then(|v| v.as_str());

        if mime == "application/json" {
            req_body.mode = Some(BodyMode::Json);
            req_body.raw = text.map(String::from);
        } else if mime == "application/x-www-form-urlencoded" {
            req_body.mode = Some(BodyMode::Urlencoded);
            if let Some(params_arr) = obj.get("params").and_then(|v| v.as_array()) {
                req_body.url_encoded = Some(parse_v5_key_value_rows(Some(params_arr)));
            } else if let Some(t) = text {
                req_body.raw = Some(t.to_string());
            }
        } else if mime == "multipart/form-data" {
            req_body.mode = Some(BodyMode::FormData);
            if let Some(params_arr) = obj.get("params").and_then(|v| v.as_array()) {
                req_body.form_data = Some(parse_v5_key_value_rows(Some(params_arr)));
            }
        } else if let Some(t) = text {
            if !t.is_empty() {
                let is_json = serde_json::from_str::<Value>(t).is_ok();
                req_body.mode = Some(if is_json {
                    BodyMode::Json
                } else {
                    BodyMode::Raw
                });
                req_body.raw = Some(t.to_string());
            }
        }
    }

    req_body
}

fn parse_v5_auth(auth_val_opt: Option<&Value>) -> AuthConfig {
    let mut config = AuthConfig::default();
    let Some(val) = auth_val_opt else {
        return config;
    };
    let Some(obj) = val.as_object() else {
        return config;
    };

    if obj
        .get("disabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return config;
    }

    let auth_type_str = obj.get("type").and_then(|v| v.as_str());

    match auth_type_str {
        Some("basic") => {
            config.auth_type = AuthType::Basic;
            config.basic = Some(BasicAuth {
                username: obj
                    .get("username")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                password: obj
                    .get("password")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            });
            return config;
        }
        Some("bearer") => {
            config.auth_type = AuthType::Bearer;
            config.bearer = Some(BearerAuth {
                token: obj
                    .get("token")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            });
            return config;
        }
        Some("apikey") => {
            config.auth_type = AuthType::ApiKey;
            let add_to = match obj.get("addTo").and_then(|v| v.as_str()) {
                Some("query") => ApiKeyTarget::Query,
                _ => ApiKeyTarget::Header,
            };
            config.api_key = Some(ApiKeyAuth {
                key: obj
                    .get("key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                value: obj
                    .get("value")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                add_to,
            });
            return config;
        }
        Some("oauth2") => {
            let token = obj
                .get("accessToken")
                .or_else(|| obj.get("token"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !token.is_empty() {
                config.auth_type = AuthType::Bearer;
                config.bearer = Some(BearerAuth {
                    token: token.to_string(),
                });
                return config;
            }
        }
        Some("singleToken") => {
            let token = obj.get("token").and_then(|v| v.as_str()).unwrap_or("");
            if !token.is_empty() {
                config.auth_type = AuthType::Bearer;
                config.bearer = Some(BearerAuth {
                    token: token.to_string(),
                });
                return config;
            }
        }
        Some("none") => return config,
        _ => {}
    }

    // Check named auth forms: { "basic": { ... } }, { "bearer": { ... } }, etc.
    if let Some(basic_obj) = obj.get("basic").and_then(|v| v.as_object()) {
        config.auth_type = AuthType::Basic;
        config.basic = Some(BasicAuth {
            username: basic_obj
                .get("username")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            password: basic_obj
                .get("password")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        });
    } else if let Some(bearer_obj) = obj.get("bearer").and_then(|v| v.as_object()) {
        config.auth_type = AuthType::Bearer;
        config.bearer = Some(BearerAuth {
            token: bearer_obj
                .get("token")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        });
    } else if let Some(api_key_obj) = obj.get("apikey").and_then(|v| v.as_object()) {
        config.auth_type = AuthType::ApiKey;
        let add_to = match api_key_obj.get("addTo").and_then(|v| v.as_str()) {
            Some("query") => ApiKeyTarget::Query,
            _ => ApiKeyTarget::Header,
        };
        config.api_key = Some(ApiKeyAuth {
            key: api_key_obj
                .get("key")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            value: api_key_obj
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            add_to,
        });
    }

    config
}

fn extract_environments_from_v5(
    root: &InsomniaV5File,
    root_val: &Value,
) -> Vec<ImportedEnvironment> {
    let mut envs = Vec::new();

    // Check root.environments
    if let Some(env_val) = &root.environments {
        extract_envs_from_value(env_val, &mut envs);
    }

    // If root.type == "environment.insomnia.rest/5.0" and environments was empty, check root.data
    if envs.is_empty() {
        if let Some(data_val) = root_val.get("data") {
            extract_envs_from_value(data_val, &mut envs);
        }
    }

    envs
}

fn extract_envs_from_value(val: &Value, envs: &mut Vec<ImportedEnvironment>) {
    match val {
        Value::Object(obj) => {
            // EnvironmentSchema object with data / value and subEnvironments
            let name = obj
                .get("name")
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
                .unwrap_or("Base Environment");

            let data_opt = obj.get("data").or_else(|| obj.get("value"));
            if let Some(data) = data_opt.and_then(|v| v.as_object()) {
                let vars = extract_env_vars_from_map(data);
                if !vars.is_empty() {
                    envs.push(ImportedEnvironment {
                        name: name.to_string(),
                        variables: vars,
                    });
                }
            }

            if let Some(sub_envs) = obj.get("subEnvironments").and_then(|v| v.as_array()) {
                for sub in sub_envs {
                    if let Some(sub_obj) = sub.as_object() {
                        let sub_name = sub_obj
                            .get("name")
                            .and_then(|v| v.as_str())
                            .filter(|s| !s.trim().is_empty())
                            .unwrap_or("Sub Environment");

                        let sub_data = sub_obj.get("data").or_else(|| sub_obj.get("value"));
                        if let Some(data_map) = sub_data.and_then(|v| v.as_object()) {
                            let vars = extract_env_vars_from_map(data_map);
                            envs.push(ImportedEnvironment {
                                name: sub_name.to_string(),
                                variables: vars,
                            });
                        }
                    }
                }
            }
        }
        Value::Array(arr) => {
            // Array of environments, e.g. [ { "Base": { "value": { ... } } }, { "name": "Dev", "data": { ... } } ]
            for item in arr {
                if let Some(item_obj) = item.as_object() {
                    if let Some(name) = item_obj.get("name").and_then(|v| v.as_str()) {
                        let data_opt = item_obj.get("data").or_else(|| item_obj.get("value"));
                        if let Some(data_map) = data_opt.and_then(|v| v.as_object()) {
                            let vars = extract_env_vars_from_map(data_map);
                            envs.push(ImportedEnvironment {
                                name: name.to_string(),
                                variables: vars,
                            });
                        }
                    } else if item_obj.len() == 1 {
                        let (name, inner) = item_obj.iter().next().unwrap();
                        let inner_map = if let Some(inner_obj) = inner.as_object() {
                            inner_obj
                                .get("value")
                                .or_else(|| inner_obj.get("data"))
                                .and_then(|v| v.as_object())
                                .unwrap_or(inner_obj)
                        } else {
                            continue;
                        };
                        let vars = extract_env_vars_from_map(inner_map);
                        envs.push(ImportedEnvironment {
                            name: name.clone(),
                            variables: vars,
                        });
                    }
                }
            }
        }
        _ => {}
    }
}

fn extract_env_vars_from_map(map: &serde_json::Map<String, Value>) -> Vec<EnvironmentVariable> {
    let env_id = Uuid::new_v4().to_string();
    map.iter()
        .map(|(key, val)| {
            let val_str = match val {
                Value::String(s) => s.clone(),
                Value::Null => String::new(),
                Value::Bool(b) => b.to_string(),
                Value::Number(n) => n.to_string(),
                other => other.to_string(),
            };
            EnvironmentVariable {
                id: Uuid::new_v4().to_string(),
                environmentid: env_id.clone(),
                key: key.clone(),
                value: val_str,
                enabled: true,
                is_secret: false,
            }
        })
        .collect()
}
