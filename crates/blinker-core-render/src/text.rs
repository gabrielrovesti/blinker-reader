use blinker_core_common::{BlinkerError, Result};
use std::path::Path;
use crate::{DocumentRenderer, RenderSearchMatch, RenderedPage};

pub struct TextRenderer {
    content: String,
    is_markdown: bool,
}

impl TextRenderer {
    /// Convert Markdown to plain text for searching
    fn markdown_to_text(markdown: &str) -> String {
        // Parse markdown and extract text
        use pulldown_cmark::{Parser, Event, Tag};

        let parser = Parser::new(markdown);
        let mut text = String::new();

        for event in parser {
            match event {
                Event::Text(t) => text.push_str(&t),
                Event::Code(c) => text.push_str(&c),
                Event::Start(Tag::Paragraph) => text.push('\n'),
                Event::End(Tag::Paragraph) => text.push_str("\n\n"),
                Event::Start(Tag::Heading { .. }) => text.push('\n'),
                Event::End(Tag::Heading { .. }) => text.push_str("\n\n"),
                Event::HardBreak => text.push('\n'),
                Event::SoftBreak => text.push(' '),
                _ => {}
            }
        }

        text
    }

    /// Get the text content for searching
    fn get_text_content(&self) -> String {
        if self.is_markdown {
            Self::markdown_to_text(&self.content)
        } else {
            self.content.clone()
        }
    }
}

impl DocumentRenderer for TextRenderer {
    fn open(path: &Path) -> Result<Self> {
        tracing::info!("Opening text file: {:?}", path);

        // Read the entire file
        let content = std::fs::read_to_string(path)
            .map_err(|e| BlinkerError::Io(e))?;

        // Check if it's markdown based on extension
        let is_markdown = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase() == "md" || e.to_lowercase() == "markdown")
            .unwrap_or(false);

        tracing::debug!("Text file loaded: {} chars, markdown: {}", content.len(), is_markdown);

        Ok(Self { content, is_markdown })
    }

    fn page_count(&self) -> Result<usize> {
        // For text/markdown, we treat it as a single flowing page
        Ok(1)
    }

    fn render_page(&self, page: usize) -> Result<RenderedPage> {
        if page != 1 {
            return Err(BlinkerError::Rendering(format!("Invalid page index: {}", page)));
        }

        // For MVP, render as a simple white canvas
        // In a real implementation, you'd want to:
        // 1. Layout text with proper word wrapping
        // 2. Render markdown with formatting
        // 3. Support pagination for long documents

        let width = 800u32;
        let height = 1000u32;

        // Create white background RGBA
        let pixels = vec![255u8; (width * height * 4) as usize];

        tracing::warn!("Text rendering is simplified - proper layout TODO");

        Ok(RenderedPage { width, height, pixels })
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<RenderSearchMatch>> {
        if query.is_empty() {
            return Ok(vec![]);
        }

        let mut matches = Vec::new();
        let text = self.get_text_content();
        let text_lower = text.to_lowercase();
        let query_lower = query.to_lowercase();

        // Find all occurrences
        let mut start = 0;
        while let Some(pos) = text_lower[start..].find(&query_lower) {
            let actual_pos = start + pos;

            // Extract context around the match
            let context_start = actual_pos.saturating_sub(50);
            let context_end = (actual_pos + query.len() + 50).min(text.len());
            let context = text[context_start..context_end].trim().to_string();

            matches.push(RenderSearchMatch {
                page: 1, // Single page for text documents
                text: context,
            });

            if matches.len() >= limit {
                break;
            }

            start = actual_pos + 1;
        }

        Ok(matches)
    }
}
