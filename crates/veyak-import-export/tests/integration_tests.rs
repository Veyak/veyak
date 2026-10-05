use tempfile::tempdir;
use veyak_db::init_data_dir;
use veyak_models::{AuthType, BodyMode, ExportFormat, HttpMethod, ImportFormat, RequestItem};

use veyak_import_export::{
    detect_format, export_collection, export_environment, export_workspace, import_data,
    parse_import_content,
};

#[test]
fn test_detect_format() {
    let postman_col = r#"{
        "info": {
            "_postman_id": "12345",
            "name": "Test Postman",
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
        },
        "item": []
    }"#;
    assert_eq!(detect_format(postman_col), ImportFormat::Postman);

    let postman_env = r#"{
        "name": "Test Env",
        "values": [
            { "key": "host", "value": "localhost", "enabled": true }
        ],
        "_postman_variable_scope": "environment"
    }"#;
    assert_eq!(detect_format(postman_env), ImportFormat::PostmanEnvironment);

    let insomnia_json = r#"{
        "_type": "export",
        "__export_format": 4,
        "resources": [
            { "_id": "wrk_1", "_type": "workspace", "name": "Insomnia WS" }
        ]
    }"#;
    assert_eq!(detect_format(insomnia_json), ImportFormat::Insomnia);

    let insomnia_yaml = "
_type: export
__export_format: 4
resources:
  - _id: wrk_1
    _type: workspace
    name: Insomnia Yaml WS
";
    assert_eq!(detect_format(insomnia_yaml), ImportFormat::Insomnia);

    let yaak_json = r#"{
        "yaakSchema": 3,
        "resources": {
            "workspaces": [
                { "id": "w1", "name": "Yaak WS" }
            ],
            "httpRequests": []
        }
    }"#;
    assert_eq!(detect_format(yaak_json), ImportFormat::Yaak);
}

#[test]
fn test_postman_collection_import_and_export() {
    let postman_raw = r#"{
        "info": {
            "_postman_id": "test-pm-id",
            "name": "Sample Postman API",
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
        },
        "item": [
            {
                "name": "Users Folder",
                "item": [
                    {
                        "name": "Get User Details",
                        "request": {
                            "method": "GET",
                            "header": [
                                { "key": "Accept", "value": "application/json", "disabled": false }
                            ],
                            "url": {
                                "raw": "https://api.example.com/users/42?verbose=true",
                                "query": [
                                    { "key": "verbose", "value": "true" }
                                ]
                            },
                            "auth": {
                                "type": "bearer",
                                "bearer": [
                                    { "key": "token", "value": "secret-jwt-token" }
                                ]
                            }
                        }
                    }
                ]
            },
            {
                "name": "Create User",
                "request": {
                    "method": "POST",
                    "url": "https://api.example.com/users",
                    "header": [
                        { "key": "Content-Type", "value": "application/json" }
                    ],
                    "body": {
                        "mode": "raw",
                        "raw": "{\"name\": \"Alice\"}",
                        "options": {
                            "raw": { "language": "json" }
                        }
                    }
                }
            }
        ],
        "variable": [
            { "key": "base_url", "value": "https://api.example.com" }
        ]
    }"#;

    let import_data = parse_import_content(postman_raw, ImportFormat::Auto).unwrap();
    assert_eq!(import_data.collections.len(), 1);
    let col = &import_data.collections[0];
    assert_eq!(col.name, "Sample Postman API");
    assert_eq!(col.folders.len(), 1);
    assert_eq!(col.folders[0].name, "Users Folder");
    assert_eq!(col.requests.len(), 2);

    let get_req = col
        .requests
        .iter()
        .find(|r| match r {
            RequestItem::Http(h) => h.name == "Get User Details",
            _ => false,
        })
        .unwrap();

    if let RequestItem::Http(h) = get_req {
        assert_eq!(h.method, HttpMethod::Get);
        assert_eq!(h.folder_id.as_deref(), Some(col.folders[0].id.as_str()));
        assert_eq!(h.auth.auth_type, AuthType::Bearer);
        assert_eq!(
            h.auth.bearer.as_ref().map(|b| b.token.as_str()),
            Some("secret-jwt-token")
        );
        assert_eq!(h.headers.len(), 1);
        assert_eq!(h.params.len(), 1);
        assert_eq!(h.params[0].key, "verbose");
    }

    let post_req = col
        .requests
        .iter()
        .find(|r| match r {
            RequestItem::Http(h) => h.name == "Create User",
            _ => false,
        })
        .unwrap();

    if let RequestItem::Http(h) = post_req {
        assert_eq!(h.method, HttpMethod::Post);
        assert_eq!(h.body.mode, Some(BodyMode::Json));
        assert_eq!(h.body.raw.as_deref(), Some("{\"name\": \"Alice\"}"));
    }

    // Verify collection variables became an environment
    assert_eq!(import_data.environments.len(), 1);
    assert_eq!(import_data.environments[0].variables[0].key, "base_url");
    assert_eq!(
        import_data.environments[0].variables[0].value,
        "https://api.example.com"
    );
}

