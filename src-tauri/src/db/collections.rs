use std::collections::HashMap;

use veyak_db::{new_id, read_yaml, write_yaml, DataDir};
use veyak_error::{AppError, AppResult};
use veyak_models::{
    ApiRequest, AuthConfig, Collection, CollectionTree, Folder, FolderNode, GraphQlRequest,
    GraphQlRequestType, GrpcMethodType, GrpcRequest, HttpMethod, RequestBody, RequestItem,
};

// ---------------------------------------------------------------------------
// Helpers — scan the workspace's collection directories
// ---------------------------------------------------------------------------

/// Find which workspace a collection belongs to, by scanning all
/// workspace dirs.  Returns `(workspace_id, collection_dir)`.
pub fn find_collection_workspace(dd: &DataDir, collection_id: &str) -> AppResult<String> {
    let ws_dir = dd.workspaces_dir();
    if !ws_dir.exists() {
        return Err(AppError::NotFound(format!("collection '{collection_id}'")));
    }
    for entry in std::fs::read_dir(&ws_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let ws_id = entry.file_name().to_string_lossy().to_string();
        let col_meta = dd.collection_meta_path(&ws_id, collection_id);
        if col_meta.exists() {
            return Ok(ws_id);
        }
    }
    Err(AppError::NotFound(format!("collection '{collection_id}'")))
}

/// Find workspace_id + collection_id for a given request_id by scanning
/// all request YAML files.
fn find_request_location(dd: &DataDir, request_id: &str) -> AppResult<(String, String)> {
    let ws_dir = dd.workspaces_dir();
    if !ws_dir.exists() {
        return Err(AppError::NotFound(format!("request '{request_id}'")));
    }
    for ws_entry in std::fs::read_dir(&ws_dir)? {
        let ws_entry = ws_entry?;
        if !ws_entry.file_type()?.is_dir() {
            continue;
        }
        let ws_id = ws_entry.file_name().to_string_lossy().to_string();
        let cols_dir = dd.collections_dir(&ws_id);
        if !cols_dir.exists() {
            continue;
        }
        for col_entry in std::fs::read_dir(&cols_dir)? {
            let col_entry = col_entry?;
            if !col_entry.file_type()?.is_dir() {
                continue;
            }
            let col_id = col_entry.file_name().to_string_lossy().to_string();
            let req_path = dd.request_path(&ws_id, &col_id, request_id);
            if req_path.exists() {
                return Ok((ws_id, col_id));
            }
        }
    }
    Err(AppError::NotFound(format!("request '{request_id}'")))
}

fn load_requests_for_collection(
    dd: &DataDir,
    workspace_id: &str,
    collection_id: &str,
) -> AppResult<Vec<RequestItem>> {
    let dir = dd.requests_dir(workspace_id, collection_id);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut requests = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
            match read_yaml(&path) {
                Ok(req) => {
                    requests.push(req);
                }
                Err(e) => {
                    eprintln!("Failed to read YAML at {:?}: {}", path, e);
                }
            }
        }
    }
    Ok(requests)
}

fn load_folders_for_collection(
    dd: &DataDir,
    workspace_id: &str,
    collection_id: &str,
) -> AppResult<Vec<Folder>> {
    let dir = dd.folders_dir(workspace_id, collection_id);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut folders = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
            let folder: Folder = read_yaml(&path)?;
            folders.push(folder);
        }
    }
    folders.sort_by_key(|f| f.sort_order);
    Ok(folders)
}

// ---------------------------------------------------------------------------
// Public API — collection trees
// ---------------------------------------------------------------------------

/// Returns all collections in a workspace (metadata only, no folders/requests loaded).
pub fn list_collections(dd: &DataDir, workspace_id: &str) -> AppResult<Vec<Collection>> {
    let cols_dir = dd.collections_dir(workspace_id);
    if !cols_dir.exists() {
        return Ok(Vec::new());
    }

    let mut collections: Vec<Collection> = Vec::new();
    for entry in std::fs::read_dir(&cols_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let col_id = entry.file_name().to_string_lossy().to_string();
        let meta_path = dd.collection_meta_path(workspace_id, &col_id);
        if meta_path.exists() {
            let col: Collection = read_yaml(&meta_path)?;
            collections.push(col);
        }
    }

    collections.sort_by_key(|c| c.sort_order);
    Ok(collections)
}

