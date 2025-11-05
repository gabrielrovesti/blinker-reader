/// Annotation management: highlights, notes, bookmarks.
///
/// This crate handles:
/// - Creating and storing annotations
/// - Exporting to JSON/Markdown
/// - Future: write-back to PDF annotations

use blinker_core_common::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub item_id: String,
    pub page: usize,
    pub range: (f64, f64, f64, f64),
    pub kind: AnnotationKind,
    pub text: String,
    pub color: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnotationKind {
    Highlight,
    Note,
    Bookmark,
}

pub struct AnnotationManager;

impl AnnotationManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn add_annotation(&self, _annotation: Annotation) -> Result<String> {
        // TODO: Store annotation in database
        Ok(String::new())
    }

    pub fn list_annotations(&self, _item_id: &str) -> Result<Vec<Annotation>> {
        // TODO: Query annotations for item
        Ok(vec![])
    }

    pub fn export_json(&self, _item_id: &str) -> Result<String> {
        // TODO: Export to JSON
        Ok(String::new())
    }

    pub fn export_markdown(&self, _item_id: &str) -> Result<String> {
        // TODO: Export to Markdown
        Ok(String::new())
    }
}

/// Store abstraction for annotations.
pub trait AnnotationStore {
    fn add(&self, annotation: Annotation) -> Result<String>;
    fn list(&self, item_id: &str) -> Result<Vec<Annotation>>;
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

    fn export_json(&self, item_id: &str) -> Result<String> {
        self.export_json(item_id)
    }

    fn export_markdown(&self, item_id: &str) -> Result<String> {
        self.export_markdown(item_id)
    }
}
