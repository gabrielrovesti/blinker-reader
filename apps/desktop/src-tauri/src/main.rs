// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing_subscriber;
use tauri::Manager;

mod commands;
mod app_state;

use std::path::PathBuf;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .setup(|app| {
            // Resolve app data directory and prepare DB path
            let app_dir: PathBuf = app
                .path_resolver()
                .app_data_dir()
                .unwrap_or_else(|| app.path_resolver().app_config_dir().unwrap());
            std::fs::create_dir_all(&app_dir).ok();
            let db_path = app_dir.join("blinker.db");
            // Ensure DB is initialized once
            let _ = blinker_core_library::LibraryDatabase::new(&db_path);
            app.manage(app_state::AppState::new(db_path));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::library::scan_library,
            commands::library::query_library,
            commands::library::update_metadata,
            commands::reader::open_document,
            commands::reader::search_document,
            commands::annotations::add_annotation,
            commands::annotations::list_annotations,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Blinker Reader");
}
