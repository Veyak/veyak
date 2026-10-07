use std::str::FromStr;
use uuid::Uuid;
use veyak_error::{AppError, AppResult};
use veyak_models::{
    ApiKeyAuth, ApiKeyTarget, ApiRequest, AuthConfig, AuthType, BasicAuth, BearerAuth, BodyMode,
    EnvironmentVariable, GraphQlRequest, GraphQlRequestType, HttpMethod, KeyValueRow, RequestBody,
    RequestItem,
};

use crate::postman::models::{
    PostmanAuth, PostmanBody, PostmanCollection, PostmanEnvironment, PostmanItem,
    PostmanRequestUnion, PostmanUrlUnion,
};
use crate::types::{ImportData, ImportedCollection, ImportedEnvironment, ImportedFolder};

pub fn import_postman(content: &str) -> AppResult<ImportData> {
    // Try importing as Collection
    if let Ok(collection) = serde_json::from_str::<PostmanCollection>(content) {
        if !collection.item.is_empty()
            || collection.variable.is_some()
            || !collection.info.name.is_empty()
        {
            return import_postman_collection(collection);
        }
    }

    // Try importing as Environment
    if let Ok(env) = serde_json::from_str::<PostmanEnvironment>(content) {
        return import_postman_environment(env);
    }

    Err(AppError::Invalid(
        "Content is neither a valid Postman Collection nor a Postman Environment".to_string(),
    ))
}

pub fn import_postman_collection(collection: PostmanCollection) -> AppResult<ImportData> {
    let mut folders = Vec::new();
    let mut requests = Vec::new();
    let mut warnings = Vec::new();

    process_items(
        &collection.item,
        None,
        &mut folders,
        &mut requests,
        &mut warnings,
    );

    let mut environments = Vec::new();
    if let Some(vars) = collection.variable {
        if !vars.is_empty() {
            let env_id = Uuid::new_v4().to_string();
            let variables = vars
                .into_iter()
                .filter_map(|v| {
                    let key = v.key?;
                    let val = match v.value {
                        Some(serde_json::Value::String(s)) => s,
                        Some(val) => val.to_string(),
                        None => String::new(),
                    };
                    Some(EnvironmentVariable {
                        id: Uuid::new_v4().to_string(),
                        environmentid: env_id.clone(),
                        key,
                        value: val,
                        enabled: !v.disabled.unwrap_or(false),
                        is_secret: v.var_type.as_deref() == Some("secret"),
                    })
                })
                .collect::<Vec<_>>();

            if !variables.is_empty() {
                environments.push(ImportedEnvironment {
                    name: format!("{} Variables", collection.info.name),
                    variables,
                });
            }
        }
    }

    Ok(ImportData {
        collections: vec![ImportedCollection {
            name: if collection.info.name.trim().is_empty() {
                "Imported Postman Collection".to_string()
            } else {
                collection.info.name
            },
            folders,
            requests,
            environments: environments.clone(),
        }],
        environments,
        warnings,
    })
}

pub fn import_postman_environment(env: PostmanEnvironment) -> AppResult<ImportData> {
    let env_id = env.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let variables = env
        .values
        .into_iter()
        .map(|v| {
            let val = match v.value {
                serde_json::Value::String(s) => s,
                serde_json::Value::Null => String::new(),
                other => other.to_string(),
            };
            EnvironmentVariable {
                id: Uuid::new_v4().to_string(),
                environmentid: env_id.clone(),
                key: v.key,
                value: val,
                enabled: v.enabled.unwrap_or(true),
                is_secret: v.value_type.as_deref() == Some("secret"),
            }
        })
        .collect();

    Ok(ImportData {
        collections: Vec::new(),
        environments: vec![ImportedEnvironment {
            name: if env.name.trim().is_empty() {
                "Postman Environment".to_string()
            } else {
                env.name
            },
            variables,
        }],
        warnings: Vec::new(),
    })
}

fn process_items(
    items: &[PostmanItem],
    parent_folder_id: Option<&str>,
    folders: &mut Vec<ImportedFolder>,
    requests: &mut Vec<RequestItem>,
    warnings: &mut Vec<String>,
) {
    for (idx, item) in items.iter().enumerate() {
        if let Some(sub_items) = &item.item {
            // This is a folder
            let folder_id = item
                .id
                .clone()
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            let folder_name = item
                .name
                .clone()
                .unwrap_or_else(|| format!("Folder {}", idx + 1));

            folders.push(ImportedFolder {
                id: folder_id.clone(),
                parent_id: parent_folder_id.map(String::from),
                name: folder_name,
                sort_order: idx as i64,
            });

            process_items(sub_items, Some(&folder_id), folders, requests, warnings);
        } else if let Some(req_union) = &item.request {
            // This is a request
            let name = item
                .name
                .clone()
                .unwrap_or_else(|| format!("Request {}", idx + 1));
            let req_item = convert_postman_request(req_union, name, parent_folder_id, idx as i64);
            requests.push(req_item);
        }
    }
}

