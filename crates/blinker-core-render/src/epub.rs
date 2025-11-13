use blinker_core_common::{BlinkerError, Result};
use std::path::Path;
use crate::{DocumentRenderer, RenderSearchMatch, RenderedPage};

pub struct EpubRenderer {
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
            // Prefer new chapter-based API when available
            // set_current_chapter returns Result
            let _ = doc.set_current_chapter(i);
            if let Some((content, _base)) = doc.get_current_str() {
                let sanitized = Self::sanitize_html(&content);
                chapters.push(sanitized);
            }
        }

        tracing::debug!("EPUB loaded with {} chapters", chapters.len());

        Ok(Self { chapters })
    }

    fn page_count(&self) -> Result<usize> {
        // For EPUB, we treat each chapter/spine item as a "page"
        Ok(self.chapters.len())
    }

    fn render_page(&self, page: usize) -> Result<RenderedPage> {
        let page_index = page.saturating_sub(1);

        if page_index >= self.chapters.len() {
            return Err(BlinkerError::Rendering(format!("Invalid page index: {}", page)));
        }

        let html = &self.chapters[page_index];
        let text = Self::extract_text_from_html(html);

        let width = 800u32;
        let height = 1000u32;

        // Reuse the simple text rasterizer from the text renderer
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

        let mut pixels = vec![255u8; (width * height * 4) as usize];
        if let Some(bytes) = load_system_font_bytes() {
            if let Ok(font) = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()) {
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
                    let gy = y - metrics.height as i32;
                    for row in 0..(metrics.height as i32) {
                        let dy = gy + row;
                        if dy < 0 || dy >= h_i { continue; }
                        for col in 0..(metrics.width as i32) {
                            let dx = gx + col;
                            if dx < 0 || dx >= w_i { continue; }
                            let src_alpha = bitmap[(row as usize) * (metrics.width as usize) + (col as usize)];
                            if src_alpha == 0 { continue; }
                            let idx = ((dy as u32) * width + (dx as u32)) as usize * 4;
                            let val = 255u8.saturating_sub(src_alpha);
                            pixels[idx] = val;
                            pixels[idx + 1] = val;
                            pixels[idx + 2] = val;
                            pixels[idx + 3] = 255;
                        }
                    }
                    x += adv.max(1);
                }
            }
        } else {
            tracing::warn!("No system font found; returning blank canvas for EPUB page");
        }

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