#[test]
fn test_insomnia_import_and_export() {
    let insomnia_raw = r#"{
        "_type": "export",
        "__export_format": 4,
        "resources": [
            {
                "_id": "wrk_1",
                "_type": "workspace",
                "name": "My Insomnia API"
            },
            {
                "_id": "fld_1",
                "parentId": "wrk_1",
                "_type": "request_group",
                "name": "Auth Endpoints"
            },
            {
                "_id": "req_1",
                "parentId": "fld_1",
                "_type": "request",
                "name": "Login",
                "method": "POST",
                "url": "https://api.example.com/login",
                "headers": [
                    { "name": "Content-Type", "value": "application/x-www-form-urlencoded" }
                ],
                "body": {
                    "mimeType": "application/x-www-form-urlencoded",
                    "params": [
                        { "name": "username", "value": "admin" },
                        { "name": "password", "value": "pass123" }
                    ]
                }
            },
            {
                "_id": "ws_1",
                "parentId": "wrk_1",
                "_type": "websocket_request",
                "name": "Live Feed",
                "url": "wss://stream.example.com/feed"
            },
            {
                "_id": "env_1",
                "parentId": "wrk_1",
                "_type": "environment",
                "name": "Staging",
                "data": {
                    "baseUrl": "https://staging.example.com"
                }
            }
        ]
    }"#;

    let import_data = parse_import_content(insomnia_raw, ImportFormat::Insomnia).unwrap();
    assert_eq!(import_data.collections.len(), 1);
    let col = &import_data.collections[0];
    assert_eq!(col.name, "My Insomnia API");
    assert_eq!(col.folders.len(), 1);
    assert_eq!(col.folders[0].name, "Auth Endpoints");
    assert_eq!(col.requests.len(), 2);

    let login_req = col
        .requests
        .iter()
        .find(|r| match r {
            RequestItem::Http(h) => h.name == "Login",
            _ => false,
        })
        .unwrap();

    if let RequestItem::Http(h) = login_req {
        assert_eq!(h.method, HttpMethod::Post);
        assert_eq!(h.body.mode, Some(BodyMode::Urlencoded));
        assert_eq!(h.body.url_encoded.as_ref().unwrap().len(), 2);
    }

    let ws_req = col
        .requests
        .iter()
        .find(|r| match r {
            RequestItem::Http(h) => h.name == "Live Feed",
            _ => false,
        })
        .unwrap();

    if let RequestItem::Http(h) = ws_req {
        assert_eq!(h.method, HttpMethod::Ws);
        assert_eq!(h.url, "wss://stream.example.com/feed");
    }

    assert_eq!(import_data.environments.len(), 1);
    assert_eq!(import_data.environments[0].name, "Staging");
    assert_eq!(import_data.environments[0].variables[0].key, "baseUrl");
    assert_eq!(
        import_data.environments[0].variables[0].value,
        "https://staging.example.com"
    );
}