/// Returns a single collection as a fully assembled tree (folders and requests loaded).
pub fn get_collection_tree(
    dd: &DataDir,
    workspace_id: &str,
    collection_id: &str,
) -> AppResult<CollectionTree> {
    let meta_path = dd.collection_meta_path(workspace_id, collection_id);
    if !meta_path.exists() {
        return Err(AppError::NotFound(format!("collection '{collection_id}'")));
    }
    let collection: Collection = read_yaml(&meta_path)?;

    let folders = load_folders_for_collection(dd, workspace_id, collection_id)?;
    let all_requests = load_requests_for_collection(dd, workspace_id, collection_id)?;

    let mut requests_by_folder: HashMap<Option<String>, Vec<RequestItem>> = HashMap::new();
    for req in all_requests {
        requests_by_folder
            .entry(req.folder_id().map(String::from))
            .or_default()
            .push(req);
    }

    let folder_nodes = build_folder_tree(&folders, None, &mut requests_by_folder);
    let mut root_requests = requests_by_folder.remove(&None).unwrap_or_default();
    root_requests.sort_by_key(|r| r.sort_order());

    Ok(CollectionTree {
        collection,
        folders: folder_nodes,
        requests: root_requests,
    })
}

/// Returns every collection in a workspace as a fully assembled tree
/// (folders nested under folders, requests nested under both), ready for
/// the sidebar to render in one pass.
pub fn get_collection_trees(dd: &DataDir, workspace_id: &str) -> AppResult<Vec<CollectionTree>> {
    let cols_dir = dd.collections_dir(workspace_id);
    if !cols_dir.exists() {
        return Ok(Vec::new());
    }

    let mut collections: Vec<Collection> = Vec::new();
    for entry in std::fs::read_dir(&cols_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let col_id = entry.file_name().to_string_lossy().to_string();
        let meta_path = dd.collection_meta_path(workspace_id, &col_id);
        if meta_path.exists() {
            let col: Collection = read_yaml(&meta_path)?;
            collections.push(col);
        }
    }

    collections.sort_by_key(|c| c.sort_order);

    let mut trees = Vec::with_capacity(collections.len());
    for collection in collections {
        let folders = load_folders_for_collection(dd, workspace_id, &collection.id)?;
        let all_requests = load_requests_for_collection(dd, workspace_id, &collection.id)?;

        let mut requests_by_folder: HashMap<Option<String>, Vec<RequestItem>> = HashMap::new();
        for req in all_requests {
            requests_by_folder
                .entry(req.folder_id().map(String::from))
                .or_default()
                .push(req);
        }

        let folder_nodes = build_folder_tree(&folders, None, &mut requests_by_folder);
        let mut root_requests = requests_by_folder.remove(&None).unwrap_or_default();
        root_requests.sort_by_key(|r| r.sort_order());

        trees.push(CollectionTree {
            collection,
            folders: folder_nodes,
            requests: root_requests,
        });
    }
    Ok(trees)
}

fn build_folder_tree(
    all_folders: &[Folder],
    parent_id: Option<&str>,
    requests_by_folder: &mut HashMap<Option<String>, Vec<RequestItem>>,
) -> Vec<FolderNode> {
    let mut matching_folders: Vec<&Folder> = all_folders
        .iter()
        .filter(|f| f.parent_folder_id.as_deref() == parent_id)
        .collect();
    matching_folders.sort_by_key(|f| f.sort_order);

    matching_folders
        .into_iter()
        .map(|folder| {
            let children =
                build_folder_tree(all_folders, Some(folder.id.as_str()), requests_by_folder);
            let mut requests = requests_by_folder
                .remove(&Some(folder.id.clone()))
                .unwrap_or_default();
            requests.sort_by_key(|r| r.sort_order());
            FolderNode {
                folder: folder.clone(),
                children,
                requests,
            }
        })
        .collect()
}

// -- Collections ---------------------------------------------------------

pub fn create_collection(dd: &DataDir, workspace_id: &str, name: &str) -> AppResult<Collection> {
    let collection = Collection {
        id: new_id(),
        workspace_id: workspace_id.to_string(),
        name: name.to_string(),
        sort_order: 0,
        active_environment_id: None,
    };

    let col_dir = dd.collection_dir(workspace_id, &collection.id);
    std::fs::create_dir_all(&col_dir)?;
    write_yaml(
        &dd.collection_meta_path(workspace_id, &collection.id),
        &collection,
    )?;

    Ok(collection)
}

