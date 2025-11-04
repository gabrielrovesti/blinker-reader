// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing_subscriber;

mod commands;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
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
