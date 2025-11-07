use blinker_core_common::{types::Metadata, types::DocumentFormat, Result, BlinkerError};
use std::path::Path;

pub struct MetadataExtractor;

impl MetadataExtractor {
    /// Extract metadata from a file based on its type
    pub fn extract(path: &Path) -> Result<Metadata> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| BlinkerError::Parsing("Missing file extension".into()))?;

        match DocumentFormat::from_extension(ext) {
            Some(DocumentFormat::Pdf) => Self::extract_pdf(path),
            Some(DocumentFormat::Epub) => Self::extract_epub(path),
            _ => Self::extract_basic(path),
        }
    }

    /// Extract basic metadata from filename
    fn extract_basic(path: &Path) -> Result<Metadata> {
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();

        Ok(Metadata {
            title,
            ..Default::default()
        })
    }

    /// Extract metadata from PDF using PDFium
    #[cfg(feature = "pdf-metadata")]
    fn extract_pdf(path: &Path) -> Result<Metadata> {
        use pdfium_render::prelude::*;

        tracing::debug!("Extracting PDF metadata from {:?}", path);

        // Try to open PDF
        let pdfium = Pdfium::new(
            Pdfium::bind_to_system_library()
                .or_else(|_| Pdfium::bind_to_statically_linked_library())
                .map_err(|e| BlinkerError::Rendering(format!("Failed to initialize PDFium: {:?}", e)))?
        );

        let document = pdfium
            .load_pdf_from_file(path, None)
            .map_err(|e| BlinkerError::Rendering(format!("Failed to load PDF: {:?}", e)))?;

        let metadata = document.metadata();

        let title = metadata.title()
            .unwrap_or_else(|| path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string());

        let author = metadata.author();
        let subject = metadata.subject();
        let page_count = Some(document.pages().len());

        tracing::debug!("Extracted PDF metadata: title={}, author={:?}, pages={:?}",
                       title, author, page_count);

        Ok(Metadata {
            title,
            author,
            subject,
            page_count,
            ..Default::default()
        })
    }

    #[cfg(not(feature = "pdf-metadata"))]
    fn extract_pdf(path: &Path) -> Result<Metadata> {
        Self::extract_basic(path)
    }

    /// Extract metadata from EPUB
    #[cfg(feature = "epub-metadata")]
    fn extract_epub(path: &Path) -> Result<Metadata> {
        tracing::debug!("Extracting EPUB metadata from {:?}", path);

        let doc = epub::doc::EpubDoc::new(path)
            .map_err(|e| BlinkerError::Parsing(format!("Failed to load EPUB: {}", e)))?;

        let title = doc.mdata("title")
            .unwrap_or_else(|| path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string());

        let author = doc.mdata("creator");
        let subject = doc.mdata("subject");
        let publisher = doc.mdata("publisher");
        let language = doc.mdata("language");

        tracing::debug!("Extracted EPUB metadata: title={}, author={:?}", title, author);

        Ok(Metadata {
            title,
            author,
            subject,
            publisher,
            language,
            ..Default::default()
        })
    }

    #[cfg(not(feature = "epub-metadata"))]
    fn extract_epub(path: &Path) -> Result<Metadata> {
        Self::extract_basic(path)
    }
}
