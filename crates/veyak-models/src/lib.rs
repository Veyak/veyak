use serde::{Deserialize, Serialize};
use ts_rs::TS;

use std::{collections::BTreeMap, str::FromStr};

// ── Models ──────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub expires_in: u64,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct User {
    pub sub: String,
    #[ts(optional)]
    pub name: Option<String>,
    #[ts(optional)]
    pub email: Option<String>,
    #[ts(optional)]
    pub picture: Option<String>,
    #[ts(optional)]
    pub updated_at: Option<String>,
}

/// Persistent auth state saved to `auth.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct AuthState {
    pub user: Option<User>,
    pub tokens: Option<AuthTokens>,
}

/// In-flight PKCE session (kept in memory, not persisted).
#[derive(Debug, Clone)]
pub struct PkceSession {
    pub state: String,
    pub code_verifier: String,
    pub loopback_uri: String,
}

// ---------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export, export_to = "models.ts")]
pub enum HttpMethod {
    #[serde(rename = "GET")]
    Get,
    #[serde(rename = "POST")]
    Post,
    #[serde(rename = "PUT")]
    Put,
    #[serde(rename = "PATCH")]
    Patch,
    #[serde(rename = "DELETE")]
    Delete,
    #[serde(rename = "OPTIONS")]
    Options,
    #[serde(rename = "HEAD")]
    Head,
    #[serde(rename = "WS")]
    Ws,
    #[serde(rename = "QUERY")]
    Query,
}

impl HttpMethod {
    pub fn as_reqwest(&self) -> reqwest::Method {
        match self {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Patch => reqwest::Method::PATCH,
            HttpMethod::Delete => reqwest::Method::DELETE,
            HttpMethod::Options => reqwest::Method::OPTIONS,
            HttpMethod::Head => reqwest::Method::HEAD,
            HttpMethod::Query => reqwest::Method::QUERY,

            HttpMethod::Ws => panic!("WS requests should not go through the HTTP pipeline"),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Head => "HEAD",
            HttpMethod::Ws => "WS",
            HttpMethod::Query => "QUERY",
        }
    }
}

impl FromStr for HttpMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Case-insensitive matching
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "PATCH" => Ok(HttpMethod::Patch),
            "DELETE" => Ok(HttpMethod::Delete),
            "OPTIONS" => Ok(HttpMethod::Options),
            "HEAD" => Ok(HttpMethod::Head),
            "WS" => Ok(HttpMethod::Ws),
            "QUERY" => Ok(HttpMethod::Query),
            _ => Err(format!("Invalid HTTP method: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export, export_to = "models.ts")]
pub struct KeyValueRow {
    pub id: String,
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export, export_to = "models.ts")]
pub struct CookieRow {
    pub id: String,
    pub name: String,
    pub value: String,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "models.ts")]
