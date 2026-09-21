use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, Window};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_oauth::start;
use url::Url;

use crate::commands::collections::{
    clone_collection, create_collection, create_folder, create_request, create_ws_request,
    delete_collection, delete_folder, delete_request, duplicate_request, get_addition_types,
    get_collection_tree, get_collection_trees, get_request, list_collections, move_item,
    rename_collection, rename_folder, rename_request, save_request, set_active_collection,
};
use crate::commands::environments::{
    create_environment, delete_environment, list_environments, list_variables, rename_environment,
    replace_variables, set_active_environment,
};
use crate::commands::settings::{self, get_settings, update_settings};
use crate::commands::workspaces::{
    create_workspace, delete_workspace, get_active_state, get_active_state_full, list_workspaces,
    rename_workspace, set_active_workspace,
};
use crate::http::send_request;
use crate::state::AppState;
use crate::ws::{
    ws_add_saved_message, ws_connect, ws_delete_saved_message, ws_disconnect,
    ws_list_saved_messages, ws_send, ws_update_saved_message,
};
use tokio::sync::Mutex;

use crate::commands::auth::{self, PkceSessionState};
use crate::commands::graphql;
use crate::commands::grpc;

use crate::commands::fonts;
use crate::commands::history::{clear_history, delete_history_entry, list_history};

mod commands;
pub mod db;
mod error;
mod http;
mod models;
mod state;
pub mod ws;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn start_server(window: Window) -> Result<u16, String> {
    start(move |url| {
        // Because of the unprotected localhost port, you must verify the URL here.
        // Preferebly send back only the token, or nothing at all if you can handle everything else in Rust.
        let _ = window.emit("redirect_uri", url);
    })
    .map_err(|err| err.to_string())
}

async fn handle_deep_link(app: &AppHandle, raw_url: &str) {
    let clean_url = raw_url.trim_matches(|c| c == '\'' || c == '"').trim();
    let parsed = match Url::parse(clean_url) {
        Ok(u) => u,
        Err(e) => {
            log::warn!("Failed to parse deep link URL '{}': {}", clean_url, e);
            return;
        }
    };

    log::info!("Handling deep link: {}", parsed);
    if parsed.scheme().eq_ignore_ascii_case("veyak") {
        let is_theme_install = (parsed.domain() == Some("theme")
            || parsed.host_str() == Some("theme")
            || parsed.path().starts_with("/theme"))
            && (parsed.path() == "/install"
                || parsed.path().ends_with("/install")
                || parsed.host_str() == Some("theme"));

        if is_theme_install {
            let mut theme_id = String::new();

            for (key, value) in parsed.query_pairs() {
                match key.as_ref() {
                    "theme_id" | "id" => theme_id = value.to_string(),
                    _ => {}
                }
            }

            if theme_id.is_empty() {
                log::warn!(
                    "Theme install deep-link missing theme_id/id parameter: {}",
                    clean_url
                );
                return;
            }

            // Unminimize & bring window to front
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }

            // Pre-fetch theme preview from registry if possible
            let theme_preview = crate::db::themes::fetch_theme_preview(&theme_id).await.ok();

            // Forward to frontend for confirmation (DO NOT write to disk or apply yet)
            let _ = app.emit(
                "deep-link://theme-install",
                serde_json::json!({
                    "id": theme_id,
                    "theme": theme_preview,
                }),
            );
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            let app_handle = app.clone();
            if let Some(raw) = argv.into_iter().find(|arg| {
                let lower = arg.trim_matches(|c| c == '\'' || c == '"').to_lowercase();
                lower.starts_with("veyak://") || lower.starts_with("veyak:")
            }) {
                let clean_url = raw.trim_matches(|c| c == '\'' || c == '"').to_string();
                tauri::async_runtime::spawn(async move {
                    handle_deep_link(&app_handle, &clean_url).await;
                });
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_oauth::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            #[cfg(any(target_os = "linux", all(debug_assertions, windows)))]
            {
                if let Err(e) = app.deep_link().register_all() {
                    log::warn!("Failed to register deep link schemes: {}", e);
                }
            }

            let data_dir = app_handle
                .path()
                .app_data_dir()
                .expect("resolve app data dir");
            std::fs::create_dir_all(&data_dir).expect("create app data dir");

            let data_dir = veyak_db::init_data_dir(&data_dir).expect("initialize data directory");

            app_handle.manage(AppState {
                data_dir,
                ws_connections: Default::default(),
                grpc_state: Default::default(),
                graphql_state: Default::default(),
            });

            app_handle.manage::<PkceSessionState>(Arc::new(Mutex::new(None)));

            #[cfg(target_os = "macos")]
            if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.set_decorations(true);
            }

            let app_handle_deep = app_handle.clone();
            let _ = app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
                    let app_handle = app_handle_deep.clone();
                    let url_str = url.to_string();
                    tauri::async_runtime::spawn(async move {
                        handle_deep_link(&app_handle, &url_str).await;
                    });
                }
            });

            // Handle cold-start deep links passed via CLI argument
            let initial_url = std::env::args().skip(1).find(|arg| {
                let lower = arg.trim_matches(|c| c == '\'' || c == '"').to_lowercase();
                lower.starts_with("veyak://") || lower.starts_with("veyak:")
            });
            if let Some(url) = initial_url {
                let app_handle_init = app_handle.clone();
                let clean_url = url.trim_matches(|c| c == '\'' || c == '"').to_string();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
                    handle_deep_link(&app_handle_init, &clean_url).await;
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            start_server,
            send_request,
            list_workspaces,
            create_workspace,
            rename_workspace,
            delete_workspace,
            set_active_workspace,
            get_active_state,
            get_active_state_full,
            move_item,
            //  Collection commands
            get_addition_types,
            list_collections,
            get_collection_tree,
            get_collection_trees,
            create_collection,
            rename_collection,
            delete_collection,
            clone_collection,
            set_active_collection,
            // Folder commands
            create_folder,
            rename_folder,
            delete_folder,
            // Request commands
            create_request,
            create_ws_request,
            rename_request,
            delete_request,
            get_request,
            duplicate_request,
            save_request,
            // Settings
            get_settings,
            update_settings,
            // Environment commands
            create_environment,
            rename_environment,
            delete_environment,
            list_environments,
            list_variables,
            replace_variables,
            set_active_environment,
            // WebSocket commands
            ws_connect,
            ws_send,
            ws_disconnect,
            ws_list_saved_messages,
            ws_add_saved_message,
            ws_update_saved_message,
            ws_delete_saved_message,
            // History commands
            // add_entry,
            list_history,
            clear_history,
            delete_history_entry,
            //auth
            auth::get_current_user,
            auth::get_access_token,
            auth::is_authenticated,
            auth::get_auth_state,
            auth::auth_start_login,
            auth::auth_handle_callback,
            auth::auth_logout,
            // gRPC
            grpc::grpc_reflect,
            grpc::grpc_parse_proto,
            grpc::grpc_invoke,
            grpc::grpc_cancel,
            grpc::grpc_send_message,
            // GraphQL
            graphql::graphql_introspect,
            graphql::graphql_execute,
            graphql::graphql_subscribe,
            graphql::graphql_unsubscribe,
            // fonts
            fonts::get_system_fonts,
            // Themes
            settings::list_themes,
            settings::delete_theme,
            settings::set_active_theme,
            settings::fetch_theme_preview,
            settings::install_theme,
            settings::save_theme,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Veyak application");
}
