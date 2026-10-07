use std::path::{Path, PathBuf};

use serde::{Serialize, de::DeserializeOwned};
use veyak_error::AppResult;
use veyak_models::{
    ActiveState, AppSettings, HttpMethod, RequestItem, Theme, ThemeSyntax, ThemeTokens, ThemeUI,
    ThemeVariant, Workspace, default_schema,
};

#[derive(Debug, Clone)]
pub struct DataDir {
    root: PathBuf,
}

impl DataDir {
    pub fn root(&self) -> &Path {
        &self.root
    }

    // -- Convenience path builders ------------------------------------------

    pub fn app_state_path(&self) -> PathBuf {
        self.root.join("app_state.yaml")
    }

    pub fn settings_path(&self) -> PathBuf {
        self.root.join("settings.yaml")
    }

    pub fn themes_path(&self) -> PathBuf {
        self.root.join("themes")
    }

    pub fn plugins_path(&self) -> PathBuf {
        self.root.join("plugins.yaml")
    }

    pub fn history_path(&self) -> PathBuf {
        self.root.join("history.yaml")
    }

    pub fn workspaces_dir(&self) -> PathBuf {
        self.root.join("workspaces")
    }

    pub fn workspace_dir(&self, workspace_id: &str) -> PathBuf {
        self.workspaces_dir().join(workspace_id)
    }

    pub fn workspace_meta_path(&self, workspace_id: &str) -> PathBuf {
        self.workspace_dir(workspace_id).join("workspace.yaml")
    }

    pub fn environments_path(&self, workspace_id: &str) -> PathBuf {
        self.workspace_dir(workspace_id).join("environments.yaml")
    }

    pub fn collections_dir(&self, workspace_id: &str) -> PathBuf {
        self.workspace_dir(workspace_id).join("collections")
    }

    pub fn collection_dir(&self, workspace_id: &str, collection_id: &str) -> PathBuf {
        self.collections_dir(workspace_id).join(collection_id)
    }

    pub fn collection_meta_path(&self, workspace_id: &str, collection_id: &str) -> PathBuf {
        self.collection_dir(workspace_id, collection_id)
            .join("collection.yaml")
    }

    pub fn collection_environments_path(&self, workspace_id: &str, collection_id: &str) -> PathBuf {
        self.collection_dir(workspace_id, collection_id)
            .join("environments.yaml")
    }

    pub fn requests_dir(&self, workspace_id: &str, collection_id: &str) -> PathBuf {
        self.collection_dir(workspace_id, collection_id)
            .join("requests")
    }

    pub fn request_path(
        &self,
        workspace_id: &str,
        collection_id: &str,
        request_id: &str,
    ) -> PathBuf {
        self.requests_dir(workspace_id, collection_id)
            .join(format!("{request_id}.yaml"))
    }

    pub fn folders_dir(&self, workspace_id: &str, collection_id: &str) -> PathBuf {
        self.collection_dir(workspace_id, collection_id)
            .join("folders")
    }

    pub fn folder_path(&self, workspace_id: &str, collection_id: &str, folder_id: &str) -> PathBuf {
        self.folders_dir(workspace_id, collection_id)
            .join(format!("{folder_id}.yaml"))
    }

    /// Per-request YAML file that stores the user's saved WS message
    /// templates.  Lives at `<data>/ws_messages/<request_id>.yaml`.
    pub fn ws_saved_messages_path(&self, request_id: &str) -> PathBuf {
        self.root
            .join("ws_messages")
            .join(format!("{request_id}.yaml"))
    }
}

// ---------------------------------------------------------------------------
// Generic YAML read/write helpers
// ---------------------------------------------------------------------------

pub fn read_yaml<T: DeserializeOwned>(path: &Path) -> AppResult<T> {
    let contents = std::fs::read_to_string(path)?;
    let value = serde_yaml::from_str(&contents)?;
    Ok(value)
}

pub fn write_yaml<T: Serialize>(path: &Path, value: &T) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let yaml = serde_yaml::to_string(value)?;
    std::fs::write(path, yaml)?;
    Ok(())
}

/// Read a YAML file or return a default value if it doesn't exist.
pub fn read_yaml_or_default<T: DeserializeOwned + Default>(path: &Path) -> AppResult<T> {
    if path.exists() {
        read_yaml(path)
    } else {
        Ok(T::default())
    }
}

