use tauri::State;
use veyak_error::AppResult;

use crate::state::AppState;
use veyak_models::{AdditionType, ApiRequest, Collection, CollectionTree, Folder, RequestItem};

#[tauri::command]
pub async fn get_addition_types() -> AppResult<Vec<AdditionType>> {
    Ok(vec![
        AdditionType {
            id: "request".into(),
            label: "Add HTTP Request".into(),
            icon: "FilePlus".into(),
        },
        AdditionType {
            id: "ws".into(),
            label: "Add WebSocket".into(),
            icon: "FilePlus".into(),
        },
        AdditionType {
            id: "grpc".into(),
            label: "Add gRPC Request".into(),
            icon: "FilePlus".into(),
        },
        AdditionType {
            id: "graphql".into(),
            label: "Add GraphQL Request".into(),
            icon: "FilePlus".into(),
        },
        AdditionType {
            id: "folder".into(),
            label: "Add Folder".into(),
            icon: "FolderPlus".into(),
        },
    ])
}

#[tauri::command]
pub async fn list_collections(
    state: State<'_, AppState>,
    workspaceid: String,
) -> AppResult<Vec<Collection>> {
    crate::db::collections::list_collections(&state.data_dir, &workspaceid)
}

#[tauri::command]
pub async fn get_collection_tree(
    state: State<'_, AppState>,
    workspaceid: String,
    collectionid: String,
) -> AppResult<CollectionTree> {
    crate::db::collections::get_collection_tree(&state.data_dir, &workspaceid, &collectionid)
}

#[tauri::command]
pub async fn get_collection_trees(
    state: State<'_, AppState>,
    workspaceid: String,
) -> AppResult<Vec<CollectionTree>> {
    crate::db::collections::get_collection_trees(&state.data_dir, &workspaceid)
}

#[tauri::command]
pub async fn create_collection(
    state: State<'_, AppState>,
    workspaceid: String,
    name: String,
) -> AppResult<Collection> {
    println!(
        "Creating collection '{}' in workspace '{}'",
        name, workspaceid
    );
    crate::db::collections::create_collection(&state.data_dir, &workspaceid, &name)
}

#[tauri::command]
pub async fn rename_collection(
    state: State<'_, AppState>,
    collectionid: String,
    name: String,
) -> AppResult<()> {
    crate::db::collections::rename_collection(&state.data_dir, &collectionid, &name)
}

#[tauri::command]
pub async fn delete_collection(state: State<'_, AppState>, collectionid: String) -> AppResult<()> {
    crate::db::collections::delete_collection(&state.data_dir, &collectionid)
}

#[tauri::command]
pub async fn clone_collection(
    state: State<'_, AppState>,
    collectionid: String,
) -> AppResult<Collection> {
    crate::db::collections::clone_collection(&state.data_dir, &collectionid)
}

#[tauri::command]
pub async fn create_folder(
    state: State<'_, AppState>,
    collectionid: String,
    parentfolderid: Option<String>,
    name: String,
) -> AppResult<Folder> {
    println!("Creating folder '{}' in workspace '{}'", name, collectionid);
    // parentfolderid: Option<&str>
    crate::db::collections::create_folder(
        &state.data_dir,
        &collectionid,
        parentfolderid.as_deref(),
        &name,
    )
}

#[tauri::command]
pub async fn delete_folder(state: State<'_, AppState>, folderid: String) -> AppResult<()> {
    crate::db::collections::delete_folder(&state.data_dir, &folderid)
}

#[tauri::command]
pub async fn rename_folder(
    state: State<'_, AppState>,
    collectionid: String,
    folderid: String,
    name: String,
) -> AppResult<()> {
    crate::db::collections::rename_folder(&state.data_dir, &collectionid, &folderid, &name)
}

#[tauri::command]
pub async fn get_request(state: State<'_, AppState>, requestid: String) -> AppResult<RequestItem> {
    crate::db::collections::get_request(&state.data_dir, &requestid)
}

