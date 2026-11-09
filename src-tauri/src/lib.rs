//! ENPost backend. Layers (see docs/architecture.md):
//! commands (IPC) → modules (services) → database (repositories), with core (errors, state)
//! and security (redaction, later secrets) shared by all.

pub mod commands;
pub mod core;
pub mod database;
pub mod modules;
pub mod security;
pub mod utils;

use crate::core::state::AppState;
use crate::database::Database;
use crate::modules::settings::SettingsService;
use tauri::Manager;

pub fn run() {
    let result = tauri::Builder::default()
        // A second launch focuses the running window instead of starting another copy.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(if cfg!(debug_assertions) { log::LevelFilter::Debug } else { log::LevelFilter::Info })
                // Dependencies log request details at debug level; keep them out of the file.
                .level_for("sqlx", log::LevelFilter::Warn)
                .level_for("tao", log::LevelFilter::Warn)
                // Request URLs and headers must never reach the log file.
                .level_for("reqwest", log::LevelFilter::Warn)
                .level_for("hyper", log::LevelFilter::Warn)
                .level_for("hyper_util", log::LevelFilter::Warn)
                .level_for("rustls", log::LevelFilter::Warn)
                .max_file_size(2_000_000)
                .build(),
        )
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let db = tauri::async_runtime::block_on(Database::open(&data_dir.join("enpost.db"))).map_err(|e| Box::new(std::io::Error::other(e.to_string())))?;
            let settings = tauri::async_runtime::block_on(SettingsService::load(db.clone())).map_err(|e| Box::new(std::io::Error::other(e.to_string())))?;
            let bodies_dir = data_dir.join("bodies");
            let retention = tauri::async_runtime::block_on(settings.get()).history_retention_days;
            if let Err(e) = tauri::async_runtime::block_on(modules::history::prune(&db, retention)) {
                log::warn!("history cleanup failed: {e}");
            }
            app.manage(AppState { collections: modules::collections::CollectionsService::new(db.clone()), db, settings, http: modules::http::HttpEngine::new(bodies_dir.clone()), data_dir, bodies_dir });
            log::info!("ENPost {} started", app.package_info().version);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::app_info,
            commands::app::open_app_folder,
            commands::app::open_link,
            commands::update::check_update,
            commands::update::install_update,
            commands::update::open_release_page,
            commands::update::restart_app,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::reset_settings,
            commands::http::send_request,
            commands::http::cancel_request,
            commands::http::save_response_body,
            commands::history::list_history,
            commands::history::clear_history,
            commands::collections::list_workspaces,
            commands::collections::create_workspace,
            commands::collections::rename_workspace,
            commands::collections::delete_workspace,
            commands::collections::workspace_tree,
            commands::collections::create_collection,
            commands::collections::create_folder,
            commands::collections::create_request,
            commands::collections::get_request,
            commands::collections::save_request,
            commands::collections::duplicate_request,
            commands::collections::rename_item,
            commands::collections::delete_item,
            commands::collections::set_favorite,
            commands::collections::move_item,
        ])
        .run(tauri::generate_context!());
    if let Err(e) = result {
        eprintln!("ENPost could not start: {e}");
        std::process::exit(1);
    }
}