/// Read a YAML file containing a `Vec<T>`, returning an empty vec if
/// the file doesn't exist.
pub fn read_yaml_vec<T: DeserializeOwned>(path: &Path) -> AppResult<Vec<T>> {
    if path.exists() {
        read_yaml(path)
    } else {
        Ok(Vec::new())
    }
}

// ---------------------------------------------------------------------------
// Initialization — seed default data when starting fresh
// ---------------------------------------------------------------------------

/// Creates the data directory hierarchy and seeds default files when
/// they don't already exist.  Called once from `lib.rs`'s setup hook.
pub fn init_data_dir(data_dir: &Path) -> AppResult<DataDir> {
    let root = data_dir.join("data");
    std::fs::create_dir_all(&root)?;

    let dd = DataDir { root };

    // -- Seed defaults only when the files are missing ---------------------

    // Default workspace
    let default_ws_id = "default-workspace";
    if !dd.workspace_meta_path(default_ws_id).exists() {
        let ws = Workspace {
            id: default_ws_id.to_string(),
            name: "My Workspace".to_string(),
            created_at: now_iso(),
            updated_at: now_iso(),
        };
        std::fs::create_dir_all(dd.workspace_dir(default_ws_id))?;
        write_yaml(&dd.workspace_meta_path(default_ws_id), &ws)?;
    }

    // Default settings
    if !dd.settings_path().exists() {
        write_yaml(&dd.settings_path(), &AppSettings::default())?;
    }

    for theme in default_themes() {
        let theme_path = dd.themes_path().join(format!("{}.yaml", theme.id));
        if !theme_path.exists() {
            write_yaml(&theme_path, &theme)?;
        }
    }

    // Default app state
    if !dd.app_state_path().exists() {
        let state = ActiveState {
            active_workspace_id: Some(default_ws_id.to_string()),
            active_theme_id: Some("veyak-dark".to_string()),
            ..Default::default()
        };
        write_yaml(&dd.app_state_path(), &state)?;
    }

    // -- Migrations ----------------------------------------------------------
    migrate_ws_method(&dd);
    migrate_request_http_type(&dd);

    Ok(dd)
}

// ---------------------------------------------------------------------------
// Shared utilities (unchanged from the SQLite version)
// ---------------------------------------------------------------------------

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

// ---------------------------------------------------------------------------
// Migration: fix existing WS requests that have method GET + ws:// URL
// ---------------------------------------------------------------------------

