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
