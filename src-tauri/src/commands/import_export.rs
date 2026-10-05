use tauri::State;
use veyak_error::AppResult;
use veyak_models::{ExportFormat, ImportFormat, ImportSummary};

use crate::state::AppState;

#[tauri::command]
pub async fn import_data(
    state: State<'_, AppState>,
    workspaceid: String,
    target_collection_id: Option<String>,
    content: String,
    format: Option<ImportFormat>,
) -> AppResult<ImportSummary> {
    let format = format.unwrap_or(ImportFormat::Auto);
    veyak_import_export::import_data(
        &state.data_dir,
        &workspaceid,
        target_collection_id.as_deref(),
        &content,
        format,
    )
}

#[tauri::command]
pub async fn import_file(
    state: State<'_, AppState>,
    workspaceid: String,
    target_collection_id: Option<String>,
    file_path: String,
    format: Option<ImportFormat>,
) -> AppResult<ImportSummary> {
    let format = format.unwrap_or(ImportFormat::Auto);
    veyak_import_export::import_file(
        &state.data_dir,
        &workspaceid,
        target_collection_id.as_deref(),
        &file_path,
        format,
    )
}

#[tauri::command]
pub async fn detect_import_format(content: String) -> AppResult<ImportFormat> {
    Ok(veyak_import_export::detect_format(&content))
}

#[tauri::command]
pub async fn export_collection(
    state: State<'_, AppState>,
    workspaceid: String,
    collectionid: String,
    format: ExportFormat,
) -> AppResult<String> {
    veyak_import_export::export_collection(
        &state.data_dir,
        &workspaceid,
        &collectionid,
        format,
    )
}

#[tauri::command]
pub async fn export_workspace(
    state: State<'_, AppState>,
    workspaceid: String,
    format: ExportFormat,
) -> AppResult<String> {
    veyak_import_export::export_workspace(
        &state.data_dir,
        &workspaceid,
        format,
    )
}

#[tauri::command]
pub async fn export_environment(
    state: State<'_, AppState>,
    workspaceid: String,
    environmentid: String,
    format: ExportFormat,
) -> AppResult<String> {
    veyak_import_export::export_environment(
        &state.data_dir,
        &workspaceid,
        &environmentid,
        format,
    )
}
