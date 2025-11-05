use serde::{Deserialize, Serialize};
use tauri::State;
use crate::app_state::AppState;

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
pub async fn scan_library(state: State<'_, AppState>, paths: Vec<String>) -> Result<ScanReport, String> {
    tracing::info!("Scanning library paths: {:?}", paths);
    let db = blinker_core_library::LibraryDatabase::new(&state.db_path)
        .map_err(|e| e.to_string())?;
    let scanner = blinker_core_library::LibraryScanner::new();
    let pbufs: Vec<std::path::PathBuf> = paths.into_iter().map(Into::into).collect();
    let prefs: Vec<&std::path::Path> = pbufs.iter().map(|p| p.as_path()).collect();
    let rep = scanner.scan_paths(&db, &prefs).await.map_err(|e| e.to_string())?;
    Ok(ScanReport { total: rep.total, new: rep.new, updated: rep.updated, errors: rep.errors })
}

#[tauri::command]
pub async fn query_library(state: State<'_, AppState>, filters: serde_json::Value) -> Result<Vec<LibraryItem>, String> {
    tracing::info!("Querying library with filters: {:?}", filters);
    let db = blinker_core_library::LibraryDatabase::new(&state.db_path)
        .map_err(|e| e.to_string())?;

    let mut q = blinker_core_library::LibraryQuery::default();
    if let Some(text) = filters.get("text").and_then(|v| v.as_str()) { q.text = Some(text.to_string()); }
    if let Some(limit) = filters.get("limit").and_then(|v| v.as_u64()) { q.limit = Some(limit as usize); }
    if let Some(types) = filters.get("file_types").and_then(|v| v.as_array()) {
        q.file_types = Some(types.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
    }

    let items = db.query(&q).map_err(|e| e.to_string())?;
    let out = items.into_iter().map(|it| LibraryItem {
        id: it.id,
        path: it.file_path.to_string_lossy().to_string(),
        title: it.metadata.title,
        author: it.metadata.author,
        file_type: it.file_type,
        hash: it.file_hash,
        tags: it.tags,
    }).collect();
    Ok(out)
}

#[tauri::command]
pub async fn update_metadata(id: String, fields: serde_json::Value) -> Result<(), String> {
    // TODO: Implement metadata update
    tracing::info!("Updating metadata for {}: {:?}", id, fields);

    Ok(())
}
