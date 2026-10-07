use crate::insomnia::v5_importer::is_insomnia_v5_type_str;
use serde_json::Value;
use veyak_models::ImportFormat;

/// Automatically detect the import format from string content.
pub fn detect_format(content: &str) -> ImportFormat {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return ImportFormat::Auto;
    }

    // First attempt JSON parsing
    if let Ok(json) = serde_json::from_str::<Value>(trimmed) {
        if let Some(format) = detect_from_json(&json) {
            return format;
        }
    }

    // If JSON parsing fails, try YAML (Insomnia and Veyak support YAML exports)
    if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(trimmed) {
        if let Some(format) = detect_from_yaml(&yaml) {
            return format;
        }
    }

    ImportFormat::Auto
}

fn detect_from_json(json: &Value) -> Option<ImportFormat> {
    // 1. Check Veyak Native format
    if json.get("veyakSchema").is_some() || json.get("veyakVersion").is_some() {
        return Some(ImportFormat::Veyak);
    }

    // 2. Check Yaak export
    if json.get("yaakSchema").is_some() || json.get("YaakSchema").is_some() {
        return Some(ImportFormat::Yaak);
    }
    if let Some(res) = json.get("resources") {
        if res.is_object()
            && (res.get("httpRequests").is_some()
                || res.get("workspaces").is_some()
                || res.get("grpcRequests").is_some())
        {
            return Some(ImportFormat::Yaak);
        }
    }

    // 3. Check Insomnia export (v4 and v5)
    if json.get("__export_format").is_some()
        || json.get("_type").and_then(|v| v.as_str()) == Some("export")
    {
        return Some(ImportFormat::Insomnia);
    }
    if let Some(res) = json.get("resources").and_then(|r| r.as_array()) {
        if res.iter().any(|item| {
            item.get("_type")
                .and_then(|t| t.as_str())
                .map(|t| t == "workspace" || t == "request" || t == "request_group")
                .unwrap_or(false)
        }) {
            return Some(ImportFormat::Insomnia);
        }
    }
    if let Some(t) = json.get("type").and_then(|v| v.as_str()) {
        if is_insomnia_v5_type_str(t) {
            return Some(ImportFormat::Insomnia);
        }
    }
    if json.get("schema_version").is_some()
        && (json.get("collection").is_some()
            || json.get("environments").is_some()
            || json.get("routes").is_some())
    {
        return Some(ImportFormat::Insomnia);
    }

    // 4. Check Postman Environment
    if json.get("_postman_variable_scope").is_some() {
        return Some(ImportFormat::PostmanEnvironment);
    }
    if let Some(values) = json.get("values").and_then(|v| v.as_array()) {
        if json.get("name").is_some()
            && values
                .iter()
                .any(|v| v.get("key").is_some() && v.get("value").is_some())
        {
            return Some(ImportFormat::PostmanEnvironment);
        }
    }

    // 5. Check Postman Collection
    if let Some(info) = json.get("info") {
        if info.get("_postman_id").is_some()
            || info
                .get("schema")
                .and_then(|s| s.as_str())
                .map(|s| s.contains("postman.com") || s.contains("collection.json"))
                .unwrap_or(false)
            || (info.get("name").is_some() && json.get("item").is_some())
        {
            return Some(ImportFormat::Postman);
        }
    }
    if json.get("item").and_then(|i| i.as_array()).is_some() && json.get("info").is_some() {
        return Some(ImportFormat::Postman);
    }

    None
}

fn detect_from_yaml(yaml: &serde_yaml::Value) -> Option<ImportFormat> {
    if yaml.get("__export_format").is_some()
        || yaml.get("_type").and_then(|v| v.as_str()) == Some("export")
    {
        return Some(ImportFormat::Insomnia);
    }
    if let Some(res) = yaml.get("resources").and_then(|r| r.as_sequence()) {
        if res.iter().any(|item| {
            item.get("_type")
                .and_then(|t| t.as_str())
                .map(|t| t == "workspace" || t == "request" || t == "request_group")
                .unwrap_or(false)
        }) {
            return Some(ImportFormat::Insomnia);
        }
    }

    if let Some(t) = yaml.get("type").and_then(|v| v.as_str()) {
        if is_insomnia_v5_type_str(t) {
            return Some(ImportFormat::Insomnia);
        }
    }
    if yaml.get("schema_version").is_some()
        && (yaml.get("collection").is_some()
            || yaml.get("environments").is_some()
            || yaml.get("routes").is_some())
    {
        return Some(ImportFormat::Insomnia);
    }

    if yaml.get("veyakSchema").is_some() {
        return Some(ImportFormat::Veyak);
    }

    None
}
