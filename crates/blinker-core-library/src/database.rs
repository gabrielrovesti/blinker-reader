use blinker_core_common::Result;

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
