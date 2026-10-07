pub mod detector;
pub mod insomnia;
pub mod persister;
pub mod postman;
pub mod types;
pub mod veyak;
pub mod yaak;

pub use detector::detect_format;
pub use persister::persist_import_data;
pub use types::*;

use std::path::Path;
use veyak_db::{DataDir, read_yaml, read_yaml_vec};
use veyak_error::{AppError, AppResult};
use veyak_models::{
    Collection, Environment, EnvironmentVariable, EnvironmentWithVariables, ExportFormat, Folder,
    ImportFormat, ImportSummary, RequestItem, Workspace,
};

/// Parse the raw import content based on the requested format or auto-detection.
pub fn parse_import_content(content: &str, format: ImportFormat) -> AppResult<ImportData> {
    let resolved_format = match format {
        ImportFormat::Auto => {
            let detected = detect_format(content);
            if detected == ImportFormat::Auto {
                // If auto-detection was unable to find specific markers,
                // try Postman, then Insomnia, then Yaak in sequence
                if let Ok(data) = postman::import_postman(content) {
                    return Ok(data);
                }
                if let Ok(data) = insomnia::import_insomnia(content) {
                    return Ok(data);
                }
                if let Ok(data) = yaak::import_yaak(content) {
                    return Ok(data);
                }
                if let Ok(data) = veyak::import_veyak(content) {
                    return Ok(data);
                }
                return Err(AppError::Invalid(
                    "Unable to automatically detect the import format. Please specify the format explicitly.".to_string(),
                ));
            }
            detected
        }
        other => other,
    };

    match resolved_format {
        ImportFormat::Postman => postman::import_postman(content),
        ImportFormat::PostmanEnvironment => postman::import_postman(content),
        ImportFormat::Insomnia => insomnia::import_insomnia(content),
        ImportFormat::Yaak => yaak::import_yaak(content),
        ImportFormat::Veyak => veyak::import_veyak(content),
        ImportFormat::Auto => unreachable!(),
    }
}

/// Import content into a workspace and persist to disk.
pub fn import_data(
    dd: &DataDir,
    workspace_id: &str,
    target_collection_id: Option<&str>,
    content: &str,
    format: ImportFormat,
) -> AppResult<ImportSummary> {
    let parsed_data = parse_import_content(content, format)?;
    persist_import_data(dd, workspace_id, target_collection_id, parsed_data)
}

/// Import file from disk into a workspace and persist.
pub fn import_file(
    dd: &DataDir,
    workspace_id: &str,
    target_collection_id: Option<&str>,
    file_path: &str,
    format: ImportFormat,
) -> AppResult<ImportSummary> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(AppError::NotFound(format!("File at '{file_path}'")));
    }
    let content = std::fs::read_to_string(path)?;
    import_data(dd, workspace_id, target_collection_id, &content, format)
}

/// Export a collection to the desired format as a string.
pub fn export_collection(
    dd: &DataDir,
    workspace_id: &str,
    collection_id: &str,
    format: ExportFormat,
) -> AppResult<String> {
    let (collection, folders, requests) = load_collection_data(dd, workspace_id, collection_id)?;
    let environments = load_collection_environments_data(dd, workspace_id, collection_id)?;
    let env_refs: Vec<(&Environment, &[EnvironmentVariable])> = environments
        .iter()
        .map(|e| (&e.environment, e.variables.as_slice()))
        .collect();

    match format {
        ExportFormat::Postman => {
            postman::export_collection_as_postman(&collection, &folders, &requests, Some(&env_refs))
        }
        ExportFormat::Insomnia => insomnia::export_collection_as_insomnia(
            &collection,
            &folders,
            &requests,
            Some(&env_refs),
        ),
        ExportFormat::Yaak => {
            yaak::export_collection_as_yaak(&collection, &folders, &requests, Some(&env_refs))
        }
        ExportFormat::Veyak => {
            veyak::export_collection_as_veyak(&collection, &folders, &requests, Some(&env_refs))
        }
    }
}

/// Export an entire workspace to the desired format as a string.
pub fn export_workspace(
    dd: &DataDir,
    workspace_id: &str,
    format: ExportFormat,
) -> AppResult<String> {
    let ws_path = dd.workspace_meta_path(workspace_id);
    let workspace: Workspace = read_yaml(&ws_path)?;

    let cols_dir = dd.collections_dir(workspace_id);
    let mut collections_data = Vec::new();

    if cols_dir.exists() {
        for entry in std::fs::read_dir(&cols_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let col_id = entry.file_name().to_string_lossy().to_string();
                if let Ok(data) = load_collection_data(dd, workspace_id, &col_id) {
                    collections_data.push(data);
                }
            }
        }
    }

    let environments = load_environments_data(dd, workspace_id)?;
    let env_refs: Vec<(&Environment, &[EnvironmentVariable])> = environments
        .iter()
        .map(|e| (&e.environment, e.variables.as_slice()))
        .collect();

    match format {
        ExportFormat::Veyak => {
            veyak::export_workspace_as_veyak(&workspace, &collections_data, &env_refs)
        }
        ExportFormat::Postman => {
            // For Postman, if multiple collections exist, export the first or wrap them
            if let Some((col, folders, requests)) = collections_data.first() {
                postman::export_collection_as_postman(col, folders, requests, Some(&env_refs))
            } else {
                let dummy = Collection {
                    id: workspace.id.clone(),
                    workspace_id: workspace.id.clone(),
                    name: workspace.name.clone(),
                    sort_order: 0,
                    active_environment_id: None,
                };
                postman::export_collection_as_postman(&dummy, &[], &[], None)
            }
        }
        ExportFormat::Insomnia => {
            // For Insomnia, create a workspace export with all collections' items
            let dummy = Collection {
                id: workspace.id.clone(),
                workspace_id: workspace.id.clone(),
                name: workspace.name.clone(),
                sort_order: 0,
                active_environment_id: None,
            };
            let mut all_folders = Vec::new();
            let mut all_requests = Vec::new();
            for (_, folders, requests) in collections_data {
                all_folders.extend(folders);
                all_requests.extend(requests);
            }
            insomnia::export_collection_as_insomnia(
                &dummy,
                &all_folders,
                &all_requests,
                Some(&env_refs),
            )
        }
        ExportFormat::Yaak => {
            let dummy = Collection {
                id: workspace.id.clone(),
                workspace_id: workspace.id.clone(),
                name: workspace.name.clone(),
                sort_order: 0,
                active_environment_id: None,
            };
            let mut all_folders = Vec::new();
            let mut all_requests = Vec::new();
            for (_, folders, requests) in collections_data {
                all_folders.extend(folders);
                all_requests.extend(requests);
            }
            yaak::export_collection_as_yaak(&dummy, &all_folders, &all_requests, Some(&env_refs))
        }
    }
}

