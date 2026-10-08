use tauri::State;
use veyak_error::AppResult;

use crate::state::AppState;
use veyak_models::{Environment, EnvironmentVariable, EnvironmentWithVariables};

#[tauri::command]
pub async fn list_environments(
    state: State<'_, AppState>,
    workspaceid: Option<String>,
    collectionid: Option<String>,
) -> AppResult<Vec<EnvironmentWithVariables>> {
    let dd = &state.data_dir;
    let col_id = match collectionid {
        Some(cid) if !cid.is_empty() => cid,
        _ => {
            if let Some(ref wid) = workspaceid {
                let cols = crate::db::collections::list_collections(dd, wid)?;
                if let Some(first) = cols.into_iter().next() {
                    first.id
                } else {
                    return Ok(Vec::new());
                }
            } else {
                return Ok(Vec::new());
            }
        }
    };
    let ws_id = match workspaceid {
        Some(wid) if !wid.is_empty() => wid,
        _ => crate::db::collections::find_collection_workspace(dd, &col_id)?,
    };
    crate::db::environments::list_environments(dd, &ws_id, &col_id)
}

#[tauri::command]
pub async fn list_variables(
    state: State<'_, AppState>,
    workspaceid: Option<String>,
    collectionid: Option<String>,
    environmentid: String,
) -> AppResult<Vec<EnvironmentVariable>> {
    let dd = &state.data_dir;
    let (ws_id, col_id) = match (workspaceid, collectionid) {
        (Some(wid), Some(cid)) if !wid.is_empty() && !cid.is_empty() => (wid, cid),
        _ => crate::db::environments::find_environment_location(dd, &environmentid)?,
    };
    crate::db::environments::list_variables(dd, &ws_id, &col_id, &environmentid)
}

#[tauri::command]
pub async fn create_environment(
    state: State<'_, AppState>,
    workspaceid: Option<String>,
    collectionid: String,
    name: String,
) -> AppResult<Environment> {
    let dd = &state.data_dir;
    let ws_id = match workspaceid {
        Some(wid) if !wid.is_empty() => wid,
        _ => crate::db::collections::find_collection_workspace(dd, &collectionid)?,
    };
    crate::db::environments::create_environment(dd, &ws_id, &collectionid, &name)
}

#[tauri::command]
pub async fn rename_environment(
    state: State<'_, AppState>,
    environmentid: String,
    name: String,
) -> AppResult<()> {
    crate::db::environments::rename_environment(&state.data_dir, &environmentid, &name)
}

#[tauri::command]
pub async fn delete_environment(
    state: State<'_, AppState>,
    environmentid: String,
) -> AppResult<()> {
    crate::db::environments::delete_environment(&state.data_dir, &environmentid)
}

#[tauri::command]
pub async fn replace_variables(
    state: State<'_, AppState>,
    environmentid: String,
    variables: Vec<EnvironmentVariable>,
) -> AppResult<()> {
    crate::db::environments::replace_variables(&state.data_dir, &environmentid, &variables)
}

/// Persists the active environment selection to `app_state.yaml`.
/// Pass `null` / `None` from the frontend to clear the active environment.
#[tauri::command]
pub async fn set_active_environment(
    state: State<'_, AppState>,
    collectionid: Option<String>,
    environmentid: Option<String>,
) -> AppResult<()> {
    crate::db::app_state::set_active_environment(
        &state.data_dir,
        collectionid.as_deref(),
        environmentid.as_deref(),
    )
}

#[tauri::command]
pub async fn set_env_variable(
    state: State<'_, AppState>,
    environmentid: String,
    key: String,
    value: String,
) -> AppResult<()> {
    let mut map = std::collections::HashMap::new();
    map.insert(key, value);
    crate::http::apply_env_updates(&state.data_dir, &environmentid, map)
}
