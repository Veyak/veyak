use serde::{Deserialize, Serialize};

/// Top-level Insomnia v5 export structure conforming to
/// https://raw.githubusercontent.com/Kong/insomnia/develop/schemas/insomnia.schema.5.1.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InsomniaV5File {
    #[serde(rename = "type", default)]
    pub file_type: Option<String>,
    #[serde(rename = "schema_version", default)]
    pub schema_version: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub meta: Option<InsomniaV5Meta>,

    // collection.insomnia.rest/5.0 and spec.insomnia.rest/5.0
    #[serde(default)]
    pub collection: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub requests: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub data: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub environments: Option<serde_json::Value>,
    #[serde(rename = "cookieJar", default)]
    pub cookie_jar: Option<InsomniaV5CookieJar>,
    #[serde(default)]
    pub spec: Option<serde_json::Value>,

    // mock.insomnia.rest/5.0
    #[serde(default)]
    pub server: Option<serde_json::Value>,
    #[serde(default)]
    pub routes: Option<Vec<serde_json::Value>>,

    // mcpClient.insomnia/5.0
    #[serde(rename = "mcpRequest", default)]
    pub mcp_request: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InsomniaV5Meta {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub created: Option<f64>,
    #[serde(default)]
    pub modified: Option<f64>,
    #[serde(rename = "isPrivate", default)]
    pub is_private: Option<bool>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "sortKey", default)]
    pub sort_key: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InsomniaV5Item {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub meta: Option<InsomniaV5Meta>,
    #[serde(default)]
    pub description: Option<String>,

    // Folders
    #[serde(default)]
    pub children: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub requests: Option<Vec<serde_json::Value>>,

    // HTTP / WebSocket / SocketIO
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub headers: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub parameters: Option<Vec<serde_json::Value>>,
    #[serde(rename = "queryparams", default)]
    pub query_params: Option<serde_json::Value>,
    #[serde(rename = "pathParameters", default)]
    pub path_parameters: Option<serde_json::Value>,
    #[serde(default)]
    pub body: Option<serde_json::Value>,
    #[serde(default)]
    pub authentication: Option<serde_json::Value>,
    #[serde(default)]
    pub scripts: Option<InsomniaV5Scripts>,
    #[serde(default)]
    pub settings: Option<serde_json::Value>,

    // gRPC
    #[serde(default)]
    pub metadata: Option<Vec<serde_json::Value>>,
    #[serde(rename = "protoFileId", default)]
    pub proto_file_id: Option<String>,
    #[serde(rename = "protoMethodName", default)]
    pub proto_method_name: Option<String>,
    #[serde(rename = "reflectionApi", default)]
    pub reflection_api: Option<InsomniaV5ReflectionApi>,

    // WebSocket / SocketIO
    #[serde(rename = "eventListeners", default)]
    pub event_listeners: Option<serde_json::Value>,

    // Mock Route
    #[serde(rename = "statusCode", default)]
    pub status_code: Option<u16>,
    #[serde(rename = "mimeType", default)]
    pub mime_type: Option<String>,

    // MCP
    #[serde(rename = "transportType", default)]
    pub transport_type: Option<String>,

    // Folder Environment
    #[serde(default)]
    pub environment: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InsomniaV5Scripts {
    #[serde(rename = "preRequest", default)]
    pub pre_request: Option<serde_json::Value>,
    #[serde(rename = "afterResponse", default)]
    pub after_response: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InsomniaV5ReflectionApi {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(rename = "apiKey", default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub module: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InsomniaV5CookieJar {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub cookies: Option<Vec<InsomniaV5Cookie>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InsomniaV5Cookie {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
}