pub fn rename_collection(dd: &DataDir, id: &str, name: &str) -> AppResult<()> {
    let ws_id = find_collection_workspace(dd, id)?;
    let path = dd.collection_meta_path(&ws_id, id);
    let mut col: Collection = read_yaml(&path)?;
    col.name = name.to_string();
    write_yaml(&path, &col)
}

pub fn delete_collection(dd: &DataDir, id: &str) -> AppResult<()> {
    let ws_id = find_collection_workspace(dd, id)?;
    let col_dir = dd.collection_dir(&ws_id, id);
    if col_dir.exists() {
        std::fs::remove_dir_all(&col_dir)?;
    }
    Ok(())
}

pub fn clone_collection(dd: &DataDir, id: &str) -> AppResult<Collection> {
    let ws_id = find_collection_workspace(dd, id)?;
    let col_dir = dd.collection_dir(&ws_id, id);
    if !col_dir.exists() {
        return Err(AppError::NotFound(format!("collection '{id}'")));
    }

    let new_collection = Collection {
        id: new_id(),
        workspace_id: ws_id.clone(),
        name: format!("{} (copy)", id),
        sort_order: 0,
        active_environment_id: None,
    };
    let new_col_dir = dd.collection_dir(&ws_id, &new_collection.id);
    std::fs::create_dir_all(&new_col_dir)?;

    // Copy all files from the old collection dir to the new one
    for entry in std::fs::read_dir(&col_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap();
            std::fs::copy(&path, new_col_dir.join(file_name))?;
        }
    }

    Ok(new_collection)
}

// -- Folders --------------------------------------------------------------

pub fn create_folder(
    dd: &DataDir,
    collection_id: &str,
    parent_folder_id: Option<&str>,
    name: &str,
) -> AppResult<Folder> {
    let ws_id = find_collection_workspace(dd, collection_id)?;
    let folder = Folder {
        id: new_id(),
        collection_id: collection_id.to_string(),
        parent_folder_id: parent_folder_id.map(str::to_string),
        name: name.to_string(),
        sort_order: 0,
    };

    let folders_dir = dd.folders_dir(&ws_id, collection_id);
    std::fs::create_dir_all(&folders_dir)?;
    write_yaml(&dd.folder_path(&ws_id, collection_id, &folder.id), &folder)?;

    Ok(folder)
}

pub fn rename_folder(
    dd: &DataDir,
    collection_id: &str,
    folder_id: &str,
    name: &str,
) -> AppResult<()> {
    let ws_id = find_collection_workspace(dd, collection_id)?;
    let path = dd.folder_path(&ws_id, collection_id, folder_id);
    let mut folder: Folder = read_yaml(&path)?;
    folder.name = name.to_string();
    write_yaml(&path, &folder)
}

pub fn delete_folder(dd: &DataDir, id: &str) -> AppResult<()> {
    // We need to find the folder file and also delete any requests that
    // belong to this folder.
    let ws_dir = dd.workspaces_dir();
    if !ws_dir.exists() {
        return Ok(());
    }
    for ws_entry in std::fs::read_dir(&ws_dir)? {
        let ws_entry = ws_entry?;
        if !ws_entry.file_type()?.is_dir() {
            continue;
        }
        let ws_id = ws_entry.file_name().to_string_lossy().to_string();
        let cols_dir = dd.collections_dir(&ws_id);
        if !cols_dir.exists() {
            continue;
        }
        for col_entry in std::fs::read_dir(&cols_dir)? {
            let col_entry = col_entry?;
            if !col_entry.file_type()?.is_dir() {
                continue;
            }
            let col_id = col_entry.file_name().to_string_lossy().to_string();
            let folder_path = dd.folder_path(&ws_id, &col_id, id);
            if folder_path.exists() {
                // Delete the folder file
                std::fs::remove_file(&folder_path)?;
                // Delete requests belonging to this folder
                let reqs_dir = dd.requests_dir(&ws_id, &col_id);
                if reqs_dir.exists() {
                    for req_entry in std::fs::read_dir(&reqs_dir)? {
                        let req_entry = req_entry?;
                        let path = req_entry.path();
                        if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
                            if let Ok(req) = read_yaml::<ApiRequest>(&path) {
                                if req.folder_id.as_deref() == Some(id) {
                                    std::fs::remove_file(&path)?;
                                }
                            }
                        }
                    }
                }
                return Ok(());
            }
        }
    }
    Ok(())
}

// -- Requests ---------------------------------------------------------------