#[tauri::command]
pub async fn create_request(
    state: State<'_, AppState>,
    collectionid: String,
    folderid: Option<String>,
    name: String,
    reqtype: String,
) -> AppResult<RequestItem> {
    let request_item: RequestItem = match reqtype.to_lowercase().as_str() {
        "grpc" => {
            let req = crate::db::collections::create_grpc_request(
                &state.data_dir,
                &collectionid,
                folderid.as_deref(),
                &name,
            )?;
            RequestItem::Grpc(req)
        }
        "graphql" => {
            let req = crate::db::collections::create_graphql_request(
                &state.data_dir,
                &collectionid,
                folderid.as_deref(),
                &name,
            )?;
            RequestItem::GraphQL(req)
        }
        "rest" => {
            let req = crate::db::collections::create_request(
                &state.data_dir,
                &collectionid,
                folderid.as_deref(),
                &name,
            )?;
            RequestItem::Http(req)
        }
        "ws" => {
            let req = crate::db::collections::create_ws_request(
                &state.data_dir,
                &collectionid,
                folderid.as_deref(),
                &name,
            )?;
            RequestItem::Http(req)
        }
        _ => {
            let req = crate::db::collections::create_request(
                &state.data_dir,
                &collectionid,
                folderid.as_deref(),
                &name,
            )?;
            RequestItem::Http(req)
        }
    };
    Ok(request_item)
}

#[tauri::command]
pub async fn create_ws_request(
    state: State<'_, AppState>,
    collectionid: String,
    folderid: Option<String>,
    name: String,
) -> AppResult<ApiRequest> {
    crate::db::collections::create_ws_request(
        &state.data_dir,
        &collectionid,
        folderid.as_deref(),
        &name,
    )
}

#[tauri::command]
pub async fn delete_request(state: State<'_, AppState>, requestid: String) -> AppResult<()> {
    crate::db::collections::delete_request(&state.data_dir, &requestid)
}

#[tauri::command]
pub async fn rename_request(
    state: State<'_, AppState>,
    requestid: String,
    name: String,
) -> AppResult<()> {
    crate::db::collections::rename_request(&state.data_dir, &requestid, &name)
}

#[tauri::command]
pub async fn duplicate_request(
    state: State<'_, AppState>,
    requestid: String,
) -> AppResult<RequestItem> {
    crate::db::collections::duplicate_request(&state.data_dir, &requestid)
}

/// Saves a request and marks it as the active request.
///
/// # Examples
///
/// ```ignore
/// save_request(state, request).await?;
/// ```
#[tauri::command]
pub async fn save_request(state: State<'_, AppState>, request: RequestItem) -> AppResult<()> {
    crate::db::collections::save_request(&state.data_dir, &request)?;
    crate::db::app_state::set_active_request(&state.data_dir, Some(&request.id()))?;
    Ok(())
}

#[tauri::command]
pub async fn set_active_collection(
    state: State<'_, AppState>,
    collectionid: Option<String>,
) -> AppResult<()> {
    crate::db::app_state::set_active_collection(&state.data_dir, collectionid.as_deref())
}

#[tauri::command]
pub async fn move_request(
    status: State<'_, AppState>,
    requestid: String,
    target_folder_id: Option<String>,
    new_sort_order: i64,
) -> AppResult<()> {
    crate::db::collections::move_request(
        &status.data_dir,
        &requestid,
        target_folder_id.as_deref(),
        new_sort_order,
    )
}

#[tauri::command]
pub async fn move_folder(
    status: State<'_, AppState>,
    folderid: String,
    target_folder_id: Option<String>,
    new_sort_order: i64,
) -> AppResult<()> {
    crate::db::collections::move_folder(
        &status.data_dir,
        &folderid,
        target_folder_id.as_deref(),
        new_sort_order,
    )
}
