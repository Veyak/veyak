use serde::{Deserialize, Serialize};
use veyak_models::{Collection, EnvironmentWithVariables, Folder, RequestItem, Workspace};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VeyakCollectionExport {
    #[serde(default = "default_veyak_schema")]
    pub veyak_schema: i64,
    pub exported_at: String,
    pub version: String,
    pub collection: Collection,
    pub folders: Vec<Folder>,
    pub requests: Vec<RequestItem>,
    #[serde(default)]
    pub environments: Option<Vec<EnvironmentWithVariables>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VeyakWorkspaceExport {
    #[serde(default = "default_veyak_schema")]
    pub veyak_schema: i64,
    pub exported_at: String,
    pub version: String,
    pub workspace: Workspace,
    pub collections: Vec<VeyakExportedCollectionTree>,
    #[serde(default)]
    pub environments: Vec<EnvironmentWithVariables>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VeyakExportedCollectionTree {
    pub collection: Collection,
    pub folders: Vec<Folder>,
    pub requests: Vec<RequestItem>,
}

fn default_veyak_schema() -> i64 {
    1
}
