use blinker_core_common::{BlinkerError, Result};
use pdfium_render::prelude::*;
use std::path::Path;
use crate::{DocumentRenderer, RenderSearchMatch, RenderedPage};

pub struct PdfRenderer {
    document: PdfDocument<'static>,
    pdfium: Pdfium,
}

impl PdfRenderer {
    /// Get the global PDFium instance
    fn get_pdfium() -> Result<Pdfium> {
        // Try to bind to system library first, then bundled
        Pdfium::new(
            Pdfium::bind_to_system_library()
                .or_else(|_| Pdfium::bind_to_statically_linked_library())
                .map_err(|e| BlinkerError::Rendering(format!("Failed to initialize PDFium: {:?}", e)))?
        )
    }
}

impl DocumentRenderer for PdfRenderer {
    fn open(path: &Path) -> Result<Self> {
        tracing::info!("Opening PDF: {:?}", path);

        let pdfium = Self::get_pdfium()?;

        // Load the PDF document without password
        let document = pdfium
            .load_pdf_from_file(path, None)
            .map_err(|e| BlinkerError::Rendering(format!("Failed to load PDF: {:?}", e)))?;

        // Verify JavaScript is disabled (PDFium disables it by default)
        tracing::debug!("PDF loaded with {} pages", document.pages().len());

        Ok(Self { document, pdfium })
    }

    fn page_count(&self) -> Result<usize> {
        Ok(self.document.pages().len())
    }

    fn render_page(&self, page: usize) -> Result<RenderedPage> {
        // Pages in pdfium are 0-indexed, but we use 1-indexed externally
        let page_index = page.saturating_sub(1);

        let pdf_page = self.document
            .pages()
            .get(page_index)
            .map_err(|e| BlinkerError::Rendering(format!("Invalid page index {}: {:?}", page, e)))?;

        // Calculate render dimensions at 150 DPI (good balance of quality/performance)
        let width_points = pdf_page.width().value;
        let height_points = pdf_page.height().value;
        let dpi = 150.0;
        let scale = dpi / 72.0; // PDF points are 1/72 inch

        let target_width = (width_points * scale) as i32;
        let target_height = (height_points * scale) as i32;

        tracing::debug!("Rendering page {} at {}x{}", page, target_width, target_height);

        // Configure rendering
        let config = PdfRenderConfig::new()
            .set_target_width(target_width)
            .set_target_height(target_height)
            .rotate_if_landscape(PdfBitmapRotation::None, false);

        // Render to bitmap
        let bitmap = pdf_page
            .render_with_config(&config)
            .map_err(|e| BlinkerError::Rendering(format!("Failed to render page: {:?}", e)))?;

        // Convert to RGBA8
        let width = bitmap.width() as u32;
        let height = bitmap.height() as u32;

        // Get the raw buffer - pdfium-render provides BGRA format
        let buffer = bitmap.as_bytes();

        // Convert BGRA to RGBA
        let mut pixels = Vec::with_capacity(buffer.len());
        for chunk in buffer.chunks_exact(4) {
            pixels.push(chunk[2]); // R (from B)
            pixels.push(chunk[1]); // G
            pixels.push(chunk[0]); // B (from R)
            pixels.push(chunk[3]); // A
        }

        Ok(RenderedPage { width, height, pixels })
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<RenderSearchMatch>> {
        if query.is_empty() {
            return Ok(vec![]);
        }

        let mut matches = Vec::new();
        let query_lower = query.to_lowercase();

        // Search through all pages
        for (page_idx, page) in self.document.pages().iter().enumerate() {
            if matches.len() >= limit {
                break;
            }

            // Extract text from the page
            let text_result = page.text();
            if let Ok(page_text) = text_result {
                let all_text = page_text.all();
                if let Ok(text) = all_text {
                    let text_lower = text.to_lowercase();

                    // Find all occurrences in this page
                    let mut start = 0;
                    while let Some(pos) = text_lower[start..].find(&query_lower) {
                        let actual_pos = start + pos;

                        // Extract context around the match (50 chars before and after)
                        let context_start = actual_pos.saturating_sub(50);
                        let context_end = (actual_pos + query.len() + 50).min(text.len());
                        let context = &text[context_start..context_end];

                        matches.push(RenderSearchMatch {
                            page: page_idx + 1, // Convert to 1-indexed
                            text: context.to_string(),
                        });

                        if matches.len() >= limit {
                            break;
                        }

                        start = actual_pos + 1;
                    }
                }
            }
        }

        Ok(matches)
    }
}
