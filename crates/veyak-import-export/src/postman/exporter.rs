use serde_json::json;
use veyak_error::AppResult;
use veyak_models::{
    ApiKeyTarget, ApiRequest, AuthConfig, AuthType, BodyMode, Collection, Environment,
    EnvironmentVariable, Folder, GraphQlRequest, HttpMethod, KeyValueRow, RequestBody, RequestItem,
};

use crate::postman::models::{
    PostmanAuth, PostmanAuthParam, PostmanBody, PostmanCollection, PostmanEnvValue,
    PostmanEnvironment, PostmanFormDataParam, PostmanGraphQl, PostmanHeader, PostmanInfo,
    PostmanItem, PostmanQueryParam, PostmanRequest, PostmanRequestUnion, PostmanUrl,
    PostmanUrlEncodedParam, PostmanUrlUnion, PostmanVariable,
};

pub fn export_collection_as_postman(
    collection: &Collection,
    folders: &[Folder],
    requests: &[RequestItem],
    environments: Option<&[(&Environment, &[EnvironmentVariable])]>,
) -> AppResult<String> {
    let postman_items = build_folder_items(None, folders, requests);

    let variable = environments.and_then(|envs| {
        let mut vars = Vec::new();
        for (_env, v_list) in envs {
            for v in *v_list {
                vars.push(PostmanVariable {
                    id: Some(v.id.clone()),
                    key: Some(v.key.clone()),
                    value: Some(serde_json::Value::String(v.value.clone())),
                    var_type: Some(if v.is_secret {
                        "secret".to_string()
                    } else {
                        "string".to_string()
                    }),
                    disabled: Some(!v.enabled),
                });
            }
        }
        if vars.is_empty() { None } else { Some(vars) }
    });

    let postman_collection = PostmanCollection {
        info: PostmanInfo {
            postman_id: Some(collection.id.clone()),
            name: collection.name.clone(),
            description: None,
            schema: Some(
                "https://schema.getpostman.com/json/collection/v2.1.0/collection.json".to_string(),
            ),
        },
        item: postman_items,
        variable,
        auth: None,
    };

    let json_str = serde_json::to_string_pretty(&postman_collection)?;
    Ok(json_str)
}

pub fn export_environment_as_postman(
    environment: &Environment,
    variables: &[EnvironmentVariable],
) -> AppResult<String> {
    let postman_env = PostmanEnvironment {
        id: Some(environment.id.clone()),
        name: environment.name.clone(),
        scope: Some("environment".to_string()),
        values: variables
            .iter()
            .map(|v| PostmanEnvValue {
                key: v.key.clone(),
                value: serde_json::Value::String(v.value.clone()),
                enabled: Some(v.enabled),
                value_type: Some(if v.is_secret {
                    "secret".to_string()
                } else {
                    "default".to_string()
                }),
            })
            .collect(),
    };

    let json_str = serde_json::to_string_pretty(&postman_env)?;
    Ok(json_str)
}

fn build_folder_items(
    parent_id: Option<&str>,
    folders: &[Folder],
    requests: &[RequestItem],
) -> Vec<PostmanItem> {
    let mut items = Vec::new();

    // 1. Add subfolders matching parent_id
    let mut child_folders: Vec<&Folder> = folders
        .iter()
        .filter(|f| f.parent_folder_id.as_deref() == parent_id)
        .collect();
    child_folders.sort_by_key(|f| f.sort_order);

    for folder in child_folders {
        let sub_items = build_folder_items(Some(&folder.id), folders, requests);
        items.push(PostmanItem {
            id: Some(folder.id.clone()),
            name: Some(folder.name.clone()),
            description: None,
            item: Some(sub_items),
            request: None,
            response: None,
        });
    }

    // 2. Add requests matching parent_id (folder_id)
    let mut child_requests: Vec<&RequestItem> = requests
        .iter()
        .filter(|r| r.folder_id() == parent_id)
        .collect();
    child_requests.sort_by_key(|r| r.sort_order());

    for req in child_requests {
        items.push(convert_request_item_to_postman(req));
    }

    items
}