/// Scans all request YAML files and updates any that have `method: GET`
/// with a `ws://` or `wss://` URL to `method: WS`. Runs once (uses a
/// marker file to avoid re-scanning on every startup).
fn migrate_ws_method(dd: &DataDir) {
    use veyak_models::ApiRequest;

    let marker = dd.root().join(".migration_ws_method_done");
    if marker.exists() {
        return;
    }

    let workspaces_dir = dd.workspaces_dir();
    if !workspaces_dir.exists() {
        // Nothing to migrate
        let _ = std::fs::write(&marker, "done");
        return;
    }

    let mut updated = 0u32;

    if let Ok(ws_entries) = std::fs::read_dir(&workspaces_dir) {
        for ws_entry in ws_entries.flatten() {
            if !ws_entry.path().is_dir() {
                continue;
            }
            let collections_dir = ws_entry.path().join("collections");
            if !collections_dir.exists() {
                continue;
            }
            if let Ok(col_entries) = std::fs::read_dir(&collections_dir) {
                for col_entry in col_entries.flatten() {
                    let requests_dir = col_entry.path().join("requests");
                    if !requests_dir.exists() {
                        continue;
                    }
                    if let Ok(req_entries) = std::fs::read_dir(&requests_dir) {
                        for req_entry in req_entries.flatten() {
                            let path = req_entry.path();
                            if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
                                continue;
                            }
                            // Try to read and fix the request
                            if let Ok(mut req) = read_yaml::<ApiRequest>(&path) {
                                let url_lower = req.url.trim().to_lowercase();
                                if req.method == HttpMethod::Get
                                    && (url_lower.starts_with("ws://")
                                        || url_lower.starts_with("wss://"))
                                {
                                    req.method = HttpMethod::Ws;
                                    if write_yaml(&path, &req).is_ok() {
                                        updated += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if updated > 0 {
        println!("[migration] Updated {updated} WS request(s) from GET to WS method");
    }

    let _ = std::fs::write(&marker, "done");
}

fn migrate_request_http_type(dd: &DataDir) {
    use veyak_models::ApiRequest;

    let marker = dd.root().join(".migration_request_type_done");
    if marker.exists() {
        return;
    }

    let workspaces_dir = dd.workspaces_dir();
    if !workspaces_dir.exists() {
        // Nothing to migrate
        let _ = std::fs::write(&marker, "done");
        return;
    }

    let mut updated = 0u32;

    if let Ok(ws_entries) = std::fs::read_dir(&workspaces_dir) {
        for ws_entry in ws_entries.flatten() {
            if !ws_entry.path().is_dir() {
                continue;
            }
            let collections_dir = ws_entry.path().join("collections");
            if !collections_dir.exists() {
                continue;
            }
            if let Ok(col_entries) = std::fs::read_dir(&collections_dir) {
                for col_entry in col_entries.flatten() {
                    let requests_dir = col_entry.path().join("requests");
                    if !requests_dir.exists() {
                        continue;
                    }
                    if let Ok(req_entries) = std::fs::read_dir(&requests_dir) {
                        for req_entry in req_entries.flatten() {
                            let path = req_entry.path();
                            if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
                                continue;
                            }
                            // Try to read and fix the request
                            let mut needs_save = false;
                            let request_item = if let Ok(item) = read_yaml::<RequestItem>(&path) {
                                item
                            } else if let Ok(old_req) = read_yaml::<ApiRequest>(&path) {
                                // Legacy file missing `type: http`. Wrap it, and flag for saving.
                                needs_save = true;
                                RequestItem::Http(old_req)
                            } else {
                                // Unparseable file, skip
                                continue; // or `continue;` if inside a loop
                            };

                            // 4. Save back to disk if any migrations (WS or missing type tag) were applied.
                            // Serializing `RequestItem` will automatically write the `type: http` key.
                            if needs_save {
                                if write_yaml(&path, &request_item).is_ok() {
                                    updated += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if updated > 0 {
        println!("[migration] Add Request Type Http on  {updated} request(s)");
    }

    let _ = std::fs::write(&marker, "done");
}

fn default_themes() -> Vec<Theme> {
    vec![
        // 1. Veyak Dark — brand default
        Theme {
            schema: default_schema(),
            id: "veyak-dark".to_string(),
            name: "Veyak Dark".to_string(),
            author: "Hrithik Dhakrey".to_string(),
            version: "1.0.0".to_string(),
            description: "Default brand palette with deep slate & violet".to_string(),
            license: "MIT".to_string(),
            is_builtin: true,
            tags: vec![
                "dark".to_string(),
                "default".to_string(),
                "violet".to_string(),
            ],
            repository: "https://github.com/Veyak/veyak".to_string(),
            variant: ThemeVariant::Dark,
            tokens: ThemeTokens {
                ui: ThemeUI {
                    color_bg: "#0D1117".to_string(),
                    color_panel: "#161B22".to_string(),
                    color_panel_raised: "#1C2129".to_string(),
                    color_border: "#30363D".to_string(),
                    color_border_muted: "#21262D".to_string(),
                    color_text_primary: "#F0F6FC".to_string(),
                    color_text_secondary: "#9198A1".to_string(),
                    color_text_muted: "#656C76".to_string(),
                    color_primary: "#8B5CF6".to_string(),
                    color_primary_hover: "#9D74F8".to_string(),
                    color_secondary: "#3B82F6".to_string(),
                    color_success: "#10B981".to_string(),
                    color_error: "#EF4444".to_string(),
                    color_warning: "#F59E0B".to_string(),
                    method_get: "#2EA043".to_string(),
                    method_post: "#388BFD".to_string(),
                    method_put: "#F7681D".to_string(),
                    method_patch: "#FFB833".to_string(),
                    method_delete: "#FF445E".to_string(),
                    method_ws: "#792DFF".to_string(),
                    method_query: "#9D74F8".to_string(),
                    method_grpc: "#9D74F8".to_string(),
                    method_graphql: "#E30372".to_string(),
                    radius_md: "8px".to_string(),
                    radius_lg: "10px".to_string(),
                },
                syntax: ThemeSyntax {
                    keyword: "#FF7B72".to_string(),
                    string: "#7EE787".to_string(),
                    comment: "#8B949E".to_string(),
                    property: "#79C0FF".to_string(),
                    punctuation: "#C9D1D9".to_string(),
                    operator: "#F2CC60".to_string(),
                    number: "#D2A8FF".to_string(),
                    boolean: "#FFAB70".to_string(),
                    null: "#FFAB70".to_string(),
                    function: "#FFAB70".to_string(),
                    variable: "#FFAB70".to_string(),
                    attribute: "#FFAB70".to_string(),
                    class_name: "#FFAB70".to_string(),
                },
            },
        },
        // 2. Zed One Dark — zinc & blue
        Theme {
            schema: default_schema(),
            id: "zed-one-dark".to_string(),
            name: "Zed One Dark".to_string(),
            author: "Hrithik Dhakrey".to_string(),
            version: "1.0.0".to_string(),
            description: "Zed's signature zinc & blue aesthetic".to_string(),
            license: "MIT".to_string(),
            is_builtin: true,
            tags: vec![
                "dark".to_string(),
                "default".to_string(),
                "zinc".to_string(),
            ],
            repository: "https://github.com/Veyak/veyak".to_string(),
            variant: ThemeVariant::Dark,
            tokens: ThemeTokens {
                ui: ThemeUI {
                    color_bg: "#282C34".to_string(),
                    color_panel: "#21252B".to_string(),
                    color_panel_raised: "#2C313A".to_string(),
                    color_border: "#3E4451".to_string(),
                    color_border_muted: "#2C313A".to_string(),
                    color_text_primary: "#ABB2BF".to_string(),
                    color_text_secondary: "#828997".to_string(),
                    color_text_muted: "#5C6370".to_string(),
                    color_primary: "#61AFEF".to_string(),
                    color_primary_hover: "#528BFF".to_string(),
                    color_secondary: "#56B6C2".to_string(),
                    color_success: "#98C379".to_string(),
                    color_error: "#E06C75".to_string(),
                    color_warning: "#E5C07B".to_string(),
                    method_get: "#98C379".to_string(),
                    method_post: "#61AFEF".to_string(),
                    method_put: "#E5C07B".to_string(),
                    method_patch: "#D19A66".to_string(),
                    method_delete: "#E06C75".to_string(),
                    method_ws: "#C678DD".to_string(),
                    method_query: "#56B6C2".to_string(),
                    method_grpc: "#528BFF".to_string(),
                    method_graphql: "#C678DD".to_string(),
                    radius_md: "6px".to_string(),
                    radius_lg: "8px".to_string(),
                },
                syntax: ThemeSyntax {
                    keyword: "#C678DD".to_string(),
                    string: "#98C379".to_string(),
                    comment: "#5C6370".to_string(),
                    property: "#E06C75".to_string(),
                    punctuation: "#ABB2BF".to_string(),
                    operator: "#56B6C2".to_string(),
                    number: "#D19A66".to_string(),
                    boolean: "#D19A66".to_string(),
                    null: "#D19A66".to_string(),
                    function: "#61AFEF".to_string(),
                    variable: "#E06C75".to_string(),
                    attribute: "#D19A66".to_string(),
                    class_name: "#E5C07B".to_string(),
                },
            },
        },
        // 3. Antigravity Deep Space — high-contrast neon
        Theme {
            schema: default_schema(),
            id: "antigravity-deep-space".to_string(),
            name: "Antigravity Deep Space".to_string(),
            author: "Hrithik Dhakrey".to_string(),
            version: "1.0.0".to_string(),
            description: "High-contrast dark theme with vibrant neon highlights".to_string(),
            license: "MIT".to_string(),
            is_builtin: true,
            tags: vec![
                "dark".to_string(),
                "default".to_string(),
                "neon".to_string(),
                "high-contrast".to_string(),
            ],
            repository: "https://github.com/Veyak/veyak".to_string(),
            variant: ThemeVariant::Dark,
            tokens: ThemeTokens {
                ui: ThemeUI {
                    color_bg: "#05060A".to_string(),
                    color_panel: "#0A0D16".to_string(),
                    color_panel_raised: "#121622".to_string(),
                    color_border: "#232A3D".to_string(),
                    color_border_muted: "#161B29".to_string(),
                    color_text_primary: "#EAF2FF".to_string(),
                    color_text_secondary: "#8B93B0".to_string(),
                    color_text_muted: "#4F5875".to_string(),
                    color_primary: "#00F5D4".to_string(),
                    color_primary_hover: "#3DFFE4".to_string(),
                    color_secondary: "#FF2E9A".to_string(),
                    color_success: "#39FF88".to_string(),
                    color_error: "#FF3B5C".to_string(),
                    color_warning: "#FFD60A".to_string(),
                    method_get: "#39FF88".to_string(),
                    method_post: "#00F5D4".to_string(),
                    method_put: "#FFD60A".to_string(),
                    method_patch: "#FF9F1C".to_string(),
                    method_delete: "#FF3B5C".to_string(),
                    method_ws: "#B537F2".to_string(),
                    method_query: "#00D1FF".to_string(),
                    method_grpc: "#7B61FF".to_string(),
                    method_graphql: "#FF2E9A".to_string(),
                    radius_md: "4px".to_string(),
                    radius_lg: "6px".to_string(),
                },
                syntax: ThemeSyntax {
                    keyword: "#FF2E9A".to_string(),
                    string: "#39FF88".to_string(),
                    comment: "#4F5875".to_string(),
                    property: "#00D1FF".to_string(),
                    punctuation: "#EAF2FF".to_string(),
                    operator: "#FFD60A".to_string(),
                    number: "#B537F2".to_string(),
                    boolean: "#FF9F1C".to_string(),
                    null: "#FF9F1C".to_string(),
                    function: "#00F5D4".to_string(),
                    variable: "#EAF2FF".to_string(),
                    attribute: "#FFD60A".to_string(),
                    class_name: "#FF2E9A".to_string(),
                },
            },
        },
        // 4. VS Code Dark+ — classic Microsoft dark
        Theme {
            schema: default_schema(),
            author: "Hrithik Dhakrey".to_string(),
            description: "Classic Microsoft Visual Studio Code Dark".to_string(),
            license: "MIT".to_string(),
            is_builtin: true,
            tags: vec!["dark".to_string(), "default".to_string()],
            repository: "https://github.com/Veyak/veyak".to_string(),
            variant: ThemeVariant::Dark,
            id: "vscode-dark-plus".to_string(),
            name: "VS Code Dark+".to_string(),
            version: "1.0.0".to_string(),
            tokens: ThemeTokens {
                ui: ThemeUI {
                    color_bg: "#1E1E1E".to_string(),
                    color_panel: "#252526".to_string(),
                    color_panel_raised: "#2D2D2D".to_string(),
                    color_border: "#3C3C3C".to_string(),
                    color_border_muted: "#2B2B2B".to_string(),
                    color_text_primary: "#CCCCCC".to_string(),
                    color_text_secondary: "#969696".to_string(),
                    color_text_muted: "#6E7681".to_string(),
                    color_primary: "#007ACC".to_string(),
                    color_primary_hover: "#0062A3".to_string(),
                    color_secondary: "#3A3D41".to_string(),
                    color_success: "#89D185".to_string(),
                    color_error: "#F48771".to_string(),
                    color_warning: "#CCA700".to_string(),
                    method_get: "#4EC9B0".to_string(),
                    method_post: "#569CD6".to_string(),
                    method_put: "#DCDCAA".to_string(),
                    method_delete: "#F44747".to_string(),
                    method_patch: "#C586C0".to_string(),
                    method_query: "#9CDCFE".to_string(),
                    method_ws: "#B5CEA8".to_string(),
                    method_grpc: "#4FC1FF".to_string(),
                    method_graphql: "#E06C75".to_string(),
                    radius_md: "4px".to_string(),
                    radius_lg: "6px".to_string(),
                },
                syntax: ThemeSyntax {
                    keyword: "#C586C0".to_string(),
                    string: "#CE9178".to_string(),
                    comment: "#6A9955".to_string(),
                    property: "#9CDCFE".to_string(),
                    punctuation: "#D4D4D4".to_string(),
                    operator: "#D4D4D4".to_string(),
                    number: "#B5CEA8".to_string(),
                    boolean: "#569CD6".to_string(),
                    null: "#569CD6".to_string(),
                    function: "#569CD6".to_string(),
                    variable: "#569CD6".to_string(),
                    attribute: "#569CD6".to_string(),
                    class_name: "#569CD6".to_string(),
                },
            },
        },
        // 5. Catppuccin Macchiato — warm pastel
        Theme {
            schema: default_schema(),
            id: "catppuccin-macchiato".to_string(),
            name: "Catppuccin Macchiato".to_string(),
            author: "Hrithik Dhakrey".to_string(),
            version: "1.0.0".to_string(),
            description: "Warm, soothing pastel aesthetic".to_string(),
            license: "MIT".to_string(),
            is_builtin: true,
            tags: vec![
                "dark".to_string(),
                "default".to_string(),
                "pastel".to_string(),
            ],
            repository: "https://github.com/Veyak/veyak".to_string(),
            variant: ThemeVariant::Dark,
            tokens: ThemeTokens {
                ui: ThemeUI {
                    color_bg: "#24273A".to_string(),
                    color_panel: "#1E2030".to_string(),
                    color_panel_raised: "#363A4F".to_string(),
                    color_border: "#494D64".to_string(),
                    color_border_muted: "#363A4F".to_string(),
                    color_text_primary: "#CAD3F5".to_string(),
                    color_text_secondary: "#B8C0E0".to_string(),
                    color_text_muted: "#8087A2".to_string(),
                    color_primary: "#C6A0F6".to_string(),
                    color_primary_hover: "#F5BDE6".to_string(),
                    color_secondary: "#8AADF4".to_string(),
                    color_success: "#A6DA95".to_string(),
                    color_error: "#ED8796".to_string(),
                    color_warning: "#EED49F".to_string(),
                    method_get: "#A6DA95".to_string(),
                    method_post: "#8AADF4".to_string(),
                    method_put: "#F5A97F".to_string(),
                    method_patch: "#EED49F".to_string(),
                    method_delete: "#ED8796".to_string(),
                    method_ws: "#C6A0F6".to_string(),
                    method_query: "#7DC4E4".to_string(),
                    method_grpc: "#B7BDF8".to_string(),
                    method_graphql: "#F5BDE6".to_string(),
                    radius_md: "10px".to_string(),
                    radius_lg: "14px".to_string(),
                },
                syntax: ThemeSyntax {
                    keyword: "#C6A0F6".to_string(),
                    string: "#A6DA95".to_string(),
                    comment: "#6E738D".to_string(),
                    property: "#8AADF4".to_string(),
                    punctuation: "#939AB7".to_string(),
                    operator: "#91D7E3".to_string(),
                    number: "#F5A97F".to_string(),
                    boolean: "#F5A97F".to_string(),
                    null: "#F5A97F".to_string(),
                    function: "#8AADF4".to_string(),
                    variable: "#CAD3F5".to_string(),
                    attribute: "#EED49F".to_string(),
                    class_name: "#EED49F".to_string(),
                },
            },
        },
        // 6. Tokyo Night — cyber blue & purple
        Theme {
            schema: default_schema(),
            id: "tokyo-night".to_string(),
            name: "Tokyo Night".to_string(),
            author: "Hrithik Dhakrey".to_string(),
            version: "1.0.0".to_string(),
            description: "Cyber blue & purple nighttime theme".to_string(),
            license: "MIT".to_string(),
            is_builtin: true,
            tags: vec![
                "dark".to_string(),
                "default".to_string(),
                "purple".to_string(),
            ],
            repository: "https://github.com/Veyak/veyak".to_string(),
            variant: ThemeVariant::Dark,
            tokens: ThemeTokens {
                ui: ThemeUI {
                    color_bg: "#1A1B26".to_string(),
                    color_panel: "#16161E".to_string(),
                    color_panel_raised: "#292E42".to_string(),
                    color_border: "#3B4261".to_string(),
                    color_border_muted: "#292E42".to_string(),
                    color_text_primary: "#C0CAF5".to_string(),
                    color_text_secondary: "#A9B1D6".to_string(),
                    color_text_muted: "#565F89".to_string(),
                    color_primary: "#7AA2F7".to_string(),
                    color_primary_hover: "#89DDFF".to_string(),
                    color_secondary: "#BB9AF7".to_string(),
                    color_success: "#9ECE6A".to_string(),
                    color_error: "#F7768E".to_string(),
                    color_warning: "#E0AF68".to_string(),
                    method_get: "#9ECE6A".to_string(),
                    method_post: "#7AA2F7".to_string(),
                    method_put: "#E0AF68".to_string(),
                    method_patch: "#FF9E64".to_string(),
                    method_delete: "#F7768E".to_string(),
                    method_ws: "#BB9AF7".to_string(),
                    method_query: "#7DCFFF".to_string(),
                    method_grpc: "#2AC3DE".to_string(),
                    method_graphql: "#9D7CD8".to_string(),
                    radius_md: "6px".to_string(),
                    radius_lg: "8px".to_string(),
                },
                syntax: ThemeSyntax {
                    keyword: "#BB9AF7".to_string(),
                    string: "#9ECE6A".to_string(),
                    comment: "#565F89".to_string(),
                    property: "#7AA2F7".to_string(),
                    punctuation: "#C0CAF5".to_string(),
                    operator: "#89DDFF".to_string(),
                    number: "#FF9E64".to_string(),
                    boolean: "#FF9E64".to_string(),
                    null: "#FF9E64".to_string(),
                    function: "#7AA2F7".to_string(),
                    variable: "#C0CAF5".to_string(),
                    attribute: "#E0AF68".to_string(),
                    class_name: "#E0AF68".to_string(),
                },
            },
        },
        // 7. GitHub Light — clean daytime palette
        Theme {
            schema: default_schema(),
            id: "github-light".to_string(),
            name: "GitHub Light".to_string(),
            author: "Hrithik Dhakrey".to_string(),
            version: "1.0.0".to_string(),
            description: "Clean daytime palette for light-mode users".to_string(),
            license: "MIT".to_string(),
            is_builtin: true,
            tags: vec!["light".to_string(), "default".to_string()],
            repository: "https://github.com/Veyak/veyak".to_string(),
            variant: ThemeVariant::Light,
            tokens: ThemeTokens {
                ui: ThemeUI {
                    color_bg: "#FFFFFF".to_string(),
                    color_panel: "#F6F8FA".to_string(),
                    color_panel_raised: "#EAEEF2".to_string(),
                    color_border: "#D0D7DE".to_string(),
                    color_border_muted: "#D8DEE4".to_string(),
                    color_text_primary: "#1F2328".to_string(),
                    color_text_secondary: "#656D76".to_string(),
                    color_text_muted: "#6E7781".to_string(),
                    color_primary: "#0969DA".to_string(),
                    color_primary_hover: "#0550AE".to_string(),
                    color_secondary: "#8250DF".to_string(),
                    color_success: "#1A7F37".to_string(),
                    color_error: "#D1242F".to_string(),
                    color_warning: "#9A6700".to_string(),
                    method_get: "#1A7F37".to_string(),
                    method_post: "#0969DA".to_string(),
                    method_put: "#9A6700".to_string(),
                    method_patch: "#BF3989".to_string(),
                    method_delete: "#D1242F".to_string(),
                    method_ws: "#8250DF".to_string(),
                    method_query: "#0969DA".to_string(),
                    method_grpc: "#1B7C83".to_string(),
                    method_graphql: "#E10098".to_string(),
                    radius_md: "6px".to_string(),
                    radius_lg: "8px".to_string(),
                },
                syntax: ThemeSyntax {
                    keyword: "#CF222E".to_string(),
                    string: "#0A3069".to_string(),
                    comment: "#6E7781".to_string(),
                    property: "#0550AE".to_string(),
                    punctuation: "#1F2328".to_string(),
                    operator: "#0550AE".to_string(),
                    number: "#0550AE".to_string(),
                    boolean: "#0550AE".to_string(),
                    null: "#0550AE".to_string(),
                    function: "#8250DF".to_string(),
                    variable: "#953800".to_string(),
                    attribute: "#0550AE".to_string(),
                    class_name: "#953800".to_string(),
                },
            },
        },
    ]
}
