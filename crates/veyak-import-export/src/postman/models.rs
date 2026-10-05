use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanCollection {
    pub info: PostmanInfo,
    #[serde(default)]
    pub item: Vec<PostmanItem>,
    #[serde(default)]
    pub variable: Option<Vec<PostmanVariable>>,
    #[serde(default)]
    pub auth: Option<PostmanAuth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanInfo {
    #[serde(rename = "_postman_id", default)]
    pub postman_id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub schema: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanItem {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    /// If present, this item is a folder containing sub-items
    #[serde(default)]
    pub item: Option<Vec<PostmanItem>>,
    /// If present, this item is a request
    #[serde(default)]
    pub request: Option<PostmanRequestUnion>,
    #[serde(default)]
    pub response: Option<Vec<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PostmanRequestUnion {
    String(String),
    Object(PostmanRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanRequest {
    #[serde(default)]
    pub url: Option<PostmanUrlUnion>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub header: Option<Vec<PostmanHeader>>,
    #[serde(default)]
    pub body: Option<PostmanBody>,
    #[serde(default)]
    pub auth: Option<PostmanAuth>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PostmanUrlUnion {
    String(String),
    Object(PostmanUrl),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanUrl {
    #[serde(default)]
    pub raw: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub host: Option<Value>,
    #[serde(default)]
    pub path: Option<Value>,
    #[serde(default)]
    pub query: Option<Vec<PostmanQueryParam>>,
    #[serde(default)]
    pub variable: Option<Vec<PostmanVariable>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanHeader {
    pub key: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub disabled: Option<bool>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanQueryParam {
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub disabled: Option<bool>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanBody {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub raw: Option<String>,
    #[serde(default)]
    pub urlencoded: Option<Vec<PostmanUrlEncodedParam>>,
    #[serde(default)]
    pub formdata: Option<Vec<PostmanFormDataParam>>,
    #[serde(default)]
    pub graphql: Option<PostmanGraphQl>,
    #[serde(default)]
    pub options: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanUrlEncodedParam {
    pub key: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub disabled: Option<bool>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanFormDataParam {
    pub key: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(rename = "type", default)]
    pub param_type: Option<String>, // "text" | "file"
    #[serde(default)]
    pub src: Option<Value>,
    #[serde(default)]
    pub disabled: Option<bool>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanGraphQl {
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub variables: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanAuth {
    #[serde(rename = "type")]
    pub auth_type: String,
    #[serde(default)]
    pub bearer: Option<Vec<PostmanAuthParam>>,
    #[serde(default)]
    pub basic: Option<Vec<PostmanAuthParam>>,
    #[serde(default)]
    pub apikey: Option<Vec<PostmanAuthParam>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanAuthParam {
    pub key: String,
    #[serde(default)]
    pub value: Option<Value>,
    #[serde(rename = "type", default)]
    pub param_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanVariable {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub value: Option<Value>,
    #[serde(rename = "type", default)]
    pub var_type: Option<String>,
    #[serde(default)]
    pub disabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanEnvironment {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub values: Vec<PostmanEnvValue>,
    #[serde(rename = "_postman_variable_scope", default)]
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostmanEnvValue {
    pub key: String,
    #[serde(default)]
    pub value: Value,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(rename = "type", default)]
    pub value_type: Option<String>,
}
