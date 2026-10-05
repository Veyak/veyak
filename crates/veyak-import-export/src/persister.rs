use std::collections::HashMap;
use uuid::Uuid;
use veyak_db::{read_yaml_vec, write_yaml, DataDir};
use veyak_error::AppResult;
use veyak_models::{
    Collection, Environment, EnvironmentVariable, EnvironmentWithVariables, Folder, ImportSummary,
    RequestItem,
};

use crate::types::ImportData;

pub fn persist_import_data(
    dd: &DataDir,
    workspace_id: &str,
    target_collection_id: Option<&str>,
    data: ImportData,
) -> AppResult<ImportSummary> {
    let mut collections_count = 0;
    let mut folders_count = 0;
    let mut requests_count = 0;
    let mut environments_count = 0;
    let warnings = data.warnings;

    // Process collections
    for imported_col in data.collections {
        let (col_id, is_new_collection) = match target_collection_id {
            Some(tid) if !tid.trim().is_empty() => (tid.to_string(), false),
            _ => {
                let new_id = Uuid::new_v4().to_string();
                let col = Collection {
                    id: new_id.clone(),
                    workspace_id: workspace_id.to_string(),
                    name: imported_col.name,
                    sort_order: 0,
                };
                let meta_path = dd.collection_meta_path(workspace_id, &new_id);
                write_yaml(&meta_path, &col)?;
                collections_count += 1;
                (new_id, true)
            }
        };

        // Create folders mapping old_id -> new_id
        let mut folder_id_map: HashMap<String, String> = HashMap::new();

        for folder in imported_col.folders {
            let new_folder_id = Uuid::new_v4().to_string();
            folder_id_map.insert(folder.id.clone(), new_folder_id.clone());

            let mapped_parent = folder
                .parent_id
                .as_ref()
                .and_then(|pid| folder_id_map.get(pid).cloned());

            let folder_model = Folder {
                id: new_folder_id.clone(),
                collection_id: col_id.clone(),
                parent_folder_id: mapped_parent,
                name: folder.name,
                sort_order: folder.sort_order,
            };

            let folder_path = dd.folder_path(workspace_id, &col_id, &new_folder_id);
            write_yaml(&folder_path, &folder_model)?;
            folders_count += 1;
        }

        // Save requests
        for req in imported_col.requests {
            let new_req_id = Uuid::new_v4().to_string();
            let mapped_folder_id = req
                .folder_id()
                .and_then(|fid| folder_id_map.get(fid).cloned());

            let updated_req = update_request_item(req, new_req_id.clone(), col_id.clone(), mapped_folder_id);
            let req_path = dd.request_path(workspace_id, &col_id, &new_req_id);
            write_yaml(&req_path, &updated_req)?;
            requests_count += 1;
        }

        if !is_new_collection && collections_count == 0 {
            // Target collection was used
            collections_count = 1;
        }
    }

    // Process environments
    if !data.environments.is_empty() {
        let envs_path = dd.environments_path(workspace_id);
        let mut existing_envs: Vec<EnvironmentWithVariables> = read_yaml_vec(&envs_path)?;

        for imported_env in data.environments {
            let new_env_id = Uuid::new_v4().to_string();
            let env_model = Environment {
                id: new_env_id.clone(),
                workspace_id: workspace_id.to_string(),
                name: imported_env.name,
                sort_order: existing_envs.len() as i64,
            };

            let variables = imported_env
                .variables
                .into_iter()
                .map(|mut v| {
                    v.id = Uuid::new_v4().to_string();
                    v.environmentid = new_env_id.clone();
                    v
                })
                .collect::<Vec<EnvironmentVariable>>();

            existing_envs.push(EnvironmentWithVariables {
                environment: env_model,
                variables,
            });
            environments_count += 1;
        }

        write_yaml(&envs_path, &existing_envs)?;
    }

    Ok(ImportSummary {
        collections_count,
        folders_count,
        requests_count,
        environments_count,
        warnings,
    })
}

fn update_request_item(
    mut req: RequestItem,
    new_id: String,
    collection_id: String,
    folder_id: Option<String>,
) -> RequestItem {
    match &mut req {
        RequestItem::Http(r) => {
            r.id = new_id;
            r.collection_id = collection_id;
            r.folder_id = folder_id;
        }
        RequestItem::Grpc(r) => {
            r.id = new_id;
            r.collection_id = collection_id;
            r.folder_id = folder_id;
        }
        RequestItem::GraphQL(r) => {
            r.id = new_id;
            r.collection_id = collection_id;
            r.folder_id = folder_id;
        }
    }
    req
}
