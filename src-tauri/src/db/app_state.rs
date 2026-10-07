use veyak_db::{read_yaml_or_default, write_yaml, DataDir};
use veyak_error::AppResult;
use veyak_models::ActiveState;

pub fn get_active_state(dd: &DataDir) -> AppResult<ActiveState> {
    read_yaml_or_default(&dd.app_state_path())
}

pub fn set_active_workspace(dd: &DataDir, workspace_id: &str) -> AppResult<()> {
    let mut state = get_active_state(dd)?;
    state.active_workspace_id = Some(workspace_id.to_string());
    write_yaml(&dd.app_state_path(), &state)
}

pub fn set_active_environment(
    dd: &DataDir,
    collection_id: Option<&str>,
    environment_id: Option<&str>,
) -> AppResult<()> {
    let mut state = get_active_state(dd)?;
    state.active_environment_id = environment_id.map(str::to_string);
    if let (Some(cid), Some(eid)) = (collection_id, environment_id) {
        state
            .collection_active_environments
            .insert(cid.to_string(), eid.to_string());
    } else if let Some(cid) = collection_id {
        state.collection_active_environments.remove(cid);
    }
    write_yaml(&dd.app_state_path(), &state)
}

pub fn set_active_theme(dd: &DataDir, theme_id: &str) -> AppResult<()> {
    let mut state = get_active_state(dd)?;
    state.active_theme_id = Some(theme_id.to_string());
    write_yaml(&dd.app_state_path(), &state)
}

pub fn set_active_request(dd: &DataDir, request_id: Option<&str>) -> AppResult<()> {
    let mut state = get_active_state(dd)?;
    state.active_item_id = request_id.map(str::to_string);
    write_yaml(&dd.app_state_path(), &state)
}

pub fn set_active_collection(dd: &DataDir, collection_id: Option<&str>) -> AppResult<()> {
    let mut state = get_active_state(dd)?;
    state.active_collection_id = collection_id.map(str::to_string);
    write_yaml(&dd.app_state_path(), &state)
}