fn convert_request_item_to_postman(req_item: &RequestItem) -> PostmanItem {
    match req_item {
        RequestItem::Http(req) => convert_http_to_postman(req),
        RequestItem::GraphQL(req) => convert_graphql_to_postman(req),
        RequestItem::Grpc(req) => {
            // Represent gRPC in Postman request format
            PostmanItem {
                id: Some(req.id.clone()),
                name: Some(req.name.clone()),
                description: None,
                item: None,
                request: Some(PostmanRequestUnion::Object(PostmanRequest {
                    url: Some(PostmanUrlUnion::String(req.url.clone())),
                    method: Some("POST".to_string()),
                    header: Some(
                        req.metadata
                            .iter()
                            .map(|m| PostmanHeader {
                                key: m.key.clone(),
                                value: m.value.clone(),
                                disabled: Some(!m.enabled),
                                description: None,
                            })
                            .collect(),
                    ),
                    body: Some(PostmanBody {
                        mode: Some("raw".to_string()),
                        raw: Some(req.message.clone()),
                        options: Some(json!({ "raw": { "language": "json" } })),
                        ..Default::default()
                    }),
                    auth: None,
                    description: Some(format!("Service: {} Method: {}", req.service, req.method)),
                })),
                response: None,
            }
        }
    }
}

fn convert_http_to_postman(req: &ApiRequest) -> PostmanItem {
    let url = build_postman_url(&req.url, &req.params);
    let header = req
        .headers
        .iter()
        .map(|h| PostmanHeader {
            key: h.key.clone(),
            value: h.value.clone(),
            disabled: Some(!h.enabled),
            description: None,
        })
        .collect();

    let auth = convert_auth_to_postman(&req.auth);
    let body = convert_body_to_postman(&req.body);

    let method_str = if req.method == HttpMethod::Ws {
        "GET".to_string()
    } else {
        req.method.as_str().to_string()
    };

    PostmanItem {
        id: Some(req.id.clone()),
        name: Some(req.name.clone()),
        description: None,
        item: None,
        request: Some(PostmanRequestUnion::Object(PostmanRequest {
            url: Some(PostmanUrlUnion::Object(url)),
            method: Some(method_str),
            header: Some(header),
            body: Some(body),
            auth,
            description: None,
        })),
        response: None,
    }
}

fn convert_graphql_to_postman(req: &GraphQlRequest) -> PostmanItem {
    let url = PostmanUrl {
        raw: Some(req.url.clone()),
        ..Default::default()
    };

    let header = req
        .headers
        .iter()
        .map(|h| PostmanHeader {
            key: h.key.clone(),
            value: h.value.clone(),
            disabled: Some(!h.enabled),
            description: None,
        })
        .collect();

    let auth = convert_auth_to_postman(&req.auth);
    let body = PostmanBody {
        mode: Some("graphql".to_string()),
        graphql: Some(PostmanGraphQl {
            query: Some(req.query.clone()),
            variables: Some(req.variables.clone()),
        }),
        ..Default::default()
    };

    PostmanItem {
        id: Some(req.id.clone()),
        name: Some(req.name.clone()),
        description: None,
        item: None,
        request: Some(PostmanRequestUnion::Object(PostmanRequest {
            url: Some(PostmanUrlUnion::Object(url)),
            method: Some("POST".to_string()),
            header: Some(header),
            body: Some(body),
            auth,
            description: None,
        })),
        response: None,
    }
}

