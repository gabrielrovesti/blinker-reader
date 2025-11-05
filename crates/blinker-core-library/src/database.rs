use blinker_core_common::Result;
use std::path::Path;
use crate::{LibraryItem, LibraryQuery, LibraryStore};

pub struct LibraryDatabase {
    // TODO: Add SQLite connection
}

impl LibraryDatabase {
    pub fn new(_path: &str) -> Result<Self> {
        // TODO: Initialize SQLite with schema
        // - library_item table
        // - tag & item_tag tables
        // - reading_state table
        // - annotation table
        // - FTS5 virtual table
        Ok(Self {})
    }

    pub fn migrate(&self) -> Result<()> {
        // TODO: Run migrations
        Ok(())
    }
}

impl LibraryStore for LibraryDatabase {
    fn add_or_update_path(&self, _path: &Path) -> Result<String> {
        // TODO: Hash file, extract metadata, upsert DB, return item id
        Ok(String::new())
    }

    fn get_item(&self, _id: &str) -> Result<Option<LibraryItem>> {
        // TODO: Fetch item and associated tags
        Ok(None)
    }

    fn query(&self, _query: &LibraryQuery) -> Result<Vec<LibraryItem>> {
        // TODO: Full-text search with filters
        Ok(vec![])
    }
}
