use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YaakExport {
    #[serde(rename = "yaakSchema", alias = "YaakSchema", default = "default_yaak_schema")]
    pub yaak_schema: i64,
    #[serde(default)]
    pub timestamp: Option<String>,
    pub resources: YaakResourcesUnion,
}

fn default_yaak_schema() -> i64 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum YaakResourcesUnion {
    Object(YaakResourceObject),
    List(Vec<YaakResourceItem>),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct YaakResourceObject {
    #[serde(default)]
    pub workspaces: Vec<YaakWorkspace>,
    #[serde(default)]
    pub environments: Vec<YaakEnvironment>,
    #[serde(default)]
    pub folders: Vec<YaakFolder>,
    #[serde(default)]
    pub http_requests: Vec<YaakHttpRequest>,
    #[serde(default)]
    pub grpc_requests: Vec<YaakGrpcRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct YaakResourceItem {
    pub id: String,
    #[serde(default)]
    pub model: Option<String>, // "workspace", "folder", "http_request", "grpc_request", "environment"
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, alias = "workspace_id")]
    pub workspace_id: Option<String>,
    #[serde(default, alias = "folder_id", alias = "parentId")]
    pub folder_id: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub headers: Option<Vec<YaakHeader>>,
    #[serde(default, alias = "url_parameters")]
    pub url_parameters: Option<Vec<YaakUrlParameter>>,
    #[serde(default)]
    pub body: Option<YaakBodyUnion>,
    #[serde(default, alias = "body_type")]
    pub body_type: Option<String>,
    #[serde(default)]
    pub authentication: Option<Value>,
    #[serde(default, alias = "authentication_type")]
    pub authentication_type: Option<String>,
    #[serde(default)]
    pub variables: Option<Vec<YaakVariable>>,
    #[serde(default)]
    pub service: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub metadata: Option<Vec<YaakHeader>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct YaakWorkspace {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct YaakFolder {
    pub id: String,
    pub name: String,
    #[serde(default, alias = "workspace_id")]
    pub workspace_id: Option<String>,
    #[serde(default, alias = "folder_id", alias = "parentId")]
    pub folder_id: Option<String>,
    #[serde(default, alias = "sort_priority")]
    pub sort_priority: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct YaakHttpRequest {
    pub id: String,
    pub name: String,
    #[serde(default, alias = "workspace_id")]
    pub workspace_id: Option<String>,
    #[serde(default, alias = "folder_id", alias = "parentId")]
    pub folder_id: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub headers: Option<Vec<YaakHeader>>,
    #[serde(default, alias = "url_parameters")]
    pub url_parameters: Option<Vec<YaakUrlParameter>>,
    #[serde(default)]
    pub body: Option<YaakBodyUnion>,
    #[serde(default, alias = "body_type")]
    pub body_type: Option<String>,
    #[serde(default)]
    pub authentication: Option<Value>,
    #[serde(default, alias = "authentication_type")]
    pub authentication_type: Option<String>,
    #[serde(default, alias = "sort_priority")]
    pub sort_priority: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum YaakBodyUnion {
    String(String),
    Object(YaakBodyObject),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct YaakBodyObject {
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub form: Option<Vec<YaakKeyValuePair>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YaakHeader {
    pub name: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YaakUrlParameter {
    pub name: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YaakKeyValuePair {
    pub name: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct YaakGrpcRequest {
    pub id: String,
    pub name: String,
    #[serde(default, alias = "workspace_id")]
    pub workspace_id: Option<String>,
    #[serde(default, alias = "folder_id", alias = "parentId")]
    pub folder_id: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub service: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub metadata: Option<Vec<YaakHeader>>,
    #[serde(default, alias = "sort_priority")]
    pub sort_priority: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct YaakEnvironment {
    pub id: String,
    pub name: String,
    #[serde(default, alias = "workspace_id")]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub variables: Vec<YaakVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YaakVariable {
    pub name: String,
    #[serde(default)]
    pub value: Value,
    #[serde(default)]
    pub enabled: Option<bool>,
}