pub fn get_request(dd: &DataDir, id: &str) -> AppResult<RequestItem> {
    let (ws_id, col_id) = find_request_location(dd, id)?;
    let path = dd.request_path(&ws_id, &col_id, id);
    read_yaml(&path)
}

pub fn create_request(
    dd: &DataDir,
    collection_id: &str,
    folder_id: Option<&str>,
    name: &str,
) -> AppResult<ApiRequest> {
    let request = ApiRequest {
        id: new_id(),
        collection_id: collection_id.to_string(),
        folder_id: folder_id.map(str::to_string),
        name: name.to_string(),
        method: HttpMethod::Get,
        sort_order: 0,
        url: String::new(),
        params: vec![],
        headers: vec![],
        cookies: vec![],
        auth: AuthConfig::default(),
        body: RequestBody::default(),
    };
    let req = RequestItem::Http(request.clone());
    save_request(dd, &req)?;
    Ok(request)
}

pub fn create_ws_request(
    dd: &DataDir,
    collection_id: &str,
    folder_id: Option<&str>,
    name: &str,
) -> AppResult<ApiRequest> {
    let request = ApiRequest {
        id: new_id(),
        collection_id: collection_id.to_string(),
        folder_id: folder_id.map(str::to_string),
        name: name.to_string(),
        sort_order: 0,
        method: HttpMethod::Ws,
        url: "ws://".to_string(),
        params: vec![],
        headers: vec![],
        cookies: vec![],
        auth: AuthConfig::default(),
        body: RequestBody::default(),
    };
    let req = RequestItem::Http(request.clone());
    save_request(dd, &req)?;
    Ok(request)
}

pub fn create_grpc_request(
    dd: &DataDir,
    collection_id: &str,
    folder_id: Option<&str>,
    name: &str,
) -> AppResult<GrpcRequest> {
    let request = GrpcRequest {
        id: new_id(),
        collection_id: collection_id.to_string(),
        folder_id: folder_id.map(str::to_string),
        name: name.to_string(),
        method: "grpc".to_string(),
        method_type: GrpcMethodType::Unary,
        url: String::new(),
        sort_order: 0,
        auth: AuthConfig::default(),
        message: String::new(),
        use_reflection: false,
        proto_file_ids: Vec::new(),
        service: String::new(),
        metadata: vec![],
    };

    let req = RequestItem::Grpc(request.clone());
    save_request(dd, &req)?;
    Ok(request)
}

pub fn create_graphql_request(
    dd: &DataDir,
    collection_id: &str,
    folder_id: Option<&str>,
    name: &str,
) -> AppResult<GraphQlRequest> {
    let request = GraphQlRequest {
        id: new_id(),
        collection_id: collection_id.to_string(),
        folder_id: folder_id.map(str::to_string),
        name: name.to_string(),
        method: "GRAPHQL".to_string(),
        sort_order: 0,
        url: String::new(),
        query: "query {\n  \n}".to_string(),
        variables: String::new(),
        operation_name: None,
        headers: vec![],
        auth: AuthConfig::default(),
        request_type: GraphQlRequestType::Query,
    };

    let req = RequestItem::GraphQL(request.clone());
    save_request(dd, &req)?;
    Ok(request)
}

/// Upsert — used both to create a brand-new request row (frontend
/// generates the id client-side when a tab opens) and to save edits.
pub fn save_request(dd: &DataDir, request: &RequestItem) -> AppResult<()> {
    let ws_id = find_collection_workspace(dd, &request.collection_id())?;
    let reqs_dir = dd.requests_dir(&ws_id, &request.collection_id());
    std::fs::create_dir_all(&reqs_dir)?;
    let path = dd.request_path(&ws_id, &request.collection_id(), &request.id());
    write_yaml(&path, request)
}