pub struct BearerAuth {
    pub token: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub enum ApiKeyTarget {
    Header,
    Query,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct ApiKeyAuth {
    pub key: String,
    pub value: String,
    pub add_to: ApiKeyTarget,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub enum AuthType {
    #[default]
    None,
    Basic,
    Bearer,
    ApiKey,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct AuthConfig {
    #[serde(rename = "type")]
    pub auth_type: AuthType,
    pub basic: Option<BasicAuth>,
    pub bearer: Option<BearerAuth>,
    pub api_key: Option<ApiKeyAuth>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct UploadedFile {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
    /// Absolute filesystem path, populated by the `pick_files` command
    /// (native file dialog), giving the backend real disk access for
    /// multipart uploads.
    pub path: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "models.ts")]
pub enum BodyMode {
    Json,
    FormData,
    Urlencoded,
    Raw,
    Multipart,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct RequestBody {
    pub mode: Option<BodyMode>,
    #[ts(optional)]
    pub raw: Option<String>,
    #[ts(optional)]
    pub form_data: Option<Vec<KeyValueRow>>,
    #[ts(optional)]
    pub url_encoded: Option<Vec<KeyValueRow>>,
    #[ts(optional)]
    pub files: Option<Vec<UploadedFile>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct ApiRequest {
    pub id: String,
    #[serde(default)]
    pub collection_id: String,
    #[serde(default)]
    pub folder_id: Option<String>,
    #[serde(default)]
    pub sort_order: i64,
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    pub params: Vec<KeyValueRow>,
    pub headers: Vec<KeyValueRow>,
    pub cookies: Vec<CookieRow>,
    pub auth: AuthConfig,
    pub body: RequestBody,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct ApiResponse {
    pub status: u16,
    pub status_text: String,
    pub time_ms: u128,
    pub size_bytes: u64,
    pub headers: BTreeMap<String, String>,
    pub cookies: Vec<CookieRow>,
    pub body: String,
}

// ---------------------------------------------------------------------
// Workspaces / collections / folders
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct AdditionType {
    pub id: String,
    pub label: String,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct Collection {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct Folder {
    pub id: String,
    pub collection_id: String,
    pub parent_folder_id: Option<String>,
    pub name: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub enum RequestItem {
    Http(ApiRequest),
    Grpc(GrpcRequest),
    #[serde(rename = "graphql")]
    #[ts(rename = "graphql")]
    GraphQL(GraphQlRequest),
}

impl RequestItem {
    pub fn id(&self) -> &str {
        match self {
            RequestItem::Http(r) => &r.id,
            RequestItem::Grpc(r) => &r.id,
            RequestItem::GraphQL(r) => &r.id,
        }
    }

    pub fn collection_id(&self) -> &str {
        match self {
            RequestItem::Http(r) => &r.collection_id,
            RequestItem::Grpc(r) => &r.collection_id,
            RequestItem::GraphQL(r) => &r.collection_id,
        }
    }

    pub fn folder_id(&self) -> Option<&str> {
        match self {
            RequestItem::Http(r) => r.folder_id.as_deref(),
            RequestItem::Grpc(r) => r.folder_id.as_deref(),
            RequestItem::GraphQL(r) => r.folder_id.as_deref(),
        }
    }

    pub fn sort_order(&self) -> i64 {
        match self {
            RequestItem::Http(r) => r.sort_order,
            RequestItem::Grpc(r) => r.sort_order,
            RequestItem::GraphQL(r) => r.sort_order,
        }
    }
}

/// Nested tree shape sent to the frontend for rendering the sidebar in
/// one shot, rather than making it re-assemble flat rows.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct CollectionTree {
    pub collection: Collection,
    pub folders: Vec<FolderNode>,
    /// Requests directly under the collection root (no folder).
    pub requests: Vec<RequestItem>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct FolderNode {
    pub folder: Folder,
    pub children: Vec<FolderNode>,
    pub requests: Vec<RequestItem>,
}

// ---------------------------------------------------------------------
// Environments
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct EnvironmentVariable {
    pub id: String,
    pub environmentid: String,
    pub key: String,
    pub value: String,
    pub enabled: bool,
    pub is_secret: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct Environment {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct EnvironmentWithVariables {
    pub environment: Environment,
    pub variables: Vec<EnvironmentVariable>,
}

// ---------------------------------------------------------------------
// History
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct HistoryEntry {
    pub id: String,
    pub request_id: Option<String>,
    pub name: Option<String>,
    pub method: HttpMethod,
    pub url: String,
    pub status: u16,
    pub duration_ms: i64,
    pub created_at: String,
}

// ---------------------------------------------------------------------
// Import / Export
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct ImportSummary {
    pub collections_count: usize,
    pub folders_count: usize,
    pub requests_count: usize,
    pub environments_count: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "models.ts")]
pub enum ImportFormat {
    #[default]
    Auto,
    Postman,
    PostmanEnvironment,
    Insomnia,
    Yaak,
    Veyak,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "models.ts")]
pub enum ExportFormat {
    #[default]
    Postman,
    Insomnia,
    Yaak,
    Veyak,
}

// ---------------------------------------------------------------------
// Settings — follow-redirect / TLS / proxy behavior, all the way down
// to a single persisted JSON blob (see migrations/0001_init.sql).
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct FontSettings {
    pub app_font_family: String,
    pub font_family: String,
    pub custom_font_path: Option<String>,
    pub font_size: u32,
    pub line_height: f32,
    pub enable_ligatures: bool,
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            app_font_family: "Inter".to_string(),
            font_family: "Fira Code, monospace".to_string(),
            custom_font_path: None,
            font_size: 14,
            line_height: 1.5,
            enable_ligatures: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct AppSettings {
    pub follow_redirects: bool,
    pub max_redirects: u32,
    /// When `false`, the HTTP client is built with
    /// `danger_accept_invalid_certs(true)` — i.e. self-signed / invalid
    /// certs are accepted. Named for clarity in the UI ("Validate SSL
    /// certificates" toggle) rather than mirroring reqwest's "danger_*"
    /// naming directly.
    pub verify_ssl_certificates: bool,
    pub timeout_ms: u64,
    pub user_agent: String,
    pub proxy_url: Option<String>,
    #[serde(default)]
    pub font: FontSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            follow_redirects: true,
            max_redirects: 10,
            verify_ssl_certificates: true,
            timeout_ms: 30_000,
            user_agent: format!("Veyak/{}", env!("CARGO_PKG_VERSION")),
            proxy_url: None,
            font: FontSettings::default(),
        }
    }
}

// ---------------------------------------------------------------------
// Themes
// ---------------------------------------------------------------------

/// Mirrors the `@theme` tokens in the frontend's `src/index.css`. Custom
/// themes are just a different value set for the same keys — the
/// frontend applies a theme by writing these as CSS custom properties
/// on `:root` at runtime (see commands/themes.rs doc comment).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct ThemeUI {
    pub color_bg: String,
    pub color_panel: String,
    pub color_panel_raised: String,
    pub color_border: String,
    pub color_border_muted: String,
    pub color_text_primary: String,
    pub color_text_secondary: String,
    pub color_text_muted: String,
    pub color_primary: String,
    pub color_primary_hover: String,
    pub color_secondary: String,
    pub color_success: String,
    pub color_error: String,
    pub color_warning: String,
    pub method_get: String,
    pub method_post: String,
    pub method_put: String,
    pub method_delete: String,
    pub method_patch: String,
    pub method_query: String,
    pub method_ws: String,
    pub method_grpc: String,
    pub method_graphql: String,
    pub radius_md: String,
    pub radius_lg: String,
}

impl Default for ThemeUI {
    fn default() -> Self {
        Self {
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
            method_delete: "#FF445E".to_string(),
            method_patch: "#FFB833".to_string(),
            method_query: "#9D74F8".to_string(),
            method_ws: "#792DFF".to_string(),
            method_grpc: "#9D74F8".to_string(),
            method_graphql: "#E30372".to_string(),
            radius_md: "8px".to_string(),
            radius_lg: "10px".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub enum ThemeVariant {
    Dark,
    Light,
}

impl Default for ThemeVariant {
    fn default() -> Self {
        Self::Dark
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct ThemeSyntax {
    pub comment: String,
    pub property: String,
    pub string: String,
    pub number: String,
    pub null: String,
    pub function: String,
    pub variable: String,
    pub attribute: String,
    pub class_name: String,
    pub boolean: String,
    pub keyword: String,
    pub punctuation: String,
    pub operator: String,
}

impl Default for ThemeSyntax {
    fn default() -> Self {
        Self {
            comment: "#8B949E".to_string(),
            property: "#79C0FF".to_string(),
            string: "#7EE787".to_string(),
            number: "#D2A8FF".to_string(),
            null: "#FFAB70".to_string(),
            function: "#FFAB70".to_string(),
            variable: "#FFAB70".to_string(),
            attribute: "#FFAB70".to_string(),
            class_name: "#FFAB70".to_string(),
            boolean: "#FFAB70".to_string(),
            keyword: "#FF7B72".to_string(),
            punctuation: "#C9D1D9".to_string(),
            operator: "#F2CC60".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct ThemeTokens {
    pub ui: ThemeUI,
    pub syntax: ThemeSyntax,
}

impl Default for ThemeTokens {
    fn default() -> Self {
        Self {
            ui: ThemeUI::default(),
            syntax: ThemeSyntax::default(),
        }
    }
}

pub fn default_schema() -> String {
    "https://veyak.iamdhakrey.dev/schemas/themes/1.0.0.json".to_string()
}

fn default_theme_version() -> String {
    "1.0.0".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct Theme {
    #[serde(rename = "$schema", default = "default_schema")]
    pub schema: String,
    pub id: String,
    pub name: String,
    #[serde(default = "default_theme_version")]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub repository: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub variant: ThemeVariant,
    #[serde(default)]
    pub is_builtin: bool,
    #[serde(default)]
    pub tokens: ThemeTokens,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            schema: default_schema(),
            id: "custom-theme".to_string(),
            name: "Custom Theme".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            author: String::new(),
            repository: String::new(),
            license: "MIT".to_string(),
            tags: Vec::new(),
            variant: ThemeVariant::Dark,
            is_builtin: false,
            tokens: ThemeTokens::default(),
        }
    }
}

// ---------------------------------------------------------------------
// Plugins
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub enum PluginHook {
    PreRequest,
    PostResponse,
}

/// `manifest.json` shape every plugin folder must provide.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    /// Path to the JS entry file, relative to the plugin's own folder.
    pub entry: String,
    /// Which lifecycle hooks this plugin implements. The entry script is
    /// expected to define a top-level function per hook it declares here
    /// — `preRequest(ctx)` and/or `postResponse(ctx)`.
    pub hooks: Vec<PluginHook>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct PluginRecord {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
    pub install_path: String,
    pub hooks: Vec<PluginHook>,
    pub installed_at: String,
}

// ---------------------------------------------------------------------
// Active app state (last-selected workspace / environment / theme)
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "models.ts")]
#[serde(rename_all = "camelCase")]
pub struct ActiveState {
    pub active_workspace_id: Option<String>,
    pub active_environment_id: Option<String>,
    pub active_theme_id: Option<String>,
    pub active_collection_id: Option<String>,
    pub active_folder_id: Option<String>,
    pub active_item_id: Option<String>,
}

impl Default for ActiveState {
    fn default() -> Self {
        Self {
            active_workspace_id: None,
            active_environment_id: None,
            active_theme_id: None,
            active_collection_id: None,
            active_folder_id: None,
            active_item_id: None,
        }
    }
}

// --------------------------------------------------------------------
// Websocket
// --------------------------------------------------------------------

/// A saved message template the user creates for reuse — persisted to
/// YAML alongside the request that owns it.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct WsSavedMessage {
    pub id: String,
    pub name: String,
    pub data: String,
}

/// Payload emitted over the Tauri event channel for each WS frame
/// (both sent and received).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct WsEvent {
    pub connection_id: String,
    pub direction: String,
    pub data: String,
    pub timestamp: String,
}

// -------------------------------------------------------
// gRPC
// -------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub enum GrpcMethodType {
    #[serde(rename = "Unary")]
    Unary,
    #[serde(rename = "ClientStreaming")]
    ClientStreaming,
    #[serde(rename = "ServerStreaming")]
    ServerStreaming,
    #[serde(rename = "BidirectionalStreaming")]
    BidirectionalStreaming,
}

impl GrpcMethodType {
    pub fn as_reqwest(&self) -> reqwest::Method {
        match self {
            GrpcMethodType::Unary => reqwest::Method::GET,
            GrpcMethodType::ClientStreaming => reqwest::Method::POST,
            GrpcMethodType::ServerStreaming => reqwest::Method::PUT,
            GrpcMethodType::BidirectionalStreaming => reqwest::Method::PATCH,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            GrpcMethodType::Unary => "GET",
            GrpcMethodType::ClientStreaming => "POST",
            GrpcMethodType::ServerStreaming => "PUT",
            GrpcMethodType::BidirectionalStreaming => "PATCH",
        }
    }
}

impl FromStr for GrpcMethodType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Case-insensitive matching
        match s.to_uppercase().as_str() {
            "UNARY" => Ok(GrpcMethodType::Unary),
            "CLIENTSTREAMING" => Ok(GrpcMethodType::ClientStreaming),
            "SERVERSTREAMING" => Ok(GrpcMethodType::ServerStreaming),
            "BIDIRECTIONALSTREAMING" => Ok(GrpcMethodType::BidirectionalStreaming),
            _ => Err(format!("Invalid gRPC method: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub enum GrpcEventType {
    Info,
    Error,
    ClientMessage,
    ServerMessage,
    ConnectionStart,
    ConnectionEnd,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "models.ts")]
pub enum GrpcStreamType {
    Unary,
    ServerStream,
    ClientStream,
    BidiStream,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct GrpcMethod {
    pub name: String,
    pub full_name: String,
    pub request_type: String,
    pub response_type: String,
    pub stream_type: GrpcStreamType,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct GrpcService {
    pub name: String,
    pub full_name: String,
    pub methods: Vec<GrpcMethod>,
}

impl Default for GrpcEventType {
    fn default() -> Self {
        GrpcEventType::Info
    }
}

/// Represents an imported `.proto` file in the workspace
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct ProtoFile {
    pub id: String,
    pub name: String,
    /// Absolute path to the .proto file on disk
    pub path: String,
    /// Import paths needed by `protoc` (the `-I` flags)
    pub include_dirs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct GrpcRequest {
    pub id: String,
    #[serde(default)]
    pub collection_id: String,
    #[serde(default)]
    pub folder_id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub sort_order: i64,
    /// e.g. "grpc.postman-echo.com:443"
    pub url: String,
    /// e.g. "helloworld.Greeter"
    pub service: String,
    /// e.g. "SayHello"
    pub method: String,
    pub method_type: GrpcMethodType,

    /// Equivalent to HTTP headers
    pub metadata: Vec<KeyValueRow>,
    pub auth: AuthConfig,

    /// JSON representation of the Protobuf message payload
    pub message: String,

    /// If true, use server reflection instead of local proto files
    pub use_reflection: bool,
    /// IDs of `ProtoFile` records to compile against if reflection is false
    pub proto_file_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct GrpcResponse {
    /// gRPC status code (e.g., 0 for OK, 14 for Unavailable)
    pub status: u16,
    pub status_text: String,
    pub time_ms: u128,
    pub size_bytes: u64,
    /// Response metadata (including trailing metadata)
    pub metadata: BTreeMap<String, String>,
    /// Decoded JSON representation of the returned Protobuf message
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct GrpcStreamEvent {
    pub connection_id: String,
    pub direction: String, // "Sent" | "Received"
    pub message: String,
    pub timestamp: String,
}

// -------------------------------------------------------
// GraphQL
// -------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub enum GraphQlRequestType {
    #[default]
    Query,
    Mutation,
    Subscription,
}

/// A persisted GraphQL request item stored in a collection.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct GraphQlRequest {
    pub id: String,
    #[serde(default)]
    pub collection_id: String,
    #[serde(default)]
    pub folder_id: Option<String>,
    pub name: String,
    #[serde(default = "default_graphql_method")]
    pub method: String,
    #[serde(default)]
    pub sort_order: i64,
    /// The GraphQL endpoint URL
    pub url: String,
    /// The query / mutation / subscription document text
    pub query: String,
    /// JSON-encoded variables object (empty string = no variables)
    #[serde(default)]
    pub variables: String,
    #[serde(default)]
    #[ts(optional)]
    pub operation_name: Option<String>,
    pub headers: Vec<KeyValueRow>,
    pub auth: AuthConfig,
    pub request_type: GraphQlRequestType,
}

fn default_graphql_method() -> String {
    "GRAPHQL".to_string()
}

impl Default for GraphQlRequest {
    fn default() -> Self {
        Self {
            id: String::new(),
            collection_id: String::new(),
            folder_id: None,
            name: "Untitled GraphQL".to_string(),
            method: "GRAPHQL".to_string(),
            sort_order: 0,
            url: String::new(),
            query: String::new(),
            variables: String::new(),
            operation_name: None,
            headers: Vec::new(),
            auth: AuthConfig::default(),
            request_type: GraphQlRequestType::Query,
        }
    }
}

/// Response returned by `graphql_execute`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct GraphQlResponse {
    pub status: u16,
    pub status_text: String,
    pub time_ms: u128,
    pub size_bytes: u64,
    pub headers: BTreeMap<String, String>,
    /// Pretty-printed `data` field, or `None` if absent
    #[ts(optional)]
    pub data: Option<String>,
    /// Pretty-printed `errors` array, or `None` if absent
    #[ts(optional)]
    pub errors: Option<String>,
    /// Pretty-printed `extensions` object, or `None` if absent
    #[ts(optional)]
    pub extensions: Option<String>,
}

/// An event emitted over the Tauri `graphql://event` channel
/// for each subscription message (data, error, complete, connecting).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct GraphQlSubscriptionEvent {
    pub connection_id: String,
    /// "data" | "error" | "complete" | "connecting"
    pub event_type: String,
    /// JSON-stringified payload
    pub payload: String,
    pub timestamp: String,
}

// ── Schema types (returned by introspection) ──────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct GraphQlTypeRef {
    pub kind: String,
    #[ts(optional)]
    pub name: Option<String>,
    #[ts(optional)]
    pub of_type: Option<Box<GraphQlTypeRef>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct GraphQlArg {
    pub name: String,
    #[ts(optional)]
    pub description: Option<String>,
    pub type_ref: GraphQlTypeRef,
    #[ts(optional)]
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct GraphQlField {
    pub name: String,
    #[ts(optional)]
    pub description: Option<String>,
    pub type_ref: GraphQlTypeRef,
    pub args: Vec<GraphQlArg>,
    pub is_deprecated: bool,
    #[ts(optional)]
    pub deprecation_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct GraphQlEnumValue {
    pub name: String,
    #[ts(optional)]
    pub description: Option<String>,
    pub is_deprecated: bool,
    #[ts(optional)]
    pub deprecation_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct GraphQlSchemaType {
    pub name: String,
    /// OBJECT | SCALAR | INTERFACE | UNION | ENUM | INPUT_OBJECT
    pub kind: String,
    #[ts(optional)]
    pub description: Option<String>,
    pub fields: Vec<GraphQlField>,
    pub input_fields: Vec<GraphQlArg>,
    pub enum_values: Vec<GraphQlEnumValue>,
}

/// The top-level schema returned by introspection.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "models.ts")]
pub struct GraphQlSchema {
    #[ts(optional)]
    pub query_type: Option<String>,
    #[ts(optional)]
    pub mutation_type: Option<String>,
    #[ts(optional)]
    pub subscription_type: Option<String>,
    pub types: Vec<GraphQlSchemaType>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_partial_deserialization_json() {
        let raw_json = r##"{
            "id": "my-legacy-theme",
            "name": "My Legacy Theme",
            "tokens": {
                "ui": {
                    "colorBg": "#121212",
                    "colorPrimary": "#FF00FF"
                },
                "syntax": {
                    "keyword": "#FF0000"
                }
            }
        }"##;

        let theme: Theme =
            serde_json::from_str(raw_json).expect("should deserialize partial JSON theme");
        assert_eq!(theme.id, "my-legacy-theme");
        assert_eq!(theme.name, "My Legacy Theme");
        assert_eq!(theme.tokens.ui.color_bg, "#121212");
        assert_eq!(theme.tokens.ui.color_primary, "#FF00FF");
        // Check default fallbacks for missing tokens
        assert_eq!(theme.tokens.ui.color_panel, "#161B22");
        assert_eq!(theme.tokens.ui.method_graphql, "#E30372");
        assert_eq!(theme.tokens.syntax.keyword, "#FF0000");
        assert_eq!(theme.tokens.syntax.comment, "#8B949E");
        assert_eq!(theme.version, "1.0.0");
        assert_eq!(theme.license, "");
        assert_eq!(theme.is_builtin, false);
    }

    #[test]
    fn test_theme_partial_deserialization_yaml() {
        let raw_yaml = r##"
id: minimal-yaml-theme
name: Minimal YAML
tokens:
  ui:
    colorBg: "#000000"
"##;

        let theme: Theme =
            serde_yaml::from_str(raw_yaml).expect("should deserialize partial YAML theme");
        assert_eq!(theme.id, "minimal-yaml-theme");
        assert_eq!(theme.tokens.ui.color_bg, "#000000");
        assert_eq!(theme.tokens.ui.color_panel, "#161B22");
        assert_eq!(theme.tokens.syntax.keyword, "#FF7B72");
    }
}
