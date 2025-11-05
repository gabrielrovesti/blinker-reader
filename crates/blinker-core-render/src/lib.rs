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
use std::path::Path;

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

/// Type-erased renderer for dynamic dispatch at runtime.
pub enum AnyRenderer {
    Pdf(PdfRenderer),
    Epub(epub::EpubRenderer),
    Comic(comic::ComicRenderer),
    Text(text::TextRenderer),
}

impl AnyRenderer {
    /// Open a renderer based on file extension.
    pub fn open_for(path: &Path) -> Result<Self> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| blinker_core_common::BlinkerError::Parsing("Missing file extension".into()))?;
        match DocumentFormat::from_extension(ext) {
            Some(DocumentFormat::Pdf) => Ok(Self::Pdf(PdfRenderer::open(path)?)),
            Some(DocumentFormat::Epub) => Ok(Self::Epub(epub::EpubRenderer::open(path)?)),
            Some(DocumentFormat::Cbz) | Some(DocumentFormat::Cbr) => {
                Ok(Self::Comic(comic::ComicRenderer::open(path)?))
            }
            Some(DocumentFormat::Txt) | Some(DocumentFormat::Markdown) => {
                Ok(Self::Text(text::TextRenderer::open(path)?))
            }
            None => Err(blinker_core_common::BlinkerError::Parsing(
                format!("Unsupported format: {}", ext),
            )),
        }
    }
}