pub fn delete_request(dd: &DataDir, id: &str) -> AppResult<()> {
    let (ws_id, col_id) = find_request_location(dd, id)?;
    let path = dd.request_path(&ws_id, &col_id, id);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

pub fn duplicate_request(dd: &DataDir, id: &str) -> AppResult<RequestItem> {
    let mut original = get_request(dd, id)?;
    let new_req_id = new_id();
    match &mut original {
        RequestItem::Http(req) => {
            req.id = new_req_id;
            req.name = format!("{} (copy)", req.name);
        }
        RequestItem::Grpc(req) => {
            req.id = new_req_id;
            req.name = format!("{} (copy)", req.name);
        }
        RequestItem::GraphQL(req) => {
            req.id = new_req_id;
            req.name = format!("{} (copy)", req.name);
        }
    }
    save_request(dd, &original)?;
    Ok(original)
}

pub fn rename_request(dd: &DataDir, id: &str, name: &str) -> AppResult<()> {
    let mut request = get_request(dd, id)?;
    match &mut request {
        RequestItem::Http(req) => {
            req.name = name.to_string();
        }
        RequestItem::Grpc(req) => {
            req.name = name.to_string();
        }
        RequestItem::GraphQL(req) => {
            req.name = name.to_string();
        }
    }
    save_request(dd, &request)
}

pub fn move_request(
    dd: &DataDir,
    id: &str,
    new_folder_id: Option<&str>,
    new_sort_order: i64,
) -> AppResult<()> {
    let (ws_id, col_id) = find_request_location(dd, id)?;
    let mut request = get_request(dd, id)?;
    match &mut request {
        RequestItem::Http(req) => {
            req.folder_id = new_folder_id.map(str::to_string);
            req.sort_order = new_sort_order;
        }
        RequestItem::Grpc(req) => {
            req.folder_id = new_folder_id.map(str::to_string);
            req.sort_order = new_sort_order;
        }
        RequestItem::GraphQL(req) => {
            req.folder_id = new_folder_id.map(str::to_string);
            req.sort_order = new_sort_order;
        }
    }
    save_request(dd, &request)?;

    // Reindex sibling requests in the target folder/root
    let all_requests = load_requests_for_collection(dd, &ws_id, &col_id)?;
    let mut siblings: Vec<RequestItem> = all_requests
        .into_iter()
        .filter(|r| r.folder_id() == new_folder_id)
        .collect();

    siblings.sort_by_key(|r| r.sort_order());

    for (idx, mut sib) in siblings.into_iter().enumerate() {
        let expected_order = (idx as i64) * 10;
        if sib.sort_order() != expected_order {
            match &mut sib {
                RequestItem::Http(r) => r.sort_order = expected_order,
                RequestItem::Grpc(r) => r.sort_order = expected_order,
                RequestItem::GraphQL(r) => r.sort_order = expected_order,
            }
            save_request(dd, &sib)?;
        }
    }

    Ok(())
}

pub fn move_folder(
    dd: &DataDir,
    folder_id: &str,
    new_parent_folder_id: Option<&str>,
    new_sort_order: i64,
) -> AppResult<()> {
    // Find the folder file
    let ws_dir = dd.workspaces_dir();
    if !ws_dir.exists() {
        return Err(AppError::NotFound(format!("folder '{folder_id}'")));
    }
    for ws_entry in std::fs::read_dir(&ws_dir)? {
        let ws_entry = ws_entry?;
        if !ws_entry.file_type()?.is_dir() {
            continue;
        }
        let ws_id = ws_entry.file_name().to_string_lossy().to_string();
        let cols_dir = dd.collections_dir(&ws_id);
        if !cols_dir.exists() {
            continue;
        }
        for col_entry in std::fs::read_dir(&cols_dir)? {
            let col_entry = col_entry?;
            if !col_entry.file_type()?.is_dir() {
                continue;
            }
            let col_id = col_entry.file_name().to_string_lossy().to_string();
            let folder_path = dd.folder_path(&ws_id, &col_id, folder_id);
            if folder_path.exists() {
                let mut folder: Folder = read_yaml(&folder_path)?;
                folder.parent_folder_id = new_parent_folder_id.map(str::to_string);
                folder.sort_order = new_sort_order;
                write_yaml(&folder_path, &folder)?;

                // Reindex sibling folders in the target parent/root
                let all_folders = load_folders_for_collection(dd, &ws_id, &col_id)?;
                let mut siblings: Vec<Folder> = all_folders
                    .into_iter()
                    .filter(|f| f.parent_folder_id.as_deref() == new_parent_folder_id)
                    .collect();
                siblings.sort_by_key(|f| f.sort_order);

                for (idx, mut sib) in siblings.into_iter().enumerate() {
                    let expected_order = (idx as i64) * 10;
                    if sib.sort_order != expected_order {
                        sib.sort_order = expected_order;
                        let sib_path = dd.folder_path(&ws_id, &col_id, &sib.id);
                        write_yaml(&sib_path, &sib)?;
                    }
                }

                return Ok(());
            }
        }
    }
    Err(AppError::NotFound(format!("folder '{folder_id}'")))
}
