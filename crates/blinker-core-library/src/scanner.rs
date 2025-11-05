use blinker_core_common::Result;
use std::path::{Path, PathBuf};
use super::{LibraryStore, AddOutcome};

pub struct LibraryScanner;

impl LibraryScanner {
    pub fn new() -> Self { Self }

    pub async fn scan_paths<S: LibraryStore>(&self, store: &S, paths: &[&Path]) -> Result<ScanReport> {
        tracing::info!("Scanning library paths: {}", paths.len());

        let mut total = 0usize;
        let mut new = 0usize;
        let mut updated = 0usize;
        let mut errors: Vec<String> = vec![];

        let mut queue: Vec<PathBuf> = paths.iter().map(|p| (*p).to_path_buf()).collect();
        while let Some(p) = queue.pop() {
            match std::fs::metadata(&p) {
                Ok(m) if m.is_dir() => {
                    if let Ok(rd) = std::fs::read_dir(&p) {
                        for e in rd.flatten() { queue.push(e.path()); }
                    }
                }
                Ok(m) if m.is_file() => {
                    // Filter by supported extensions (simple check)
                    if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                        if blinker_core_common::types::DocumentFormat::from_extension(ext).is_some() {
                            total += 1;
                            match store.add_or_update_path(&p) {
                                Ok(AddOutcome::Created { .. }) => { new += 1; }
                                Ok(AddOutcome::Updated { .. }) => { updated += 1; }
                                Ok(AddOutcome::Unchanged { .. }) => {}
                                Err(e) => errors.push(format!("{}: {}", p.display(), e)),
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(ScanReport { total, new, updated, errors })
    }
}

pub struct ScanReport {
    pub total: usize,
    pub new: usize,
    pub updated: usize,
    pub errors: Vec<String>,
}
