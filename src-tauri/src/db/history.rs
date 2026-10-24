use veyak_db::{new_id, now_iso, read_yaml_vec, write_yaml, DataDir};
use veyak_error::AppResult;
use veyak_models::{ApiResponse, HistoryEntry, HttpMethod};

pub fn add_entry(
    dd: &DataDir,
    request_id: Option<&str>,
    method: HttpMethod,
    name: Option<&str>,
    url: &str,
    status: u16,
    duration_ms: u128,
    _request_snapshot: Option<&str>,
    response: Option<ApiResponse>,
) -> AppResult<HistoryEntry> {
    let entry = HistoryEntry {
        id: new_id(),
        request_id: request_id.map(str::to_string),
        method,
        name: name.map(str::to_string),
        url: url.to_string(),
        status,
        duration_ms: duration_ms as i64,
        created_at: now_iso(),
        response,
    };

    let mut entries: Vec<HistoryEntry> = read_yaml_vec(&dd.history_path())?;
    entries.insert(0, entry.clone());
    write_yaml(&dd.history_path(), &entries)?;

    Ok(entry)
}

pub fn list_history(dd: &DataDir, limit: i64) -> AppResult<Vec<HistoryEntry>> {
    let entries: Vec<HistoryEntry> = read_yaml_vec(&dd.history_path())?;
    // Already stored newest-first (insert at 0), just truncate.
    Ok(entries.into_iter().take(limit as usize).collect())
}

pub fn list_request_history(
    dd: &DataDir,
    request_id: &str,
    limit: i64,
) -> AppResult<Vec<HistoryEntry>> {
    let entries: Vec<HistoryEntry> = read_yaml_vec(&dd.history_path())?;
    Ok(entries
        .into_iter()
        .filter(|e| e.request_id.as_deref() == Some(request_id))
        .take(limit as usize)
        .collect())
}

pub fn clear_history(dd: &DataDir) -> AppResult<()> {
    let empty: Vec<HistoryEntry> = Vec::new();
    write_yaml(&dd.history_path(), &empty)
}

pub fn clear_request_history(dd: &DataDir, request_id: &str) -> AppResult<()> {
    let mut entries: Vec<HistoryEntry> = read_yaml_vec(&dd.history_path())?;
    entries.retain(|entry| entry.request_id.as_deref() != Some(request_id));
    write_yaml(&dd.history_path(), &entries)
}

pub fn delete_entry(dd: &DataDir, entry_id: &str) -> AppResult<()> {
    let mut entries: Vec<HistoryEntry> = read_yaml_vec(&dd.history_path())?;
    entries.retain(|entry| entry.id != entry_id);
    write_yaml(&dd.history_path(), &entries)
}
