use chrono::Utc;
use serde_json::json;
use veyak_error::AppResult;
use veyak_models::{
    ApiKeyTarget, ApiRequest, AuthConfig, AuthType, BodyMode, Collection, Environment,
    EnvironmentVariable, Folder, GraphQlRequest, GrpcRequest, HttpMethod, RequestBody, RequestItem,
};

use crate::insomnia::models::{
    InsomniaAuth, InsomniaBody, InsomniaBodyParam, InsomniaExport, InsomniaHeader,
    InsomniaParameter, InsomniaResource,
};

pub fn export_collection_as_insomnia(
    collection: &Collection,
    folders: &[Folder],
    requests: &[RequestItem],
    environments: Option<&[(&Environment, &[EnvironmentVariable])]>,
) -> AppResult<String> {
    let wrk_id = format!("wrk_{}", collection.id);
    let mut resources = Vec::new();

    // 1. Workspace resource
    resources.push(InsomniaResource {
        id: wrk_id.clone(),
        resource_type: "workspace".to_string(),
        parent_id: None,
        name: Some(collection.name.clone()),
        description: None,
        meta_sort_key: Some(0.0),
        ..Default::default()
    });

    // 2. Folder resources
    for (idx, folder) in folders.iter().enumerate() {
        let parent_id = match &folder.parent_folder_id {
            Some(pid) => format!("fld_{pid}"),
            None => wrk_id.clone(),
        };

        resources.push(InsomniaResource {
            id: format!("fld_{}", folder.id),
            resource_type: "request_group".to_string(),
            parent_id: Some(parent_id),
            name: Some(folder.name.clone()),
            description: None,
            meta_sort_key: Some(idx as f64 * 100.0),
            ..Default::default()
        });
    }

    // 3. Request resources
    for (idx, req_item) in requests.iter().enumerate() {
        let parent_id = match req_item.folder_id() {
            Some(fid) => format!("fld_{fid}"),
            None => wrk_id.clone(),
        };

        let res = convert_request_item_to_insomnia(req_item, &parent_id, idx as f64 * 100.0);
        resources.push(res);
    }

    // 4. Environment resources
    if let Some(envs) = environments {
        for (env, vars) in envs {
            let mut data_map = serde_json::Map::new();
            for v in *vars {
                data_map.insert(v.key.clone(), serde_json::Value::String(v.value.clone()));
            }

            resources.push(InsomniaResource {
                id: format!("env_{}", env.id),
                resource_type: "environment".to_string(),
                parent_id: Some(wrk_id.clone()),
                name: Some(env.name.clone()),
                data: Some(data_map),
                meta_sort_key: Some(env.sort_order as f64),
                ..Default::default()
            });
        }
    }

    let export = InsomniaExport {
        export_type: "export".to_string(),
        export_format: 4,
        export_date: Some(Utc::now().to_rfc3339()),
        export_source: Some("veyak".to_string()),
        resources,
    };

    let json_str = serde_json::to_string_pretty(&export)?;
    Ok(json_str)
}

fn convert_request_item_to_insomnia(
    req_item: &RequestItem,
    parent_id: &str,
    meta_sort_key: f64,
) -> InsomniaResource {
    match req_item {
        RequestItem::Http(req) => convert_http_to_insomnia(req, parent_id, meta_sort_key),
        RequestItem::GraphQL(req) => convert_graphql_to_insomnia(req, parent_id, meta_sort_key),
        RequestItem::Grpc(req) => convert_grpc_to_insomnia(req, parent_id, meta_sort_key),
    }
}

fn convert_http_to_insomnia(
    req: &ApiRequest,
    parent_id: &str,
    meta_sort_key: f64,
) -> InsomniaResource {
    let is_ws = req.method == HttpMethod::Ws
        || req.url.starts_with("ws://")
        || req.url.starts_with("wss://");

    let resource_type = if is_ws {
        "websocket_request".to_string()
    } else {
        "request".to_string()
    };

    let headers = req
        .headers
        .iter()
        .map(|h| InsomniaHeader {
            name: h.key.clone(),
            value: h.value.clone(),
            disabled: Some(!h.enabled),
        })
        .collect();

    let parameters = req
        .params
        .iter()
        .map(|p| InsomniaParameter {
            name: p.key.clone(),
            value: p.value.clone(),
            disabled: Some(!p.enabled),
        })
        .collect();

    let authentication = convert_auth_to_insomnia(&req.auth);
    let body = convert_body_to_insomnia(&req.body);

    InsomniaResource {
        id: format!("req_{}", req.id),
        resource_type,
        parent_id: Some(parent_id.to_string()),
        name: Some(req.name.clone()),
        url: Some(req.url.clone()),
        method: Some(req.method.as_str().to_string()),
        headers: Some(headers),
        parameters: Some(parameters),
        body: Some(body),
        authentication: Some(authentication),
        meta_sort_key: Some(meta_sort_key),
        ..Default::default()
    }
}

