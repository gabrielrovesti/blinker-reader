/// Annotation management: highlights, notes, bookmarks.
///
/// This crate handles:
/// - Creating and storing annotations
/// - Exporting to JSON/Markdown
/// - Future: write-back to PDF annotations

use blinker_core_common::{BlinkerError, Result};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub item_id: String,
    pub page: usize,
    pub range: (f64, f64, f64, f64), // x, y, width, height
    pub kind: AnnotationKind,
    pub text: String,
    pub color: String,
    pub created_at: i64,
    pub modified_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationKind {
    Highlight,
    Note,
    Bookmark,
}

impl AnnotationKind {
    fn to_string(&self) -> String {
        match self {
            Self::Highlight => "highlight".to_string(),
            Self::Note => "note".to_string(),
            Self::Bookmark => "bookmark".to_string(),
        }
    }

    fn from_string(s: &str) -> Self {
        match s {
            "note" => Self::Note,
            "bookmark" => Self::Bookmark,
            _ => Self::Highlight,
        }
    }
}

pub struct AnnotationManager {
    conn: Connection,
}

impl AnnotationManager {
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)
            .map_err(|e| BlinkerError::Database(format!("Failed to open database: {}", e)))?;

        Ok(Self { conn })
    }

    fn now_secs() -> i64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    }

    pub fn add_annotation(&self, mut annotation: Annotation) -> Result<String> {
        let now = Self::now_secs();

        // Generate ID if not provided
        if annotation.id.is_empty() {
            annotation.id = uuid::Uuid::new_v4().to_string();
        }

        annotation.created_at = now;
        annotation.modified_at = now;

        self.conn.execute(
            "INSERT INTO annotation (
                id, item_id, page,
                range_x, range_y, range_width, range_height,
                kind, text, color,
                created_at, modified_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                annotation.id,
                annotation.item_id,
                annotation.page as i64,
                annotation.range.0,
                annotation.range.1,
                annotation.range.2,
                annotation.range.3,
                annotation.kind.to_string(),
                annotation.text,
                annotation.color,
                annotation.created_at,
                annotation.modified_at,
            ],
        )
        .map_err(|e| BlinkerError::Database(format!("Failed to insert annotation: {}", e)))?;

        Ok(annotation.id)
    }

    pub fn list_annotations(&self, item_id: &str) -> Result<Vec<Annotation>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, item_id, page, range_x, range_y, range_width, range_height,
                    kind, text, color, created_at, modified_at
             FROM annotation
             WHERE item_id = ?1
             ORDER BY page, created_at"
        )
        .map_err(|e| BlinkerError::Database(format!("Failed to prepare query: {}", e)))?;

        let rows = stmt.query_map(params![item_id], |row| {
            Ok(Annotation {
                id: row.get(0)?,
                item_id: row.get(1)?,
                page: row.get::<_, i64>(2)? as usize,
                range: (
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ),
                kind: AnnotationKind::from_string(&row.get::<_, String>(7)?),
                text: row.get(8)?,
                color: row.get(9)?,
                created_at: row.get(10)?,
                modified_at: row.get(11)?,
            })
        })
        .map_err(|e| BlinkerError::Database(format!("Failed to query annotations: {}", e)))?;

        let mut annotations = Vec::new();
        for row in rows {
            annotations.push(
                row.map_err(|e| BlinkerError::Database(format!("Failed to read row: {}", e)))?
            );
        }

        Ok(annotations)
    }

    pub fn delete_annotation(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM annotation WHERE id = ?1", params![id])
            .map_err(|e| BlinkerError::Database(format!("Failed to delete annotation: {}", e)))?;

        Ok(())
    }

    pub fn export_json(&self, item_id: &str) -> Result<String> {
        let annotations = self.list_annotations(item_id)?;
        serde_json::to_string_pretty(&annotations)
            .map_err(|e| BlinkerError::Parsing(format!("Failed to serialize to JSON: {}", e)))
    }

    pub fn export_markdown(&self, item_id: &str) -> Result<String> {
        let annotations = self.list_annotations(item_id)?;

        let mut markdown = String::new();
        markdown.push_str("# Annotations\n\n");

        for annotation in annotations {
            markdown.push_str(&format!("## Page {}\n", annotation.page));
            markdown.push_str(&format!("**Type:** {}\n", annotation.kind.to_string()));
            markdown.push_str(&format!("**Text:** {}\n", annotation.text));
            markdown.push_str(&format!("**Color:** {}\n", annotation.color));
            markdown.push_str("\n---\n\n");
        }

        Ok(markdown)
    }
}

/// Store abstraction for annotations.
pub trait AnnotationStore {
    fn add(&self, annotation: Annotation) -> Result<String>;
    fn list(&self, item_id: &str) -> Result<Vec<Annotation>>;
    fn delete(&self, id: &str) -> Result<()>;
    fn export_json(&self, item_id: &str) -> Result<String>;
    fn export_markdown(&self, item_id: &str) -> Result<String>;
}

impl AnnotationStore for AnnotationManager {
    fn add(&self, annotation: Annotation) -> Result<String> {
        self.add_annotation(annotation)
    }

    fn list(&self, item_id: &str) -> Result<Vec<Annotation>> {
        self.list_annotations(item_id)
    }

    fn delete(&self, id: &str) -> Result<()> {
        self.delete_annotation(id)
    }

    fn export_json(&self, item_id: &str) -> Result<String> {
        self.export_json(item_id)
    }

    fn export_markdown(&self, item_id: &str) -> Result<String> {
        self.export_markdown(item_id)
    }
}
