use std::collections::HashSet;
use std::str::FromStr;
use uuid::Uuid;
use veyak_error::{AppError, AppResult};
use veyak_models::{
    ApiKeyAuth, ApiKeyTarget, ApiRequest, AuthConfig, AuthType, BasicAuth, BearerAuth, BodyMode,
    EnvironmentVariable, GrpcMethodType, GrpcRequest, HttpMethod, KeyValueRow, RequestBody,
    RequestItem,
};

use crate::insomnia::models::{InsomniaAuth, InsomniaBody, InsomniaExport, InsomniaResource};
use crate::insomnia::v5_importer::{import_insomnia_v5, is_insomnia_v5};
use crate::types::{ImportData, ImportedCollection, ImportedEnvironment, ImportedFolder};

pub fn import_insomnia(content: &str) -> AppResult<ImportData> {
    if is_insomnia_v5(content) {
        match import_insomnia_v5(content) {
            Ok(data) => return Ok(data),
            Err(e) => {
                if let Ok(data) = import_insomnia_v4(content) {
                    return Ok(data);
                }
                return Err(e);
            }
        }
    }

    match import_insomnia_v4(content) {
        Ok(data) => Ok(data),
        Err(e) => {
            if let Ok(data) = import_insomnia_v5(content) {
                return Ok(data);
            }
            Err(e)
        }
    }
}

