use veyak_models::{EnvironmentVariable, RequestItem};

/// Clean intermediate data representation resulting from an import parser,
/// before being written to disk or merged into an existing workspace.
#[derive(Debug, Clone, Default)]
pub struct ImportData {
    pub collections: Vec<ImportedCollection>,
    pub environments: Vec<ImportedEnvironment>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ImportedCollection {
    pub name: String,
    pub folders: Vec<ImportedFolder>,
    pub requests: Vec<RequestItem>,
}

#[derive(Debug, Clone)]
pub struct ImportedFolder {
    /// Temporary/original ID used to reconstruct folder parent-child hierarchies.
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone)]
pub struct ImportedEnvironment {
    pub name: String,
    pub variables: Vec<EnvironmentVariable>,
}
