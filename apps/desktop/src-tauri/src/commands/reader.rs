use serde::{Deserialize, Serialize};
use tauri::State;
use blinker_core_library::LibraryStore;
use blinker_core_render::AnyRenderer;
use crate::app_state::{AppState, ReaderSession};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReaderSessionResponse {
    pub session_id: String,
    pub document_id: String,
    pub current_page: usize,
    pub total_pages: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMatch {
    pub page: usize,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenderedPageResponse {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[tauri::command]
pub async fn open_document(id: String, state: State<'_, AppState>) -> Result<ReaderSessionResponse, String> {
    tracing::info!("Opening document: {}", id);

    let db_path = state.db_path.clone();

    // Spawn blocking task for file I/O
    let result = tauri::async_runtime::spawn_blocking(move || {
        // Get the document from the database
        let db = blinker_core_library::LibraryDatabase::new(&db_path)
            .map_err(|e| e.to_string())?;

        let item = db.get_item(&id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Document not found: {}", id))?;

        // Open the renderer
        let renderer = AnyRenderer::open_for(&item.file_path)
            .map_err(|e| e.to_string())?;

        // Get page count
        let total_pages = renderer.page_count()
            .map_err(|e| e.to_string())?;

        Ok::<_, String>((renderer, item.id, total_pages))
    })
    .await
    .map_err(|e| e.to_string())??;

    let (renderer, item_id, total_pages) = result;

    // Create a new session
    let session_id = uuid::Uuid::new_v4().to_string();

    let session = ReaderSession {
        renderer,
        item_id: item_id.clone(),
    };

    // Store the session
    {
        let mut sessions = state.sessions.lock().unwrap();
        sessions.insert(session_id.clone(), session);
    }

    Ok(ReaderSessionResponse {
        session_id,
        document_id: item_id,
        current_page: 1,
        total_pages,
    })
}

#[tauri::command]
pub async fn render_page(
    session_id: String,
    page: usize,
    state: State<'_, AppState>,
) -> Result<RenderedPageResponse, String> {
    tracing::info!("Rendering page {} for session {}", page, session_id);

    // Get the session
    let sessions = state.sessions.lock().unwrap();
    let session = sessions.get(&session_id)
        .ok_or_else(|| format!("Session not found: {}", session_id))?;

    // Render the page
    let rendered = session.renderer.render_page(page)
        .map_err(|e| e.to_string())?;

    Ok(RenderedPageResponse {
        width: rendered.width,
        height: rendered.height,
        data: rendered.pixels,
    })
}

#[tauri::command]
pub async fn search_document(
    session_id: String,
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<SearchMatch>, String> {
    tracing::info!("Searching in session {}: {}", session_id, query);

    // Get the session
    let sessions = state.sessions.lock().unwrap();
    let session = sessions.get(&session_id)
        .ok_or_else(|| format!("Session not found: {}", session_id))?;

    // Search (limit to 100 results)
    let matches = session.renderer.search(&query, 100)
        .map_err(|e| e.to_string())?;

    Ok(matches.into_iter().map(|m| SearchMatch {
        page: m.page,
        text: m.text,
    }).collect())
}

#[tauri::command]
pub async fn close_session(session_id: String, state: State<'_, AppState>) -> Result<(), String> {
    tracing::info!("Closing session {}", session_id);

    let mut sessions = state.sessions.lock().unwrap();
    sessions.remove(&session_id);

    Ok(())
}
