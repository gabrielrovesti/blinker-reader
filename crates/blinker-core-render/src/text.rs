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
        use pulldown_cmark::{Parser, Event, Tag, TagEnd};

        let parser = Parser::new(markdown);
        let mut text = String::new();

        for event in parser {
            match event {
                Event::Text(t) => text.push_str(&t),
                Event::Code(c) => text.push_str(&c),
                Event::Start(Tag::Paragraph) => text.push('\n'),
                Event::End(TagEnd::Paragraph) => text.push_str("\n\n"),
                Event::Start(Tag::Heading { .. }) => text.push('\n'),
                Event::End(TagEnd::Heading { .. }) => text.push_str("\n\n"),
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

    #[allow(dead_code)]
    fn load_system_font_bytes() -> Option<Vec<u8>> {
        #[cfg(target_os = "windows")]
        {
            let candidates = [
                r"C:\\Windows\\Fonts\\consola.ttf",
                r"C:\\Windows\\Fonts\\arial.ttf",
                r"C:\\Windows\\Fonts\\segoeui.ttf",
            ];
            for p in candidates {
                if let Ok(b) = std::fs::read(p) { return Some(b); }
            }
        }
        #[cfg(target_os = "linux")]
        {
            let candidates = [
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
            ];
            for p in candidates {
                if let Ok(b) = std::fs::read(p) { return Some(b); }
            }
        }
        #[cfg(target_os = "macos")]
        {
            let candidates = [
                "/System/Library/Fonts/SFNS.ttf",
                "/System/Library/Fonts/Supplemental/Arial.ttf",
            ];
            for p in candidates {
                if let Ok(b) = std::fs::read(p) { return Some(b); }
            }
        }
        None
    }

    fn render_text_bitmap(width: u32, height: u32, text: &str) -> Vec<u8> {
        let mut pixels = vec![255u8; (width * height * 4) as usize];
        let Some(bytes) = Self::load_system_font_bytes() else {
            tracing::warn!("No system font found; returning blank canvas");
            return pixels;
        };
        let font = match fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()) {
            Ok(f) => f,
            Err(_) => return pixels,
        };
        let font_size = 18.0;
        let line_h = (font_size * 1.4) as i32;
        let margin = 16i32;
        let mut x = margin;
        let mut y = margin + line_h;

        let w_i = width as i32;
        let h_i = height as i32;

        for ch in text.chars() {
            if ch == '\n' {
                x = margin;
                y += line_h;
                if y >= h_i - margin { break; }
                continue;
            }

            let (metrics, bitmap) = font.rasterize(ch, font_size);
            let adv = metrics.advance_width as i32;
            if x + adv >= w_i - margin {
                x = margin;
                y += line_h;
                if y >= h_i - margin { break; }
            }

            let gx = x + metrics.xmin;
            let gy = y - metrics.height as i32; // approximate baseline placement

            // Blit glyph bitmap (grayscale) onto RGBA buffer with black text
            for row in 0..(metrics.height as i32) {
                let dy = gy + row;
                if dy < 0 || dy >= h_i { continue; }
                for col in 0..(metrics.width as i32) {
                    let dx = gx + col;
                    if dx < 0 || dx >= w_i { continue; }
                    let src_alpha = bitmap[(row as usize) * (metrics.width as usize) + (col as usize)];
                    if src_alpha == 0 { continue; }
                    let idx = ((dy as u32) * width + (dx as u32)) as usize * 4;
                    // simple alpha-over on white: new = 255 - alpha
                    let val = 255u8.saturating_sub(src_alpha);
                    pixels[idx] = val;
                    pixels[idx + 1] = val;
                    pixels[idx + 2] = val;
                    pixels[idx + 3] = 255;
                }
            }

            x += adv.max(1);
        }

        pixels
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

        let width = 800u32;
        let height = 1000u32;
        let text = self.get_text_content();
        let pixels = Self::render_text_bitmap(width, height, &text);
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
