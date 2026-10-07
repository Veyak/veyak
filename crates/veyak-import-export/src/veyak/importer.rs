use veyak_error::{AppError, AppResult};

use crate::types::{ImportData, ImportedCollection, ImportedEnvironment, ImportedFolder};
use crate::veyak::models::{VeyakCollectionExport, VeyakWorkspaceExport};

pub fn import_veyak(content: &str) -> AppResult<ImportData> {
    // Try collection export first
    if let Ok(col_export) = serde_json::from_str::<VeyakCollectionExport>(content) {
        let folders = col_export
            .folders
            .into_iter()
            .map(|f| ImportedFolder {
                id: f.id,
                parent_id: f.parent_folder_id,
                name: f.name,
                sort_order: f.sort_order,
            })
            .collect();

        let environments: Vec<ImportedEnvironment> = col_export
            .environments
            .unwrap_or_default()
            .into_iter()
            .map(|e| ImportedEnvironment {
                name: e.environment.name,
                variables: e.variables,
            })
            .collect();

        return Ok(ImportData {
            collections: vec![ImportedCollection {
                name: col_export.collection.name,
                folders,
                requests: col_export.requests,
                environments: environments.clone(),
            }],
            environments,
            warnings: Vec::new(),
        });
    }

    // Try workspace export
    if let Ok(ws_export) = serde_json::from_str::<VeyakWorkspaceExport>(content) {
        let collections = ws_export
            .collections
            .into_iter()
            .map(|c| {
                let folders = c
                    .folders
                    .into_iter()
                    .map(|f| ImportedFolder {
                        id: f.id,
                        parent_id: f.parent_folder_id,
                        name: f.name,
                        sort_order: f.sort_order,
                    })
                    .collect();

                let col_envs = c
                    .environments
                    .into_iter()
                    .map(|e| ImportedEnvironment {
                        name: e.environment.name,
                        variables: e.variables,
                    })
                    .collect();

                ImportedCollection {
                    name: c.collection.name,
                    folders,
                    requests: c.requests,
                    environments: col_envs,
                }
            })
            .collect();

        let environments = ws_export
            .environments
            .into_iter()
            .map(|e| ImportedEnvironment {
                name: e.environment.name,
                variables: e.variables,
            })
            .collect();

        return Ok(ImportData {
            collections,
            environments,
            warnings: Vec::new(),
        });
    }

    Err(AppError::Invalid(
        "Failed to parse content as Veyak JSON export".to_string(),
    ))
}
