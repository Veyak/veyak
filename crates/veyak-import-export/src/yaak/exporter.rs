use chrono::Utc;
use serde_json::json;
use veyak_error::AppResult;
use veyak_models::{
    ApiKeyTarget, ApiRequest, AuthConfig, AuthType, BodyMode, Collection, Environment,
    EnvironmentVariable, Folder, GraphQlRequest, GrpcRequest, RequestBody, RequestItem,
};

use crate::yaak::models::{
    YaakBodyObject, YaakBodyUnion, YaakEnvironment, YaakExport, YaakFolder, YaakGrpcRequest,
    YaakHeader, YaakHttpRequest, YaakKeyValuePair, YaakResourceObject, YaakResourcesUnion,
    YaakUrlParameter, YaakVariable, YaakWorkspace,
};

pub fn export_collection_as_yaak(
    collection: &Collection,
    folders: &[Folder],
    requests: &[RequestItem],
    environments: Option<&[(&Environment, &[EnvironmentVariable])]>,
) -> AppResult<String> {
    let ws_id = collection.id.clone();

    let yaak_folders = folders
        .iter()
        .map(|f| YaakFolder {
            id: f.id.clone(),
            name: f.name.clone(),
            workspace_id: Some(ws_id.clone()),
            folder_id: f.parent_folder_id.clone(),
            sort_priority: Some(f.sort_order as f64),
        })
        .collect();

    let mut http_requests = Vec::new();
    let mut grpc_requests = Vec::new();

    for req in requests {
        match req {
            RequestItem::Http(http) => {
                http_requests.push(convert_http_to_yaak(http, &ws_id));
            }
            RequestItem::GraphQL(gql) => {
                http_requests.push(convert_graphql_to_yaak(gql, &ws_id));
            }
            RequestItem::Grpc(grpc) => {
                grpc_requests.push(convert_grpc_to_yaak(grpc, &ws_id));
            }
        }
    }

    let yaak_environments = if let Some(envs) = environments {
        envs.iter()
            .map(|(env, vars)| YaakEnvironment {
                id: env.id.clone(),
                name: env.name.clone(),
                workspace_id: Some(ws_id.clone()),
                variables: vars
                    .iter()
                    .map(|v| YaakVariable {
                        name: v.key.clone(),
                        value: serde_json::Value::String(v.value.clone()),
                        enabled: Some(v.enabled),
                    })
                    .collect(),
            })
            .collect()
    } else {
        Vec::new()
    };

    let export = YaakExport {
        yaak_schema: 3,
        timestamp: Some(Utc::now().to_rfc3339()),
        resources: YaakResourcesUnion::Object(YaakResourceObject {
            workspaces: vec![YaakWorkspace {
                id: ws_id,
                name: collection.name.clone(),
                description: None,
            }],
            environments: yaak_environments,
            folders: yaak_folders,
            http_requests,
            grpc_requests,
        }),
    };

    let json_str = serde_json::to_string_pretty(&export)?;
    Ok(json_str)
}

fn convert_http_to_yaak(req: &ApiRequest, ws_id: &str) -> YaakHttpRequest {
    let headers = req
        .headers
        .iter()
        .map(|h| YaakHeader {
            name: h.key.clone(),
            value: h.value.clone(),
            enabled: Some(h.enabled),
        })
        .collect();

    let url_parameters = req
        .params
        .iter()
        .map(|p| YaakUrlParameter {
            name: p.key.clone(),
            value: p.value.clone(),
            enabled: Some(p.enabled),
        })
        .collect();

    let (body, body_type) = convert_body_to_yaak(&req.body);
    let (auth, auth_type) = convert_auth_to_yaak(&req.auth);

    YaakHttpRequest {
        id: req.id.clone(),
        name: req.name.clone(),
        workspace_id: Some(ws_id.to_string()),
        folder_id: req.folder_id.clone(),
        url: Some(req.url.clone()),
        method: Some(req.method.as_str().to_string()),
        headers: Some(headers),
        url_parameters: Some(url_parameters),
        body,
        body_type,
        authentication: auth,
        authentication_type: auth_type,
        sort_priority: Some(req.sort_order as f64),
    }
}

