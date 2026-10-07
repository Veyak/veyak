use std::collections::HashMap;

use veyak_db::{new_id, read_yaml_vec, write_yaml, DataDir};
use veyak_error::AppResult;
use veyak_models::{Environment, EnvironmentVariable, EnvironmentWithVariables};

pub fn list_environments(
    dd: &DataDir,
    workspace_id: &str,
    collection_id: &str,
) -> AppResult<Vec<EnvironmentWithVariables>> {
    migrate_legacy_environments_if_needed(dd, workspace_id, collection_id);
    let path = dd.collection_environments_path(workspace_id, collection_id);
    let mut envs: Vec<EnvironmentWithVariables> = read_yaml_vec(&path)?;
    envs.sort_by_key(|e| e.environment.sort_order);
    Ok(envs)
}

pub fn list_variables(
    dd: &DataDir,
    workspace_id: &str,
    collection_id: &str,
    environment_id: &str,
) -> AppResult<Vec<EnvironmentVariable>> {
    let envs = list_environments(dd, workspace_id, collection_id)?;
    Ok(envs
        .into_iter()
        .find(|e| e.environment.id == environment_id)
        .map(|e| e.variables)
        .unwrap_or_default())
}

/// Find which (workspace_id, collection_id) an environment belongs to by
/// scanning all collections.
pub fn find_environment_location(
    dd: &DataDir,
    environment_id: &str,
) -> AppResult<(String, String)> {
    let ws_dir = dd.workspaces_dir();
    if ws_dir.exists() {
        for ws_entry in std::fs::read_dir(&ws_dir)? {
            let ws_entry = ws_entry?;
            if !ws_entry.file_type()?.is_dir() {
                continue;
            }
            let ws_id = ws_entry.file_name().to_string_lossy().to_string();
            let cols_dir = dd.collections_dir(&ws_id);
            if cols_dir.exists() {
                for col_entry in std::fs::read_dir(&cols_dir)? {
                    let col_entry = col_entry?;
                    if !col_entry.file_type()?.is_dir() {
                        continue;
                    }
                    let col_id = col_entry.file_name().to_string_lossy().to_string();
                    let envs_path = dd.collection_environments_path(&ws_id, &col_id);
                    if envs_path.exists() {
                        let envs: Vec<EnvironmentWithVariables> = read_yaml_vec(&envs_path)?;
                        if envs.iter().any(|e| e.environment.id == environment_id) {
                            return Ok((ws_id, col_id));
                        }
                    }
                }
            }

            // Also check legacy path and migrate if needed
            let legacy_path = dd.environments_path(&ws_id);
            if legacy_path.exists() {
                if let Ok(envs) = read_yaml_vec::<EnvironmentWithVariables>(&legacy_path) {
                    if envs.iter().any(|e| e.environment.id == environment_id) {
                        // Find first collection to associate with
                        if cols_dir.exists() {
                            if let Ok(mut entries) = std::fs::read_dir(&cols_dir) {
                                if let Some(Ok(col_entry)) = entries.next() {
                                    let col_id =
                                        col_entry.file_name().to_string_lossy().to_string();
                                    migrate_legacy_environments_if_needed(dd, &ws_id, &col_id);
                                    return Ok((ws_id, col_id));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Err(veyak_error::AppError::NotFound(format!(
        "environment '{environment_id}'"
    )))
}

pub fn create_environment(
    dd: &DataDir,
    workspace_id: &str,
    collection_id: &str,
    name: &str,
) -> AppResult<Environment> {
    let path = dd.collection_environments_path(workspace_id, collection_id);
    let mut envs: Vec<EnvironmentWithVariables> = read_yaml_vec(&path)?;
    let environment = Environment {
        id: new_id(),
        collection_id: collection_id.to_string(),
        name: name.to_string(),
        sort_order: envs.len() as i64,
    };

    envs.push(EnvironmentWithVariables {
        environment: environment.clone(),
        variables: Vec::new(),
    });
    write_yaml(&path, &envs)?;

    Ok(environment)
}

pub fn rename_environment(dd: &DataDir, id: &str, name: &str) -> AppResult<()> {
    let (ws_id, col_id) = find_environment_location(dd, id)?;
    let path = dd.collection_environments_path(&ws_id, &col_id);
    let mut envs: Vec<EnvironmentWithVariables> = read_yaml_vec(&path)?;
    if let Some(env) = envs.iter_mut().find(|e| e.environment.id == id) {
        env.environment.name = name.to_string();
    }
    write_yaml(&path, &envs)
}

pub fn delete_environment(dd: &DataDir, id: &str) -> AppResult<()> {
    let (ws_id, col_id) = find_environment_location(dd, id)?;
    let path = dd.collection_environments_path(&ws_id, &col_id);
    let mut envs: Vec<EnvironmentWithVariables> = read_yaml_vec(&path)?;
    envs.retain(|e| e.environment.id != id);
    write_yaml(&path, &envs)
}

/// Replaces every variable for an environment with the given set — the
/// frontend always edits the whole variable table at once (see the
/// Environment editor), so a clear-then-insert is simpler and avoids
/// reconciling row-level diffs.
pub fn replace_variables(
    dd: &DataDir,
    environment_id: &str,
    variables: &[EnvironmentVariable],
) -> AppResult<()> {
    let (ws_id, col_id) = find_environment_location(dd, environment_id)?;
    let path = dd.collection_environments_path(&ws_id, &col_id);
    let mut envs: Vec<EnvironmentWithVariables> = read_yaml_vec(&path)?;

    if let Some(env) = envs.iter_mut().find(|e| e.environment.id == environment_id) {
        env.variables = variables
            .iter()
            .enumerate()
            .map(|(_i, v)| {
                let mut var = v.clone();
                if var.id.is_empty() {
                    var.id = new_id();
                }
                var.environmentid = environment_id.to_string();
                var
            })
            .collect();
    }

    write_yaml(&path, &envs)
}

/// Migrates any legacy workspace-level `environments.yaml` to the specified
/// collection's `environments.yaml` if the collection currently has none.
pub fn migrate_legacy_environments_if_needed(
    dd: &DataDir,
    workspace_id: &str,
    collection_id: &str,
) {
    let legacy_path = dd.environments_path(workspace_id);
    let col_path = dd.collection_environments_path(workspace_id, collection_id);
    if legacy_path.exists() && !col_path.exists() {
        if let Ok(mut legacy_envs) = read_yaml_vec::<EnvironmentWithVariables>(&legacy_path) {
            if !legacy_envs.is_empty() {
                for env in &mut legacy_envs {
                    env.environment.collection_id = collection_id.to_string();
                }
                let _ = write_yaml(&col_path, &legacy_envs);
                let _ = std::fs::remove_file(&legacy_path);
            }
        }
    }
}

/// Loads the active environment's enabled variables as a flat map, ready
/// for `interpolate`. Returns an empty map if no environment is active.
pub fn active_variable_map(
    dd: &DataDir,
    active_environment_id: Option<&str>,
) -> AppResult<HashMap<String, String>> {
    let Some(env_id) = active_environment_id else {
        return Ok(HashMap::new());
    };
    let (ws_id, col_id) = match find_environment_location(dd, env_id) {
        Ok(loc) => loc,
        Err(_) => return Ok(HashMap::new()),
    };
    let vars = list_variables(dd, &ws_id, &col_id, env_id)?;
    Ok(vars
        .into_iter()
        .filter(|v| v.enabled)
        .map(|v| (v.key, v.value))
        .collect())
}

/// Replaces every `{{key}}` placeholder in `input` with the matching
/// variable value. Unmatched placeholders are left as-is rather than
/// silently emptied, so a typo'd variable name is obvious in the sent
/// request instead of producing an empty string.
pub fn interpolate(input: &str, variables: &HashMap<String, String>) -> String {
    if !input.contains("{{") {
        return input.to_string();
    }

    let mut output = String::with_capacity(input.len());
    let mut rest = input;

    while let Some(start) = rest.find("{{") {
        output.push_str(&rest[..start]);
        let after_start = &rest[start + 2..];

        match after_start.find("}}") {
            Some(end) => {
                let key = after_start[..end].trim();
                match variables.get(key) {
                    Some(value) => output.push_str(value),
                    None => {
                        output.push_str("{{");
                        output.push_str(key);
                        output.push_str("}}");
                    }
                }
                rest = &after_start[end + 2..];
            }
            None => {
                // Unterminated `{{` — treat the rest of the string literally.
                output.push_str("{{");
                rest = after_start;
                break;
            }
        }
    }

    output.push_str(rest);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_known_keys_and_preserves_unknown_ones() {
        let mut vars = HashMap::new();
        vars.insert(
            "base_url".to_string(),
            "https://api.example.com".to_string(),
        );

        let result = interpolate("{{base_url}}/users/{{user_id}}", &vars);
        assert_eq!(result, "https://api.example.com/users/{{user_id}}");
    }

    #[test]
    fn leaves_plain_strings_untouched() {
        let vars = HashMap::new();
        assert_eq!(
            interpolate("no placeholders here", &vars),
            "no placeholders here"
        );
    }
}
