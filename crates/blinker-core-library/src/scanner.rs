use blinker_core_common::Result;
use std::path::Path;

pub struct LibraryScanner {
    // TODO: Add filesystem watcher
}

impl LibraryScanner {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn scan_paths(&self, _paths: &[&Path]) -> Result<ScanReport> {
        // TODO: Implement scanning
        // - Walk directories
        // - Filter by supported extensions
        // - Calculate BLAKE3 hashes
        // - Extract metadata
        // - Update database
        tracing::info!("Scanning library paths");

        Ok(ScanReport {
            total: 0,
            new: 0,
            updated: 0,
            errors: vec![],
        })
    }
}

pub struct ScanReport {
    pub total: usize,
    pub new: usize,
    pub updated: usize,
    pub errors: Vec<String>,
}