#[test]
fn test_yaak_import() {
    let yaak_raw = r#"{
        "yaakSchema": 3,
        "resources": {
            "workspaces": [
                { "id": "w1", "name": "Yaak Demo" }
            ],
            "folders": [
                { "id": "f1", "name": "V1", "workspaceId": "w1" }
            ],
            "httpRequests": [
                {
                    "id": "r1",
                    "workspaceId": "w1",
                    "folderId": "f1",
                    "name": "List Products",
                    "url": "https://store.example.com/products",
                    "method": "GET",
                    "headers": [
                        { "name": "X-Client", "value": "yaak", "enabled": true }
                    ],
                    "urlParameters": [
                        { "name": "limit", "value": "20", "enabled": true }
                    ],
                    "authenticationType": "basic",
                    "authentication": {
                        "username": "user",
                        "password": "secretpassword"
                    }
                }
            ],
            "grpcRequests": [
                {
                    "id": "g1",
                    "workspaceId": "w1",
                    "name": "Greeter SayHello",
                    "url": "grpc.example.com:50051",
                    "service": "helloworld.Greeter",
                    "method": "SayHello",
                    "message": "{\"name\": \"World\"}"
                }
            ],
            "environments": [
                {
                    "id": "e1",
                    "name": "Production",
                    "workspaceId": "w1",
                    "variables": [
                        { "name": "API_KEY", "value": "12345", "enabled": true }
                    ]
                }
            ]
        }
    }"#;

    let import_data = parse_import_content(yaak_raw, ImportFormat::Yaak).unwrap();
    assert_eq!(import_data.collections.len(), 1);
    let col = &import_data.collections[0];
    assert_eq!(col.name, "Yaak Demo");
    assert_eq!(col.folders.len(), 1);
    assert_eq!(col.folders[0].name, "V1");
    assert_eq!(col.requests.len(), 2);

    let http_req = col
        .requests
        .iter()
        .find(|r| match r {
            RequestItem::Http(h) => h.name == "List Products",
            _ => false,
        })
        .unwrap();

    if let RequestItem::Http(h) = http_req {
        assert_eq!(h.method, HttpMethod::Get);
        assert_eq!(h.auth.auth_type, AuthType::Basic);
        assert_eq!(
            h.auth
                .basic
                .as_ref()
                .map(|b| (b.username.as_str(), b.password.as_str())),
            Some(("user", "secretpassword"))
        );
        assert_eq!(h.headers[0].key, "X-Client");
        assert_eq!(h.params[0].key, "limit");
    }

    let grpc_req = col
        .requests
        .iter()
        .find(|r| match r {
            RequestItem::Grpc(g) => g.name == "Greeter SayHello",
            _ => false,
        })
        .unwrap();

    if let RequestItem::Grpc(g) = grpc_req {
        assert_eq!(g.service, "helloworld.Greeter");
        assert_eq!(g.method, "SayHello");
        assert_eq!(g.message, "{\"name\": \"World\"}");
    }

    assert_eq!(import_data.environments.len(), 1);
    assert_eq!(import_data.environments[0].variables[0].key, "API_KEY");
}

#[test]
fn test_persisting_to_datadir_and_exporting() {
    let tmp = tempdir().unwrap();
    let dd = init_data_dir(tmp.path()).unwrap();
    let ws_id = "default-workspace";

    let postman_raw = r#"{
        "info": {
            "_postman_id": "pm-test",
            "name": "Integration Collection",
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
        },
        "item": [
            {
                "name": "Ping",
                "request": {
                    "method": "GET",
                    "url": "https://api.example.com/ping"
                }
            }
        ]
    }"#;

    let summary = import_data(&dd, ws_id, None, postman_raw, ImportFormat::Auto).unwrap();
    assert_eq!(summary.collections_count, 1);
    assert_eq!(summary.requests_count, 1);

    // List collections to verify
    let cols_dir = dd.collections_dir(ws_id);
    let entries: Vec<_> = std::fs::read_dir(&cols_dir)
        .unwrap()
        .map(|e| e.unwrap())
        .collect();
    assert_eq!(entries.len(), 1);
    let col_id = entries[0].file_name().to_string_lossy().to_string();

    // Export to Postman
    let exported_pm = export_collection(&dd, ws_id, &col_id, ExportFormat::Postman).unwrap();
    assert!(exported_pm.contains("Integration Collection"));
    assert!(exported_pm.contains("Ping"));

    // Export to Insomnia
    let exported_insomnia = export_collection(&dd, ws_id, &col_id, ExportFormat::Insomnia).unwrap();
    assert!(exported_insomnia.contains("Integration Collection"));
    assert!(exported_insomnia.contains("_type\": \"export\""));

    // Export to Yaak
    let exported_yaak = export_collection(&dd, ws_id, &col_id, ExportFormat::Yaak).unwrap();
    assert!(exported_yaak.contains("yaakSchema"));
    assert!(exported_yaak.contains("Integration Collection"));

    // Export to Veyak native
    let exported_veyak = export_collection(&dd, ws_id, &col_id, ExportFormat::Veyak).unwrap();
    assert!(exported_veyak.contains("veyakSchema"));
    assert!(exported_veyak.contains("Integration Collection"));

    // Export workspace
    let exported_ws = export_workspace(&dd, ws_id, ExportFormat::Veyak).unwrap();
    assert!(exported_ws.contains("workspace"));

    // Import environment
    let env_json = r#"{
        "name": "Integration Env",
        "values": [
            { "key": "API_URL", "value": "https://api.example.com", "enabled": true }
        ],
        "_postman_variable_scope": "environment"
    }"#;
    let env_summary = import_data(&dd, ws_id, None, env_json, ImportFormat::Auto).unwrap();
    assert_eq!(env_summary.environments_count, 1);

    let envs_path = dd.environments_path(ws_id);
    let envs: Vec<veyak_models::EnvironmentWithVariables> =
        veyak_db::read_yaml_vec(&envs_path).unwrap();
    let env_id = &envs[0].environment.id;

    let exported_env = export_environment(&dd, ws_id, env_id, ExportFormat::Postman).unwrap();
    assert!(exported_env.contains("Integration Env"));
    assert!(exported_env.contains("API_URL"));
}
