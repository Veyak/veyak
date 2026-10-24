use crate::state::AppState;
use tauri::State;
use veyak_error::AppResult;
use veyak_models::HistoryEntry;

#[tauri::command]
pub async fn list_history(state: State<'_, AppState>, limit: i64) -> AppResult<Vec<HistoryEntry>> {
    crate::db::history::list_history(&state.data_dir, limit)
}

#[tauri::command]
pub async fn list_request_history(
    state: State<'_, AppState>,
    request_id: String,
    limit: i64,
) -> AppResult<Vec<HistoryEntry>> {
    crate::db::history::list_request_history(&state.data_dir, &request_id, limit)
}

#[tauri::command]
pub async fn clear_history(state: State<'_, AppState>) -> AppResult<()> {
    crate::db::history::clear_history(&state.data_dir)
}

#[tauri::command]
pub async fn clear_request_history(
    state: State<'_, AppState>,
    request_id: String,
) -> AppResult<()> {
    crate::db::history::clear_request_history(&state.data_dir, &request_id)
}

#[tauri::command]
pub async fn delete_history_entry(state: State<'_, AppState>, id: String) -> AppResult<()> {
    crate::db::history::delete_entry(&state.data_dir, &id)
}
