use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanReport {
    pub total: usize,
    pub new: usize,
    pub updated: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryItem {
    pub id: String,
    pub path: String,
    pub title: String,
    pub author: Option<String>,
    pub file_type: String,
    pub hash: String,
    pub tags: Vec<String>,
}

#[tauri::command]
pub async fn scan_library(paths: Vec<String>) -> Result<ScanReport, String> {
    // TODO: Implement library scanning with blinker-core-library
    tracing::info!("Scanning library paths: {:?}", paths);

    Ok(ScanReport {
        total: 0,
        new: 0,
        updated: 0,
        errors: vec![],
    })
}

#[tauri::command]
pub async fn query_library(filters: serde_json::Value) -> Result<Vec<LibraryItem>, String> {
    // TODO: Implement library query with FTS5
    tracing::info!("Querying library with filters: {:?}", filters);

    Ok(vec![])
}

#[tauri::command]
pub async fn update_metadata(id: String, fields: serde_json::Value) -> Result<(), String> {
    // TODO: Implement metadata update
    tracing::info!("Updating metadata for {}: {:?}", id, fields);

    Ok(())
}
