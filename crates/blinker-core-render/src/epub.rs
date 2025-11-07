use blinker_core_common::{BlinkerError, Result};
use std::path::Path;
use crate::{DocumentRenderer, RenderSearchMatch, RenderedPage};

pub struct EpubRenderer {
    doc: epub::doc::EpubDoc<std::io::BufReader<std::fs::File>>,
    chapters: Vec<String>,
}

impl EpubRenderer {
    /// Sanitize HTML content to remove scripts and dangerous elements
    fn sanitize_html(html: &str) -> String {
        ammonia::Builder::default()
            .link_rel(None) // Remove all rel attributes
            .url_relative(ammonia::UrlRelative::Deny) // Block relative URLs that could leak info
            .rm_tags(&["script", "iframe", "object", "embed", "form"])
            .clean(html)
            .to_string()
    }

    /// Extract all text from HTML (simple implementation)
    fn extract_text_from_html(html: &str) -> String {
        // Very basic HTML text extraction
        // In production, you'd want to use a proper HTML parser
        let sanitized = Self::sanitize_html(html);

        // Remove HTML tags
        let mut text = String::new();
        let mut in_tag = false;

        for c in sanitized.chars() {
            match c {
                '<' => in_tag = true,
                '>' => {
                    in_tag = false;
                    text.push(' ');
                }
                _ if !in_tag => text.push(c),
                _ => {}
            }
        }

        text
    }
}

impl DocumentRenderer for EpubRenderer {
    fn open(path: &Path) -> Result<Self> {
        tracing::info!("Opening EPUB: {:?}", path);

        let mut doc = epub::doc::EpubDoc::new(path)
            .map_err(|e| BlinkerError::Parsing(format!("Failed to load EPUB: {}", e)))?;

        // Extract all chapter contents for searching and rendering
        let mut chapters = Vec::new();

        // Get spine (reading order)
        let spine_len = doc.spine.len();

        for i in 0..spine_len {
            doc.set_current_page(i);

            if let Some(content) = doc.get_current_str() {
                // Sanitize the HTML content
                let sanitized = Self::sanitize_html(&content);
                chapters.push(sanitized);
            }
        }

        tracing::debug!("EPUB loaded with {} chapters", chapters.len());

        Ok(Self { doc, chapters })
    }

    fn page_count(&self) -> Result<usize> {
        // For EPUB, we treat each chapter/spine item as a "page"
        Ok(self.chapters.len())
    }

    fn render_page(&self, page: usize) -> Result<RenderedPage> {
        // For MVP, we'll render EPUB as simple text
        // A full implementation would use a web rendering engine

        let page_index = page.saturating_sub(1);

        if page_index >= self.chapters.len() {
            return Err(BlinkerError::Rendering(format!("Invalid page index: {}", page)));
        }

        let html = &self.chapters[page_index];
        let text = Self::extract_text_from_html(html);

        // Create a simple text rendering
        // For MVP: white background with black text
        // TODO: Proper HTML/CSS rendering using a web engine

        let width = 800u32;
        let height = 1000u32;

        // Create white background RGBA
        let mut pixels = vec![255u8; (width * height * 4) as usize];

        // For now, just return the white canvas
        // In a real implementation, you'd render the text properly
        // This is a placeholder that at least shows the document opened

        tracing::warn!("EPUB rendering is simplified - proper HTML rendering TODO");

        Ok(RenderedPage { width, height, pixels })
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<RenderSearchMatch>> {
        if query.is_empty() {
            return Ok(vec![]);
        }

        let mut matches = Vec::new();
        let query_lower = query.to_lowercase();

        for (page_idx, chapter) in self.chapters.iter().enumerate() {
            if matches.len() >= limit {
                break;
            }

            let text = Self::extract_text_from_html(chapter);
            let text_lower = text.to_lowercase();

            // Find all occurrences in this chapter
            let mut start = 0;
            while let Some(pos) = text_lower[start..].find(&query_lower) {
                let actual_pos = start + pos;

                // Extract context around the match
                let context_start = actual_pos.saturating_sub(50);
                let context_end = (actual_pos + query.len() + 50).min(text.len());
                let context = text[context_start..context_end].trim().to_string();

                matches.push(RenderSearchMatch {
                    page: page_idx + 1,
                    text: context,
                });

                if matches.len() >= limit {
                    break;
                }

                start = actual_pos + 1;
            }
        }

        Ok(matches)
    }
}