fn convert_graphql_to_insomnia(
    req: &GraphQlRequest,
    parent_id: &str,
    meta_sort_key: f64,
) -> InsomniaResource {
    let headers = req
        .headers
        .iter()
        .map(|h| InsomniaHeader {
            name: h.key.clone(),
            value: h.value.clone(),
            disabled: Some(!h.enabled),
        })
        .collect();

    let authentication = convert_auth_to_insomnia(&req.auth);
    let payload = json!({
        "query": req.query,
        "variables": if req.variables.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_str::<serde_json::Value>(&req.variables).unwrap_or(serde_json::Value::String(req.variables.clone()))
        }
    });

    let body = InsomniaBody {
        mime_type: Some("application/json".to_string()),
        text: Some(payload.to_string()),
        params: None,
    };

    InsomniaResource {
        id: format!("req_{}", req.id),
        resource_type: "request".to_string(),
        parent_id: Some(parent_id.to_string()),
        name: Some(req.name.clone()),
        url: Some(req.url.clone()),
        method: Some("POST".to_string()),
        headers: Some(headers),
        parameters: None,
        body: Some(body),
        authentication: Some(authentication),
        meta_sort_key: Some(meta_sort_key),
        ..Default::default()
    }
}

fn convert_grpc_to_insomnia(
    req: &GrpcRequest,
    parent_id: &str,
    meta_sort_key: f64,
) -> InsomniaResource {
    InsomniaResource {
        id: format!("greq_{}", req.id),
        resource_type: "grpc_request".to_string(),
        parent_id: Some(parent_id.to_string()),
        name: Some(req.name.clone()),
        url: Some(req.url.clone()),
        body: Some(InsomniaBody {
            text: Some(req.message.clone()),
            ..Default::default()
        }),
        meta_sort_key: Some(meta_sort_key),
        ..Default::default()
    }
}

fn convert_auth_to_insomnia(auth: &AuthConfig) -> InsomniaAuth {
    match auth.auth_type {
        AuthType::None => InsomniaAuth::default(),
        AuthType::Bearer => {
            let token = auth
                .bearer
                .as_ref()
                .map(|b| b.token.clone())
                .unwrap_or_default();
            InsomniaAuth {
                auth_type: Some("bearer".to_string()),
                token: Some(token),
                ..Default::default()
            }
        }
        AuthType::Basic => {
            let (username, password) = auth
                .basic
                .as_ref()
                .map(|b| (b.username.clone(), b.password.clone()))
                .unwrap_or_default();
            InsomniaAuth {
                auth_type: Some("basic".to_string()),
                username: Some(username),
                password: Some(password),
                ..Default::default()
            }
        }
        AuthType::ApiKey => {
            let (key, value, add_to) = auth
                .api_key
                .as_ref()
                .map(|a| {
                    (
                        a.key.clone(),
                        a.value.clone(),
                        match a.add_to {
                            ApiKeyTarget::Header => "header",
                            ApiKeyTarget::Query => "query",
                        },
                    )
                })
                .unwrap_or_default();
            InsomniaAuth {
                auth_type: Some("apikey".to_string()),
                key: Some(key),
                value: Some(value),
                add_to: Some(add_to.to_string()),
                ..Default::default()
            }
        }
    }
}

fn convert_body_to_insomnia(body: &RequestBody) -> InsomniaBody {
    match body.mode {
        Some(BodyMode::Json) => InsomniaBody {
            mime_type: Some("application/json".to_string()),
            text: body.raw.clone(),
            params: None,
        },
        Some(BodyMode::Raw) => InsomniaBody {
            mime_type: Some("text/plain".to_string()),
            text: body.raw.clone(),
            params: None,
        },
        Some(BodyMode::Urlencoded) => InsomniaBody {
            mime_type: Some("application/x-www-form-urlencoded".to_string()),
            text: None,
            params: body.url_encoded.as_ref().map(|rows| {
                rows.iter()
                    .map(|r| InsomniaBodyParam {
                        name: r.key.clone(),
                        value: Some(r.value.clone()),
                        disabled: Some(!r.enabled),
                        file_name: None,
                    })
                    .collect()
            }),
        },
        Some(BodyMode::FormData) | Some(BodyMode::Multipart) => InsomniaBody {
            mime_type: Some("multipart/form-data".to_string()),
            text: None,
            params: body.form_data.as_ref().map(|rows| {
                rows.iter()
                    .map(|r| InsomniaBodyParam {
                        name: r.key.clone(),
                        value: Some(r.value.clone()),
                        disabled: Some(!r.enabled),
                        file_name: None,
                    })
                    .collect()
            }),
        },
        None => InsomniaBody::default(),
    }
}
