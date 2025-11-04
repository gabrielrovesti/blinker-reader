use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Annotation {
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
    item_id: String,
    page: usize,
    range: (f64, f64, f64, f64),
    kind: String,
    text: String,
    color: String,
) -> Result<Annotation, String> {
    // TODO: Implement annotation creation with blinker-core-annot
    tracing::info!("Adding annotation to item {}", item_id);

    Ok(Annotation {
        id: uuid::Uuid::new_v4().to_string(),
        item_id,
        page,
        range,
        kind,
        text,
        color,
        created_at: chrono::Utc::now().timestamp(),
    })
}

#[tauri::command]
pub async fn list_annotations(item_id: String) -> Result<Vec<Annotation>, String> {
    // TODO: Implement annotation listing
    tracing::info!("Listing annotations for item {}", item_id);

    Ok(vec![])
}
