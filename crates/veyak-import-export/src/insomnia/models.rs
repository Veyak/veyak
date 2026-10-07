use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsomniaExport {
    #[serde(rename = "_type", default = "default_export_type")]
    pub export_type: String,
    #[serde(rename = "__export_format", default = "default_export_format")]
    pub export_format: i64,
    #[serde(rename = "__export_date", default)]
    pub export_date: Option<String>,
    #[serde(rename = "__export_source", default)]
    pub export_source: Option<String>,
    #[serde(default)]
    pub resources: Vec<InsomniaResource>,
}

fn default_export_type() -> String {
    "export".to_string()
}

fn default_export_format() -> i64 {
    4
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InsomniaResource {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_type")]
    pub resource_type: String,
    #[serde(rename = "parentId", default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "metaSortKey", default)]
    pub meta_sort_key: Option<f64>,

    // Request fields
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub headers: Option<Vec<InsomniaHeader>>,
    #[serde(default)]
    pub parameters: Option<Vec<InsomniaParameter>>,
    #[serde(default)]
    pub body: Option<InsomniaBody>,
    #[serde(default)]
    pub authentication: Option<InsomniaAuth>,

    // Environment fields
    #[serde(default)]
    pub data: Option<serde_json::Map<String, Value>>,
    #[serde(default)]
    pub color: Option<String>,

    // gRPC fields
    #[serde(rename = "protoFileId", default)]
    pub proto_file_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InsomniaHeader {
    pub name: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub disabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InsomniaParameter {
    pub name: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub disabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InsomniaBody {
    #[serde(rename = "mimeType", default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub params: Option<Vec<InsomniaBodyParam>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InsomniaBodyParam {
    pub name: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub disabled: Option<bool>,
    #[serde(rename = "fileName", default)]
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InsomniaAuth {
    #[serde(rename = "type", default)]
    pub auth_type: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(rename = "addTo", default)]
    pub add_to: Option<String>,
    #[serde(default)]
    pub disabled: Option<bool>,
}
