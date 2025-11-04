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