pub fn import_insomnia_v4(content: &str) -> AppResult<ImportData> {
    let export: InsomniaExport = if let Ok(json) = serde_json::from_str::<InsomniaExport>(content) {
        json
    } else if let Ok(yaml) = serde_yaml::from_str::<InsomniaExport>(content) {
        yaml
    } else {
        return Err(AppError::Invalid(
            "Failed to parse content as Insomnia JSON or YAML export".to_string(),
        ));
    };

    let mut collection_name = "Imported Insomnia Collection".to_string();
    let mut workspace_ids = HashSet::new();

    // 1. Identify workspace names and IDs
    for res in &export.resources {
        if res.resource_type == "workspace" {
            workspace_ids.insert(res.id.clone());
            if let Some(name) = &res.name {
                if !name.trim().is_empty() {
                    collection_name = name.clone();
                }
            }
        }
    }

    // 2. Identify folders (request_group)
    let mut folders = Vec::new();
    let mut folder_resources: Vec<&InsomniaResource> = export
        .resources
        .iter()
        .filter(|r| r.resource_type == "request_group")
        .collect();

    folder_resources.sort_by(|a, b| {
        a.meta_sort_key
            .unwrap_or(0.0)
            .partial_cmp(&b.meta_sort_key.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (idx, res) in folder_resources.iter().enumerate() {
        let parent_id = match &res.parent_id {
            Some(pid) if workspace_ids.contains(pid) => None,
            Some(pid) => Some(pid.clone()),
            None => None,
        };

        folders.push(ImportedFolder {
            id: res.id.clone(),
            parent_id,
            name: res
                .name
                .clone()
                .unwrap_or_else(|| format!("Folder {}", idx + 1)),
            sort_order: idx as i64,
        });
    }

    // 3. Identify requests (request, websocket_request, grpc_request)
    let mut requests = Vec::new();
    let mut request_resources: Vec<&InsomniaResource> = export
        .resources
        .iter()
        .filter(|r| {
            r.resource_type == "request"
                || r.resource_type == "websocket_request"
                || r.resource_type == "grpc_request"
        })
        .collect();

    request_resources.sort_by(|a, b| {
        a.meta_sort_key
            .unwrap_or(0.0)
            .partial_cmp(&b.meta_sort_key.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (idx, res) in request_resources.iter().enumerate() {
        let folder_id = match &res.parent_id {
            Some(pid) if workspace_ids.contains(pid) => None,
            Some(pid) => Some(pid.clone()),
            None => None,
        };

        let req_item = convert_insomnia_resource_to_request(res, folder_id.as_deref(), idx as i64);
        requests.push(req_item);
    }

    // 4. Identify environments
    let mut environments = Vec::new();
    for res in &export.resources {
        if res.resource_type == "environment" {
            if let Some(data) = &res.data {
                let env_id = Uuid::new_v4().to_string();
                let variables = data
                    .iter()
                    .map(|(key, val)| {
                        let value_str = match val {
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Null => String::new(),
                            other => other.to_string(),
                        };
                        EnvironmentVariable {
                            id: Uuid::new_v4().to_string(),
                            environmentid: env_id.clone(),
                            key: key.clone(),
                            value: value_str,
                            enabled: true,
                            is_secret: false,
                        }
                    })
                    .collect();

                environments.push(ImportedEnvironment {
                    name: res
                        .name
                        .clone()
                        .unwrap_or_else(|| "Insomnia Environment".to_string()),
                    variables,
                });
            }
        }
    }

    Ok(ImportData {
        collections: vec![ImportedCollection {
            name: collection_name,
            folders,
            requests,
            environments: environments.clone(),
        }],
        environments,
        warnings: Vec::new(),
    })
}

fn convert_insomnia_resource_to_request(
    res: &InsomniaResource,
    folder_id: Option<&str>,
    sort_order: i64,
) -> RequestItem {
    let name = res.name.clone().unwrap_or_else(|| "Untitled".to_string());
    let url = res.url.clone().unwrap_or_default();

    if res.resource_type == "grpc_request" {
        return RequestItem::Grpc(GrpcRequest {
            id: Uuid::new_v4().to_string(),
            collection_id: String::new(),
            folder_id: folder_id.map(String::from),
            name,
            sort_order,
            url,
            service: String::new(),
            method: String::new(),
            method_type: GrpcMethodType::Unary,
            metadata: Vec::new(),
            auth: AuthConfig::default(),
            message: res
                .body
                .as_ref()
                .and_then(|b| b.text.clone())
                .unwrap_or_default(),
            use_reflection: true,
            proto_file_ids: Vec::new(),
        });
    }

    let is_ws = res.resource_type == "websocket_request"
        || url.starts_with("ws://")
        || url.starts_with("wss://");

    let method = if is_ws {
        HttpMethod::Ws
    } else {
        let method_str = res.method.as_deref().unwrap_or("GET");
        HttpMethod::from_str(method_str).unwrap_or(HttpMethod::Get)
    };

    let params: Vec<KeyValueRow> = res
        .parameters
        .as_ref()
        .map(|params| {
            params
                .iter()
                .map(|p| KeyValueRow {
                    id: Uuid::new_v4().to_string(),
                    key: p.name.clone(),
                    value: p.value.clone(),
                    enabled: !p.disabled.unwrap_or(false),
                })
                .collect()
        })
        .unwrap_or_default();

    let headers: Vec<KeyValueRow> = res
        .headers
        .as_ref()
        .map(|headers| {
            headers
                .iter()
                .map(|h| KeyValueRow {
                    id: Uuid::new_v4().to_string(),
                    key: h.name.clone(),
                    value: h.value.clone(),
                    enabled: !h.disabled.unwrap_or(false),
                })
                .collect()
        })
        .unwrap_or_default();

    let auth = convert_insomnia_auth(res.authentication.as_ref());
    let body = convert_insomnia_body(res.body.as_ref());

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
    })
}

fn convert_insomnia_auth(auth_opt: Option<&InsomniaAuth>) -> AuthConfig {
    let mut config = AuthConfig::default();
    let auth = match auth_opt {
        Some(a) if !a.disabled.unwrap_or(false) => a,
        _ => return config,
    };

    match auth.auth_type.as_deref() {
        Some("bearer") => {
            config.auth_type = AuthType::Bearer;
            config.bearer = Some(BearerAuth {
                token: auth.token.clone().unwrap_or_default(),
            });
        }
        Some("basic") => {
            config.auth_type = AuthType::Basic;
            config.basic = Some(BasicAuth {
                username: auth.username.clone().unwrap_or_default(),
                password: auth.password.clone().unwrap_or_default(),
            });
        }
        Some("apikey") => {
            config.auth_type = AuthType::ApiKey;
            let add_to = match auth.add_to.as_deref() {
                Some("query") => ApiKeyTarget::Query,
                _ => ApiKeyTarget::Header,
            };
            config.api_key = Some(ApiKeyAuth {
                key: auth.key.clone().unwrap_or_default(),
                value: auth.value.clone().unwrap_or_default(),
                add_to,
            });
        }
        _ => {}
    }

    config
}

fn convert_insomnia_body(body_opt: Option<&InsomniaBody>) -> RequestBody {
    let mut req_body = RequestBody::default();
    let body = match body_opt {
        Some(b) => b,
        None => return req_body,
    };

    let mime = body.mime_type.as_deref().unwrap_or("");
    if mime == "application/json" {
        req_body.mode = Some(BodyMode::Json);
        req_body.raw = body.text.clone();
    } else if mime == "application/x-www-form-urlencoded" {
        req_body.mode = Some(BodyMode::Urlencoded);
        if let Some(params) = &body.params {
            req_body.url_encoded = Some(
                params
                    .iter()
                    .map(|p| KeyValueRow {
                        id: Uuid::new_v4().to_string(),
                        key: p.name.clone(),
                        value: p.value.clone().unwrap_or_default(),
                        enabled: !p.disabled.unwrap_or(false),
                    })
                    .collect(),
            );
        } else if let Some(text) = &body.text {
            req_body.raw = Some(text.clone());
        }
    } else if mime == "multipart/form-data" {
        req_body.mode = Some(BodyMode::FormData);
        if let Some(params) = &body.params {
            req_body.form_data = Some(
                params
                    .iter()
                    .map(|p| KeyValueRow {
                        id: Uuid::new_v4().to_string(),
                        key: p.name.clone(),
                        value: p.value.clone().unwrap_or_default(),
                        enabled: !p.disabled.unwrap_or(false),
                    })
                    .collect(),
            );
        }
    } else if let Some(text) = &body.text {
        if !text.is_empty() {
            let is_json = serde_json::from_str::<serde_json::Value>(text).is_ok();
            req_body.mode = Some(if is_json {
                BodyMode::Json
            } else {
                BodyMode::Raw
            });
            req_body.raw = Some(text.clone());
        }
    }

    req_body
}
