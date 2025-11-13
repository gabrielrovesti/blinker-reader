/// Document rendering: PDF via PDFium, EPUB via HTML flow, images for CBZ/CBR.
///
/// This crate handles:
/// - PDF rendering with JavaScript disabled
/// - EPUB layout and rendering
/// - Comic book archive unpacking and rendering
/// - Text and Markdown rendering

pub mod pdf;
pub mod epub;
pub mod comic;
pub mod text;

pub use pdf::PdfRenderer;
pub use epub::EpubRenderer;

use blinker_core_common::{types::DocumentFormat, Result};
use std::path::{Path, PathBuf};

/// A rendered page bitmap with dimensions.
pub struct RenderedPage {
    pub width: u32,
    pub height: u32,
    /// RGBA8 pixel buffer (row-major).
    pub pixels: Vec<u8>,
}

/// Result item for in-document search.
pub struct RenderSearchMatch {
    pub page: usize,
    pub text: String,
}

/// Common interface for document renderers.
pub trait DocumentRenderer {
    /// Open a renderer for the given file path.
    fn open(path: &Path) -> Result<Self>
    where
        Self: Sized;

    /// Return total pages for paged formats; 1 for flow content.
    fn page_count(&self) -> Result<usize>;

    /// Render a page to a bitmap (RGBA8). Page starts at 1.
    fn render_page(&self, page: usize) -> Result<RenderedPage>;

    /// Naive in-document search; implementations may return empty.
    fn search(&self, _query: &str, _limit: usize) -> Result<Vec<RenderSearchMatch>> {
        Ok(vec![])
    }
}

/// Lightweight handle for rendering that avoids storing non-Send backends.
pub struct AnyRenderer {
    path: PathBuf,
    kind: DocumentFormat,
}

impl AnyRenderer {
    /// Create a handle based on file extension.
    pub fn open_for(path: &Path) -> Result<Self> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| blinker_core_common::BlinkerError::Parsing("Missing file extension".into()))?;
        if let Some(kind) = DocumentFormat::from_extension(ext) {
            Ok(Self { path: path.to_path_buf(), kind })
        } else {
            Err(blinker_core_common::BlinkerError::Parsing(format!("Unsupported format: {}", ext)))
        }
    }

    /// Get page count by opening the appropriate backend on-demand.
    pub fn page_count(&self) -> Result<usize> {
        match self.kind {
            DocumentFormat::Pdf => PdfRenderer::open(&self.path)?.page_count(),
            DocumentFormat::Epub => epub::EpubRenderer::open(&self.path)?.page_count(),
            DocumentFormat::Cbz => comic::ComicRenderer::open(&self.path)?.page_count(),
            DocumentFormat::Cbr => Err(blinker_core_common::BlinkerError::Parsing(
                "CBR (RAR) archives are not supported yet".into(),
            )),
            DocumentFormat::Txt | DocumentFormat::Markdown => text::TextRenderer::open(&self.path)?.page_count(),
        }
    }

    /// Render a specific page by opening the backend on-demand.
    pub fn render_page(&self, page: usize) -> Result<RenderedPage> {
        match self.kind {
            DocumentFormat::Pdf => PdfRenderer::open(&self.path)?.render_page(page),
            DocumentFormat::Epub => epub::EpubRenderer::open(&self.path)?.render_page(page),
            DocumentFormat::Cbz => comic::ComicRenderer::open(&self.path)?.render_page(page),
            DocumentFormat::Cbr => Err(blinker_core_common::BlinkerError::Parsing(
                "CBR (RAR) archives are not supported yet".into(),
            )),
            DocumentFormat::Txt | DocumentFormat::Markdown => text::TextRenderer::open(&self.path)?.render_page(page),
        }
    }

    /// Search within the document by opening the backend on-demand.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<RenderSearchMatch>> {
        match self.kind {
            DocumentFormat::Pdf => PdfRenderer::open(&self.path)?.search(query, limit),
            DocumentFormat::Epub => epub::EpubRenderer::open(&self.path)?.search(query, limit),
            DocumentFormat::Cbz => comic::ComicRenderer::open(&self.path)?.search(query, limit),
            DocumentFormat::Cbr => Ok(vec![]),
            DocumentFormat::Txt | DocumentFormat::Markdown => text::TextRenderer::open(&self.path)?.search(query, limit),
        }
    }
}