fn convert_postman_request(
    req_union: &PostmanRequestUnion,
    name: String,
    folder_id: Option<&str>,
    sort_order: i64,
) -> RequestItem {
    let req = match req_union {
        PostmanRequestUnion::String(url) => {
            return RequestItem::Http(ApiRequest {
                id: Uuid::new_v4().to_string(),
                collection_id: String::new(),
                folder_id: folder_id.map(String::from),
                sort_order,
                name,
                method: HttpMethod::Get,
                url: url.clone(),
                params: extract_query_params(url),
                headers: Vec::new(),
                cookies: Vec::new(),
                auth: AuthConfig::default(),
                body: RequestBody::default(),
                pre_request_script: None,
                post_request_script: None,
            });
        }
        PostmanRequestUnion::Object(req) => req,
    };

    // Determine URL string and query parameters
    let (url_str, mut params) = extract_url_and_params(req.url.as_ref());

    // Determine HTTP Method
    let method_str = req.method.as_deref().unwrap_or("GET");
    let method = match HttpMethod::from_str(method_str) {
        Ok(m) => m,
        Err(_) => HttpMethod::Get,
    };

    // Extract headers
    let headers: Vec<KeyValueRow> = req
        .header
        .as_ref()
        .map(|headers| {
            headers
                .iter()
                .map(|h| KeyValueRow {
                    id: Uuid::new_v4().to_string(),
                    key: h.key.clone(),
                    value: h.value.clone(),
                    enabled: !h.disabled.unwrap_or(false),
                })
                .collect()
        })
        .unwrap_or_default();

    // Extract auth
    let auth = convert_postman_auth(req.auth.as_ref());

    // Check if GraphQL
    if let Some(body) = &req.body {
        if body.mode.as_deref() == Some("graphql") {
            if let Some(gql) = &body.graphql {
                return RequestItem::GraphQL(GraphQlRequest {
                    id: Uuid::new_v4().to_string(),
                    collection_id: String::new(),
                    folder_id: folder_id.map(String::from),
                    name,
                    method: "GRAPHQL".to_string(),
                    sort_order,
                    url: url_str,
                    query: gql.query.clone().unwrap_or_default(),
                    variables: gql.variables.clone().unwrap_or_default(),
                    operation_name: None,
                    headers,
                    auth,
                    request_type: GraphQlRequestType::Query,
                });
            }
        }
    }

    // Convert request body
    let body = convert_postman_body(req.body.as_ref());

    // If query parameters weren't explicitly extracted from postman URL object,
    // extract them from url_str if empty
    if params.is_empty() && url_str.contains('?') {
        params = extract_query_params(&url_str);
    }

    // Check if WebSocket
    let is_ws =
        method == HttpMethod::Ws || url_str.starts_with("ws://") || url_str.starts_with("wss://");
    let final_method = if is_ws { HttpMethod::Ws } else { method };

    RequestItem::Http(ApiRequest {
        id: Uuid::new_v4().to_string(),
        collection_id: String::new(),
        folder_id: folder_id.map(String::from),
        sort_order,
        name,
        method: final_method,
        url: url_str,
        params,
        headers,
        cookies: Vec::new(),
        auth,
        body,
        pre_request_script: None,
        post_request_script: None,
    })
}

fn extract_url_and_params(url_opt: Option<&PostmanUrlUnion>) -> (String, Vec<KeyValueRow>) {
    let mut params = Vec::new();
    let url_str = match url_opt {
        None => String::new(),
        Some(PostmanUrlUnion::String(s)) => s.clone(),
        Some(PostmanUrlUnion::Object(obj)) => {
            if let Some(query) = &obj.query {
                for q in query {
                    if let Some(key) = &q.key {
                        params.push(KeyValueRow {
                            id: Uuid::new_v4().to_string(),
                            key: key.clone(),
                            value: q.value.clone().unwrap_or_default(),
                            enabled: !q.disabled.unwrap_or(false),
                        });
                    }
                }
            }

            if let Some(raw) = &obj.raw {
                raw.clone()
            } else {
                // Reconstruct from protocol, host, path
                let mut reconstructed = String::new();
                if let Some(protocol) = &obj.protocol {
                    reconstructed.push_str(protocol);
                    reconstructed.push_str("://");
                }
                if let Some(host) = &obj.host {
                    match host {
                        serde_json::Value::String(h) => reconstructed.push_str(h),
                        serde_json::Value::Array(arr) => {
                            let parts: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
                            reconstructed.push_str(&parts.join("."));
                        }
                        _ => {}
                    }
                }
                if let Some(path) = &obj.path {
                    match path {
                        serde_json::Value::String(p) => {
                            if !p.starts_with('/') {
                                reconstructed.push('/');
                            }
                            reconstructed.push_str(p);
                        }
                        serde_json::Value::Array(arr) => {
                            let parts: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
                            reconstructed.push('/');
                            reconstructed.push_str(&parts.join("/"));
                        }
                        _ => {}
                    }
                }
                reconstructed
            }
        }
    };
    (url_str, params)
}