fn build_postman_url(raw_url: &str, params: &[KeyValueRow]) -> PostmanUrl {
    let query_params: Vec<PostmanQueryParam> = params
        .iter()
        .map(|p| PostmanQueryParam {
            key: Some(p.key.clone()),
            value: Some(p.value.clone()),
            disabled: Some(!p.enabled),
            description: None,
        })
        .collect();

    PostmanUrl {
        raw: Some(raw_url.to_string()),
        query: if query_params.is_empty() {
            None
        } else {
            Some(query_params)
        },
        ..Default::default()
    }
}

fn convert_auth_to_postman(auth: &AuthConfig) -> Option<PostmanAuth> {
    match auth.auth_type {
        AuthType::None => None,
        AuthType::Bearer => {
            let token = auth
                .bearer
                .as_ref()
                .map(|b| b.token.clone())
                .unwrap_or_default();
            Some(PostmanAuth {
                auth_type: "bearer".to_string(),
                bearer: Some(vec![PostmanAuthParam {
                    key: "token".to_string(),
                    value: Some(serde_json::Value::String(token)),
                    param_type: Some("string".to_string()),
                }]),
                basic: None,
                apikey: None,
            })
        }
        AuthType::Basic => {
            let (username, password) = auth
                .basic
                .as_ref()
                .map(|b| (b.username.clone(), b.password.clone()))
                .unwrap_or_default();
            Some(PostmanAuth {
                auth_type: "basic".to_string(),
                bearer: None,
                basic: Some(vec![
                    PostmanAuthParam {
                        key: "username".to_string(),
                        value: Some(serde_json::Value::String(username)),
                        param_type: Some("string".to_string()),
                    },
                    PostmanAuthParam {
                        key: "password".to_string(),
                        value: Some(serde_json::Value::String(password)),
                        param_type: Some("string".to_string()),
                    },
                ]),
                apikey: None,
            })
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
            Some(PostmanAuth {
                auth_type: "apikey".to_string(),
                bearer: None,
                basic: None,
                apikey: Some(vec![
                    PostmanAuthParam {
                        key: "key".to_string(),
                        value: Some(serde_json::Value::String(key)),
                        param_type: Some("string".to_string()),
                    },
                    PostmanAuthParam {
                        key: "value".to_string(),
                        value: Some(serde_json::Value::String(value)),
                        param_type: Some("string".to_string()),
                    },
                    PostmanAuthParam {
                        key: "in".to_string(),
                        value: Some(serde_json::Value::String(add_to.to_string())),
                        param_type: Some("string".to_string()),
                    },
                ]),
            })
        }
    }
}

fn convert_body_to_postman(body: &RequestBody) -> PostmanBody {
    match body.mode {
        Some(BodyMode::Json) => PostmanBody {
            mode: Some("raw".to_string()),
            raw: body.raw.clone(),
            options: Some(json!({ "raw": { "language": "json" } })),
            ..Default::default()
        },
        Some(BodyMode::Raw) => PostmanBody {
            mode: Some("raw".to_string()),
            raw: body.raw.clone(),
            ..Default::default()
        },
        Some(BodyMode::Urlencoded) => PostmanBody {
            mode: Some("urlencoded".to_string()),
            urlencoded: body.url_encoded.as_ref().map(|rows| {
                rows.iter()
                    .map(|r| PostmanUrlEncodedParam {
                        key: r.key.clone(),
                        value: r.value.clone(),
                        disabled: Some(!r.enabled),
                        description: None,
                    })
                    .collect()
            }),
            ..Default::default()
        },
        Some(BodyMode::FormData) | Some(BodyMode::Multipart) => PostmanBody {
            mode: Some("formdata".to_string()),
            formdata: body.form_data.as_ref().map(|rows| {
                rows.iter()
                    .map(|r| PostmanFormDataParam {
                        key: r.key.clone(),
                        value: Some(r.value.clone()),
                        param_type: Some("text".to_string()),
                        disabled: Some(!r.enabled),
                        src: None,
                        description: None,
                    })
                    .collect()
            }),
            ..Default::default()
        },
        None => PostmanBody::default(),
    }
}
