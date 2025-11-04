use blinker_core_common::{types::Metadata, Result};
use std::path::Path;

pub struct MetadataExtractor;

impl MetadataExtractor {
    pub fn extract(_path: &Path) -> Result<Metadata> {
        // TODO: Extract metadata based on file type
        // - PDF: PDFium metadata
        // - EPUB: Parse OPF manifest
        // - Others: basic file info
        Ok(Metadata::default())
    }
}