fn extract_query_params(url_str: &str) -> Vec<KeyValueRow> {
    let mut params = Vec::new();
    if let Some(query_part) = url_str.split('?').nth(1) {
        for pair in query_part.split('&') {
            if pair.is_empty() {
                continue;
            }
            let mut kv = pair.splitn(2, '=');
            let key = kv.next().unwrap_or("").to_string();
            let val = kv.next().unwrap_or("").to_string();
            params.push(KeyValueRow {
                id: Uuid::new_v4().to_string(),
                key,
                value: val,
                enabled: true,
            });
        }
    }
    params
}

fn convert_postman_auth(auth_opt: Option<&PostmanAuth>) -> AuthConfig {
    let mut config = AuthConfig::default();
    let auth = match auth_opt {
        Some(a) => a,
        None => return config,
    };

    match auth.auth_type.to_lowercase().as_str() {
        "bearer" => {
            config.auth_type = AuthType::Bearer;
            let mut token = String::new();
            if let Some(params) = &auth.bearer {
                for p in params {
                    if p.key == "token" {
                        if let Some(v) = &p.value {
                            token = match v {
                                serde_json::Value::String(s) => s.clone(),
                                other => other.to_string(),
                            };
                        }
                    }
                }
            }
            config.bearer = Some(BearerAuth { token });
        }
        "basic" => {
            config.auth_type = AuthType::Basic;
            let mut username = String::new();
            let mut password = String::new();
            if let Some(params) = &auth.basic {
                for p in params {
                    if p.key == "username" {
                        if let Some(v) = &p.value {
                            username = match v {
                                serde_json::Value::String(s) => s.clone(),
                                other => other.to_string(),
                            };
                        }
                    } else if p.key == "password" {
                        if let Some(v) = &p.value {
                            password = match v {
                                serde_json::Value::String(s) => s.clone(),
                                other => other.to_string(),
                            };
                        }
                    }
                }
            }
            config.basic = Some(BasicAuth { username, password });
        }
        "apikey" => {
            config.auth_type = AuthType::ApiKey;
            let mut key = String::new();
            let mut value = String::new();
            let mut add_to = ApiKeyTarget::Header;

            if let Some(params) = &auth.apikey {
                for p in params {
                    match p.key.as_str() {
                        "key" => {
                            if let Some(v) = &p.value {
                                key = v.as_str().unwrap_or("").to_string();
                            }
                        }
                        "value" => {
                            if let Some(v) = &p.value {
                                value = v.as_str().unwrap_or("").to_string();
                            }
                        }
                        "in" | "where" => {
                            if let Some(v) = &p.value {
                                if v.as_str() == Some("query") {
                                    add_to = ApiKeyTarget::Query;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            config.api_key = Some(ApiKeyAuth { key, value, add_to });
        }
        _ => {
            config.auth_type = AuthType::None;
        }
    }

    config
}

fn convert_postman_body(body_opt: Option<&PostmanBody>) -> RequestBody {
    let mut req_body = RequestBody::default();
    let body = match body_opt {
        Some(b) => b,
        None => return req_body,
    };

    match body.mode.as_deref() {
        Some("raw") => {
            let raw_text = body.raw.clone().unwrap_or_default();
            let is_json = body
                .options
                .as_ref()
                .and_then(|o| o.get("raw"))
                .and_then(|r| r.get("language"))
                .and_then(|l| l.as_str())
                == Some("json")
                || serde_json::from_str::<serde_json::Value>(&raw_text).is_ok();

            req_body.mode = Some(if is_json {
                BodyMode::Json
            } else {
                BodyMode::Raw
            });
            req_body.raw = Some(raw_text);
        }
        Some("urlencoded") => {
            req_body.mode = Some(BodyMode::Urlencoded);
            if let Some(url_encoded) = &body.urlencoded {
                req_body.url_encoded = Some(
                    url_encoded
                        .iter()
                        .map(|p| KeyValueRow {
                            id: Uuid::new_v4().to_string(),
                            key: p.key.clone(),
                            value: p.value.clone(),
                            enabled: !p.disabled.unwrap_or(false),
                        })
                        .collect(),
                );
            }
        }
        Some("formdata") => {
            req_body.mode = Some(BodyMode::FormData);
            if let Some(formdata) = &body.formdata {
                req_body.form_data = Some(
                    formdata
                        .iter()
                        .map(|p| KeyValueRow {
                            id: Uuid::new_v4().to_string(),
                            key: p.key.clone(),
                            value: p.value.clone().unwrap_or_default(),
                            enabled: !p.disabled.unwrap_or(false),
                        })
                        .collect(),
                );
            }
        }
        _ => {}
    }

    req_body
}
