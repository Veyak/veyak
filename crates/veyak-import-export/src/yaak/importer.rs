use std::str::FromStr;
use uuid::Uuid;
use veyak_error::{AppError, AppResult};
use veyak_models::{
    ApiKeyAuth, ApiKeyTarget, ApiRequest, AuthConfig, AuthType, BasicAuth, BearerAuth, BodyMode,
    EnvironmentVariable, GrpcMethodType, GrpcRequest, HttpMethod, KeyValueRow, RequestBody,
    RequestItem,
};

use crate::types::{ImportData, ImportedCollection, ImportedEnvironment, ImportedFolder};
use crate::yaak::models::{
    YaakBodyUnion, YaakEnvironment, YaakExport, YaakGrpcRequest, YaakHttpRequest, YaakResourceItem,
    YaakResourceObject, YaakResourcesUnion,
};

pub fn import_yaak(content: &str) -> AppResult<ImportData> {
    let export: YaakExport = serde_json::from_str(content).map_err(|e| {
        AppError::Invalid(format!("Failed to parse content as Yaak JSON export: {e}"))
    })?;

    match export.resources {
        YaakResourcesUnion::Object(obj) => import_yaak_object(obj),
        YaakResourcesUnion::List(list) => import_yaak_list(list),
    }
}

fn import_yaak_object(obj: YaakResourceObject) -> AppResult<ImportData> {
    let collection_name = obj
        .workspaces
        .first()
        .map(|w| w.name.clone())
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| "Imported Yaak Workspace".to_string());

    let mut folders = Vec::new();
    let mut sorted_folders = obj.folders;
    sorted_folders.sort_by(|a, b| {
        a.sort_priority
            .unwrap_or(0.0)
            .partial_cmp(&b.sort_priority.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (idx, f) in sorted_folders.into_iter().enumerate() {
        folders.push(ImportedFolder {
            id: f.id,
            parent_id: f.folder_id,
            name: f.name,
            sort_order: idx as i64,
        });
    }

    let mut requests = Vec::new();
    let mut sorted_http = obj.http_requests;
    sorted_http.sort_by(|a, b| {
        a.sort_priority
            .unwrap_or(0.0)
            .partial_cmp(&b.sort_priority.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (idx, req) in sorted_http.into_iter().enumerate() {
        requests.push(convert_yaak_http_request(req, idx as i64));
    }

    let mut sorted_grpc = obj.grpc_requests;
    sorted_grpc.sort_by(|a, b| {
        a.sort_priority
            .unwrap_or(0.0)
            .partial_cmp(&b.sort_priority.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (idx, req) in sorted_grpc.into_iter().enumerate() {
        requests.push(convert_yaak_grpc_request(
            req,
            (requests.len() + idx) as i64,
        ));
    }

    let environments = obj
        .environments
        .into_iter()
        .map(convert_yaak_environment)
        .collect();

    Ok(ImportData {
        collections: vec![ImportedCollection {
            name: collection_name,
            folders,
            requests,
        }],
        environments,
        warnings: Vec::new(),
    })
}

fn import_yaak_list(list: Vec<YaakResourceItem>) -> AppResult<ImportData> {
    let mut collection_name = "Imported Yaak Workspace".to_string();
    let mut folders = Vec::new();
    let mut requests = Vec::new();
    let mut environments = Vec::new();

    let mut folder_items = Vec::new();
    let mut request_items = Vec::new();

    for item in list {
        match item.model.as_deref() {
            Some("workspace") => {
                if let Some(name) = item.name {
                    if !name.trim().is_empty() {
                        collection_name = name;
                    }
                }
            }
            Some("folder") => {
                folder_items.push(item);
            }
            Some("http_request") | None if item.url.is_some() => {
                request_items.push(item);
            }
            Some("grpc_request") => {
                let grpc_req = YaakGrpcRequest {
                    id: item.id,
                    name: item.name.unwrap_or_else(|| "Untitled gRPC".to_string()),
                    workspace_id: item.workspace_id,
                    folder_id: item.folder_id,
                    url: item.url,
                    service: item.service,
                    method: item.method,
                    message: item.message,
                    metadata: item.metadata,
                    sort_priority: None,
                };
                requests.push(convert_yaak_grpc_request(grpc_req, requests.len() as i64));
            }
            Some("environment") => {
                let env_id = Uuid::new_v4().to_string();
                let variables = item
                    .variables
                    .unwrap_or_default()
                    .into_iter()
                    .map(|v| {
                        let value_str = match v.value {
                            serde_json::Value::String(s) => s,
                            serde_json::Value::Null => String::new(),
                            other => other.to_string(),
                        };
                        EnvironmentVariable {
                            id: Uuid::new_v4().to_string(),
                            environmentid: env_id.clone(),
                            key: v.name,
                            value: value_str,
                            enabled: v.enabled.unwrap_or(true),
                            is_secret: false,
                        }
                    })
                    .collect();

                environments.push(ImportedEnvironment {
                    name: item.name.unwrap_or_else(|| "Yaak Environment".to_string()),
                    variables,
                });
            }
            _ => {}
        }
    }

    for (idx, f) in folder_items.into_iter().enumerate() {
        folders.push(ImportedFolder {
            id: f.id,
            parent_id: f.folder_id,
            name: f.name.unwrap_or_else(|| format!("Folder {}", idx + 1)),
            sort_order: idx as i64,
        });
    }

    for (idx, r) in request_items.into_iter().enumerate() {
        let http_req = YaakHttpRequest {
            id: r.id,
            name: r.name.unwrap_or_else(|| format!("Request {}", idx + 1)),
            workspace_id: r.workspace_id,
            folder_id: r.folder_id,
            url: r.url,
            method: r.method,
            headers: r.headers,
            url_parameters: r.url_parameters,
            body: r.body,
            body_type: r.body_type,
            authentication: r.authentication,
            authentication_type: r.authentication_type,
            sort_priority: None,
        };
        requests.push(convert_yaak_http_request(http_req, idx as i64));
    }

    Ok(ImportData {
        collections: vec![ImportedCollection {
            name: collection_name,
            folders,
            requests,
        }],
        environments,
        warnings: Vec::new(),
    })
}

fn convert_yaak_http_request(req: YaakHttpRequest, sort_order: i64) -> RequestItem {
    let url = req.url.unwrap_or_default();
    let method_str = req.method.as_deref().unwrap_or("GET");
    let is_ws = method_str.eq_ignore_ascii_case("ws")
        || url.starts_with("ws://")
        || url.starts_with("wss://");

    let method = if is_ws {
        HttpMethod::Ws
    } else {
        HttpMethod::from_str(method_str).unwrap_or(HttpMethod::Get)
    };

    let params = req
        .url_parameters
        .unwrap_or_default()
        .into_iter()
        .map(|p| KeyValueRow {
            id: Uuid::new_v4().to_string(),
            key: p.name,
            value: p.value,
            enabled: p.enabled.unwrap_or(true),
        })
        .collect();

    let headers = req
        .headers
        .unwrap_or_default()
        .into_iter()
        .map(|h| KeyValueRow {
            id: Uuid::new_v4().to_string(),
            key: h.name,
            value: h.value,
            enabled: h.enabled.unwrap_or(true),
        })
        .collect();

    let auth = convert_yaak_auth(
        req.authentication.as_ref(),
        req.authentication_type.as_deref(),
    );
    let body = convert_yaak_body(req.body.as_ref(), req.body_type.as_deref());

    RequestItem::Http(ApiRequest {
        id: Uuid::new_v4().to_string(),
        collection_id: String::new(),
        folder_id: req.folder_id,
        sort_order,
        name: req.name,
        method,
        url,
        params,
        headers,
        cookies: Vec::new(),
        auth,
        body,
    })
}

fn convert_yaak_grpc_request(req: YaakGrpcRequest, sort_order: i64) -> RequestItem {
    let metadata = req
        .metadata
        .unwrap_or_default()
        .into_iter()
        .map(|h| KeyValueRow {
            id: Uuid::new_v4().to_string(),
            key: h.name,
            value: h.value,
            enabled: h.enabled.unwrap_or(true),
        })
        .collect();

    RequestItem::Grpc(GrpcRequest {
        id: Uuid::new_v4().to_string(),
        collection_id: String::new(),
        folder_id: req.folder_id,
        name: req.name,
        sort_order,
        url: req.url.unwrap_or_default(),
        service: req.service.unwrap_or_default(),
        method: req.method.unwrap_or_default(),
        method_type: GrpcMethodType::Unary,
        metadata,
        auth: AuthConfig::default(),
        message: req.message.unwrap_or_default(),
        use_reflection: true,
        proto_file_ids: Vec::new(),
    })
}

fn convert_yaak_environment(env: YaakEnvironment) -> ImportedEnvironment {
    let env_id = Uuid::new_v4().to_string();
    let variables = env
        .variables
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
                key: v.name,
                value: val,
                enabled: v.enabled.unwrap_or(true),
                is_secret: false,
            }
        })
        .collect();

    ImportedEnvironment {
        name: env.name,
        variables,
    }
}

fn convert_yaak_auth(auth_val: Option<&serde_json::Value>, auth_type: Option<&str>) -> AuthConfig {
    let mut config = AuthConfig::default();
    let val = match auth_val {
        Some(v) if v.is_object() => v,
        _ => return config,
    };

    let resolved_type = auth_type
        .or_else(|| val.get("type").and_then(|t| t.as_str()))
        .unwrap_or("");

    match resolved_type.to_lowercase().as_str() {
        "bearer" => {
            config.auth_type = AuthType::Bearer;
            let token = val
                .get("token")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            config.bearer = Some(BearerAuth { token });
        }
        "basic" => {
            config.auth_type = AuthType::Basic;
            let username = val
                .get("username")
                .and_then(|u| u.as_str())
                .unwrap_or("")
                .to_string();
            let password = val
                .get("password")
                .and_then(|p| p.as_str())
                .unwrap_or("")
                .to_string();
            config.basic = Some(BasicAuth { username, password });
        }
        "apikey" | "api_key" => {
            config.auth_type = AuthType::ApiKey;
            let key = val
                .get("key")
                .and_then(|k| k.as_str())
                .unwrap_or("")
                .to_string();
            let value = val
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let add_to = match val
                .get("addTo")
                .or_else(|| val.get("in"))
                .and_then(|a| a.as_str())
            {
                Some("query") => ApiKeyTarget::Query,
                _ => ApiKeyTarget::Header,
            };
            config.api_key = Some(ApiKeyAuth { key, value, add_to });
        }
        _ => {}
    }

    config
}

fn convert_yaak_body(body_val: Option<&YaakBodyUnion>, body_type: Option<&str>) -> RequestBody {
    let mut req_body = RequestBody::default();
    let body = match body_val {
        Some(b) => b,
        None => return req_body,
    };

    match body {
        YaakBodyUnion::String(s) => {
            let is_json = body_type == Some("application/json")
                || serde_json::from_str::<serde_json::Value>(s).is_ok();
            req_body.mode = Some(if is_json {
                BodyMode::Json
            } else {
                BodyMode::Raw
            });
            req_body.raw = Some(s.clone());
        }
        YaakBodyUnion::Object(obj) => {
            if let Some(form) = &obj.form {
                req_body.mode = Some(BodyMode::FormData);
                req_body.form_data = Some(
                    form.iter()
                        .map(|pair| KeyValueRow {
                            id: Uuid::new_v4().to_string(),
                            key: pair.name.clone(),
                            value: pair.value.clone(),
                            enabled: pair.enabled.unwrap_or(true),
                        })
                        .collect(),
                );
            } else if let Some(text) = &obj.text {
                let is_json = body_type == Some("application/json")
                    || serde_json::from_str::<serde_json::Value>(text).is_ok();
                req_body.mode = Some(if is_json {
                    BodyMode::Json
                } else {
                    BodyMode::Raw
                });
                req_body.raw = Some(text.clone());
            }
        }
    }

    req_body
}
