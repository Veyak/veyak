use chrono::Utc;
use veyak_error::AppResult;
use veyak_models::{
    Collection, Environment, EnvironmentVariable, EnvironmentWithVariables, Folder, RequestItem,
    Workspace,
};

use crate::veyak::models::{
    VeyakCollectionExport, VeyakExportedCollectionTree, VeyakWorkspaceExport,
};

pub fn export_collection_as_veyak(
    collection: &Collection,
    folders: &[Folder],
    requests: &[RequestItem],
    environments: Option<&[(&Environment, &[EnvironmentVariable])]>,
) -> AppResult<String> {
    let env_with_vars = environments.map(|envs| {
        envs.iter()
            .map(|(env, vars)| EnvironmentWithVariables {
                environment: (*env).clone(),
                variables: vars.to_vec(),
            })
            .collect()
    });

    let export = VeyakCollectionExport {
        veyak_schema: 1,
        exported_at: Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        collection: collection.clone(),
        folders: folders.to_vec(),
        requests: requests.to_vec(),
        environments: env_with_vars,
    };

    let json_str = serde_json::to_string_pretty(&export)?;
    Ok(json_str)
}

pub fn export_workspace_as_veyak(
    workspace: &Workspace,
    collections_data: &[(Collection, Vec<Folder>, Vec<RequestItem>)],
    environments: &[(&Environment, &[EnvironmentVariable])],
) -> AppResult<String> {
    let collections = collections_data
        .iter()
        .map(|(col, flds, reqs)| VeyakExportedCollectionTree {
            collection: col.clone(),
            folders: flds.clone(),
            requests: reqs.clone(),
        })
        .collect();

    let envs = environments
        .iter()
        .map(|(env, vars)| EnvironmentWithVariables {
            environment: (*env).clone(),
            variables: vars.to_vec(),
        })
        .collect();

    let export = VeyakWorkspaceExport {
        veyak_schema: 1,
        exported_at: Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        workspace: workspace.clone(),
        collections,
        environments: envs,
    };

    let json_str = serde_json::to_string_pretty(&export)?;
    Ok(json_str)
}
