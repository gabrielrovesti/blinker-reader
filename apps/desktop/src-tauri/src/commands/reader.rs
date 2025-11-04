use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReaderSession {
    pub id: String,
    pub document_id: String,
    pub current_page: usize,
    pub total_pages: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMatch {
    pub page: usize,
    pub text: String,
    pub position: (f64, f64),
}

#[tauri::command]
pub async fn open_document(id: String) -> Result<ReaderSession, String> {
    // TODO: Implement document opening with blinker-core-render
    tracing::info!("Opening document: {}", id);

    Ok(ReaderSession {
        id: uuid::Uuid::new_v4().to_string(),
        document_id: id,
        current_page: 1,
        total_pages: 0,
    })
}

#[tauri::command]
pub async fn search_document(session_id: String, query: String) -> Result<Vec<SearchMatch>, String> {
    // TODO: Implement in-document search
    tracing::info!("Searching in session {}: {}", session_id, query);

    Ok(vec![])
}
