use serde::{Deserialize, Serialize};
use tauri::State;
use crate::app_state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct AnnotationRequest {
    pub item_id: String,
    pub page: usize,
    pub range: (f64, f64, f64, f64),
    pub kind: String,
    pub text: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnnotationResponse {
    pub id: String,
    pub item_id: String,
    pub page: usize,
    pub range: (f64, f64, f64, f64),
    pub kind: String,
    pub text: String,
    pub color: String,
    pub created_at: i64,
}

#[tauri::command]
pub async fn add_annotation(
    annotation: AnnotationRequest,
    state: State<'_, AppState>,
) -> Result<AnnotationResponse, String> {
    tracing::info!("Adding annotation to item {}", annotation.item_id);

    let db_path = state.db_path.clone();

    let result = tauri::async_runtime::spawn_blocking(move || {
        use blinker_core_annot::{Annotation, AnnotationKind, AnnotationManager};

        let manager = AnnotationManager::new(&db_path)
            .map_err(|e| e.to_string())?;

        let kind = match annotation.kind.as_str() {
            "note" => AnnotationKind::Note,
            "bookmark" => AnnotationKind::Bookmark,
            _ => AnnotationKind::Highlight,
        };

        let annot = Annotation {
            id: String::new(), // Will be generated
            item_id: annotation.item_id.clone(),
            page: annotation.page,
            range: annotation.range,
            kind,
            text: annotation.text.clone(),
            color: annotation.color.clone(),
            created_at: 0,
            modified_at: 0,
        };

        let id = manager.add_annotation(annot)
            .map_err(|e| e.to_string())?;

        Ok::<_, String>((id, annotation))
    })
    .await
    .map_err(|e| e.to_string())??;

    let (id, req) = result;

    Ok(AnnotationResponse {
        id,
        item_id: req.item_id,
        page: req.page,
        range: req.range,
        kind: req.kind,
        text: req.text,
        color: req.color,
        created_at: chrono::Utc::now().timestamp(),
    })
}

#[tauri::command]
pub async fn list_annotations(
    item_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<AnnotationResponse>, String> {
    tracing::info!("Listing annotations for item {}", item_id);

    let db_path = state.db_path.clone();

    let annotations = tauri::async_runtime::spawn_blocking(move || {
        use blinker_core_annot::AnnotationManager;

        let manager = AnnotationManager::new(&db_path)
            .map_err(|e| e.to_string())?;

        manager.list_annotations(&item_id)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(annotations.into_iter().map(|a| AnnotationResponse {
        id: a.id,
        item_id: a.item_id,
        page: a.page,
        range: a.range,
        kind: match a.kind {
            blinker_core_annot::AnnotationKind::Highlight => "highlight".to_string(),
            blinker_core_annot::AnnotationKind::Note => "note".to_string(),
            blinker_core_annot::AnnotationKind::Bookmark => "bookmark".to_string(),
        },
        text: a.text,
        color: a.color,
        created_at: a.created_at,
    }).collect())
}

#[tauri::command]
pub async fn delete_annotation(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    tracing::info!("Deleting annotation {}", id);

    let db_path = state.db_path.clone();

    tauri::async_runtime::spawn_blocking(move || {
        use blinker_core_annot::AnnotationManager;

        let manager = AnnotationManager::new(&db_path)
            .map_err(|e| e.to_string())?;

        manager.delete_annotation(&id)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(())
}

#[tauri::command]
pub async fn export_annotations(
    item_id: String,
    format: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    tracing::info!("Exporting annotations for item {} as {}", item_id, format);

    let db_path = state.db_path.clone();

    tauri::async_runtime::spawn_blocking(move || {
        use blinker_core_annot::AnnotationManager;

        let manager = AnnotationManager::new(&db_path)
            .map_err(|e| e.to_string())?;

        match format.as_str() {
            "json" => manager.export_json(&item_id).map_err(|e| e.to_string()),
            "markdown" | "md" => manager.export_markdown(&item_id).map_err(|e| e.to_string()),
            _ => Err(format!("Unsupported format: {}", format)),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}