fn convert_graphql_to_yaak(req: &GraphQlRequest, ws_id: &str) -> YaakHttpRequest {
    let headers = req
        .headers
        .iter()
        .map(|h| YaakHeader {
            name: h.key.clone(),
            value: h.value.clone(),
            enabled: Some(h.enabled),
        })
        .collect();

    let (auth, auth_type) = convert_auth_to_yaak(&req.auth);
    let payload = json!({
        "query": req.query,
        "variables": if req.variables.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_str::<serde_json::Value>(&req.variables).unwrap_or(serde_json::Value::String(req.variables.clone()))
        }
    });

    YaakHttpRequest {
        id: req.id.clone(),
        name: req.name.clone(),
        workspace_id: Some(ws_id.to_string()),
        folder_id: req.folder_id.clone(),
        url: Some(req.url.clone()),
        method: Some("POST".to_string()),
        headers: Some(headers),
        url_parameters: None,
        body: Some(YaakBodyUnion::Object(YaakBodyObject {
            text: Some(payload.to_string()),
            form: None,
        })),
        body_type: Some("application/json".to_string()),
        authentication: auth,
        authentication_type: auth_type,
        sort_priority: Some(req.sort_order as f64),
    }
}

fn convert_grpc_to_yaak(req: &GrpcRequest, ws_id: &str) -> YaakGrpcRequest {
    let metadata = req
        .metadata
        .iter()
        .map(|h| YaakHeader {
            name: h.key.clone(),
            value: h.value.clone(),
            enabled: Some(h.enabled),
        })
        .collect();

    YaakGrpcRequest {
        id: req.id.clone(),
        name: req.name.clone(),
        workspace_id: Some(ws_id.to_string()),
        folder_id: req.folder_id.clone(),
        url: Some(req.url.clone()),
        service: Some(req.service.clone()),
        method: Some(req.method.clone()),
        message: Some(req.message.clone()),
        metadata: Some(metadata),
        sort_priority: Some(req.sort_order as f64),
    }
}

fn convert_auth_to_yaak(auth: &AuthConfig) -> (Option<serde_json::Value>, Option<String>) {
    match auth.auth_type {
        AuthType::None => (None, None),
        AuthType::Bearer => {
            let token = auth
                .bearer
                .as_ref()
                .map(|b| b.token.clone())
                .unwrap_or_default();
            (Some(json!({ "token": token })), Some("bearer".to_string()))
        }
        AuthType::Basic => {
            let (username, password) = auth
                .basic
                .as_ref()
                .map(|b| (b.username.clone(), b.password.clone()))
                .unwrap_or_default();
            (
                Some(json!({ "username": username, "password": password })),
                Some("basic".to_string()),
            )
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
            (
                Some(json!({ "key": key, "value": value, "addTo": add_to })),
                Some("apikey".to_string()),
            )
        }
    }
}

fn convert_body_to_yaak(body: &RequestBody) -> (Option<YaakBodyUnion>, Option<String>) {
    match body.mode {
        Some(BodyMode::Json) => (
            Some(YaakBodyUnion::Object(YaakBodyObject {
                text: body.raw.clone(),
                form: None,
            })),
            Some("application/json".to_string()),
        ),
        Some(BodyMode::Raw) => (
            Some(YaakBodyUnion::Object(YaakBodyObject {
                text: body.raw.clone(),
                form: None,
            })),
            Some("text/plain".to_string()),
        ),
        Some(BodyMode::Urlencoded) => (
            Some(YaakBodyUnion::Object(YaakBodyObject {
                text: None,
                form: body.url_encoded.as_ref().map(|rows| {
                    rows.iter()
                        .map(|r| YaakKeyValuePair {
                            name: r.key.clone(),
                            value: r.value.clone(),
                            enabled: Some(r.enabled),
                        })
                        .collect()
                }),
            })),
            Some("application/x-www-form-urlencoded".to_string()),
        ),
        Some(BodyMode::FormData) | Some(BodyMode::Multipart) => (
            Some(YaakBodyUnion::Object(YaakBodyObject {
                text: None,
                form: body.form_data.as_ref().map(|rows| {
                    rows.iter()
                        .map(|r| YaakKeyValuePair {
                            name: r.key.clone(),
                            value: r.value.clone(),
                            enabled: Some(r.enabled),
                        })
                        .collect()
                }),
            })),
            Some("multipart/form-data".to_string()),
        ),
        None => (None, None),
    }
}