/// Export a single environment as a string.
pub fn export_environment(
    dd: &DataDir,
    workspace_id: &str,
    environment_id: &str,
    format: ExportFormat,
) -> AppResult<String> {
    let environments = load_environments_data(dd, workspace_id)?;
    let target = environments
        .iter()
        .find(|e| e.environment.id == environment_id)
        .ok_or_else(|| AppError::NotFound(format!("Environment '{environment_id}'")))?;

    match format {
        ExportFormat::Postman => {
            postman::export_environment_as_postman(&target.environment, &target.variables)
        }
        ExportFormat::Insomnia => {
            let dummy_col = Collection {
                id: target.environment.id.clone(),
                workspace_id: workspace_id.to_string(),
                name: target.environment.name.clone(),
                sort_order: 0,
                active_environment_id: None,
            };
            let env_ref = [(&target.environment, target.variables.as_slice())];
            insomnia::export_collection_as_insomnia(&dummy_col, &[], &[], Some(&env_ref))
        }
        ExportFormat::Yaak => {
            let dummy_col = Collection {
                id: target.environment.id.clone(),
                workspace_id: workspace_id.to_string(),
                name: target.environment.name.clone(),
                sort_order: 0,
                active_environment_id: None,
            };
            let env_ref = [(&target.environment, target.variables.as_slice())];
            yaak::export_collection_as_yaak(&dummy_col, &[], &[], Some(&env_ref))
        }
        ExportFormat::Veyak => {
            let dummy_col = Collection {
                id: target.environment.id.clone(),
                workspace_id: workspace_id.to_string(),
                name: target.environment.name.clone(),
                sort_order: 0,
                active_environment_id: None,
            };
            let env_ref = [(&target.environment, target.variables.as_slice())];
            veyak::export_collection_as_veyak(&dummy_col, &[], &[], Some(&env_ref))
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn load_collection_data(
    dd: &DataDir,
    workspace_id: &str,
    collection_id: &str,
) -> AppResult<(Collection, Vec<Folder>, Vec<RequestItem>)> {
    let meta_path = dd.collection_meta_path(workspace_id, collection_id);
    if !meta_path.exists() {
        return Err(AppError::NotFound(format!("collection '{collection_id}'")));
    }
    let collection: Collection = read_yaml(&meta_path)?;

    // Load folders
    let folders_dir = dd.folders_dir(workspace_id, collection_id);
    let mut folders = Vec::new();
    if folders_dir.exists() {
        for entry in std::fs::read_dir(&folders_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
                if let Ok(folder) = read_yaml::<Folder>(&path) {
                    folders.push(folder);
                }
            }
        }
    }
    folders.sort_by_key(|f| f.sort_order);

    // Load requests
    let reqs_dir = dd.requests_dir(workspace_id, collection_id);
    let mut requests = Vec::new();
    if reqs_dir.exists() {
        for entry in std::fs::read_dir(&reqs_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
                if let Ok(req) = read_yaml::<RequestItem>(&path) {
                    requests.push(req);
                }
            }
        }
    }
    requests.sort_by_key(|r| r.sort_order());

    Ok((collection, folders, requests))
}

fn load_collection_environments_data(
    dd: &DataDir,
    workspace_id: &str,
    collection_id: &str,
) -> AppResult<Vec<EnvironmentWithVariables>> {
    let col_envs_path = dd.collection_environments_path(workspace_id, collection_id);
    if col_envs_path.exists() {
        return read_yaml_vec(&col_envs_path);
    }
    // Fallback: check legacy workspace environments path
    let legacy_path = dd.environments_path(workspace_id);
    if legacy_path.exists() {
        return read_yaml_vec(&legacy_path);
    }
    Ok(Vec::new())
}

fn load_environments_data(
    dd: &DataDir,
    workspace_id: &str,
) -> AppResult<Vec<EnvironmentWithVariables>> {
    let mut all_envs = Vec::new();
    let cols_dir = dd.collections_dir(workspace_id);
    if cols_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&cols_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let col_id = entry.file_name().to_string_lossy().to_string();
                    let path = dd.collection_environments_path(workspace_id, &col_id);
                    if path.exists() {
                        if let Ok(envs) = read_yaml_vec::<EnvironmentWithVariables>(&path) {
                            all_envs.extend(envs);
                        }
                    }
                }
            }
        }
    }
    if all_envs.is_empty() {
        let legacy_path = dd.environments_path(workspace_id);
        if legacy_path.exists() {
            if let Ok(envs) = read_yaml_vec::<EnvironmentWithVariables>(&legacy_path) {
                all_envs = envs;
            }
        }
    }
    Ok(all_envs)
}
