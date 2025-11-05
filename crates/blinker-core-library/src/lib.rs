/// Library management: indexing, metadata, tags, and search.
///
/// This crate handles:
/// - File-system scanning and watching
/// - BLAKE3 hashing for deduplication
/// - SQLite database with FTS5 for search
/// - Metadata extraction and management

pub mod scanner;
pub mod database;
pub mod metadata;

pub use scanner::LibraryScanner;
pub use database::LibraryDatabase;

use blinker_core_common::{types::Metadata, Result};
use std::path::PathBuf;

/// Representation of a library item in memory.
#[derive(Debug, Clone)]
pub struct LibraryItem {
    pub id: String,
    pub file_path: PathBuf,
    pub file_hash: String,
    pub file_type: String,
    pub file_size: u64,
    pub metadata: Metadata,
    pub tags: Vec<String>,
}

/// Query parameters for library search.
#[derive(Debug, Clone, Default)]
pub struct LibraryQuery {
    pub text: Option<String>,
    pub file_types: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<usize>,
}

/// Store operations required by higher layers.
pub trait LibraryStore {
    fn add_or_update_path(&self, path: &std::path::Path) -> Result<AddOutcome>;
    fn get_item(&self, id: &str) -> Result<Option<LibraryItem>>;
    fn query(&self, query: &LibraryQuery) -> Result<Vec<LibraryItem>>;
}

#[derive(Debug, Clone)]
pub enum AddOutcome {
    Created { id: String },
    Updated { id: String },
    Unchanged { id: String },
}
