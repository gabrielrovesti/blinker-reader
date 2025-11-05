use blinker_core_common::Result;
use std::path::Path;
use crate::{DocumentRenderer, RenderedPage};

pub struct TextRenderer;

impl TextRenderer {
    pub fn render_txt(_content: &str) -> Result<String> {
        // TODO: Render plain text
        Ok(String::new())
    }

    pub fn render_markdown(_content: &str) -> Result<String> {
        // TODO: Render Markdown to HTML
        Ok(String::new())
    }
}

impl DocumentRenderer for TextRenderer {
    fn open(_path: &Path) -> Result<Self> {
        // TODO: Load file and prepare lines/HTML
        Ok(Self)
    }

    fn page_count(&self) -> Result<usize> {
        // Flow content; 1 logical page for MVP
        Ok(1)
    }

    fn render_page(&self, _page: usize) -> Result<RenderedPage> {
        // TODO: Rasterize text/markdown into a bitmap target
        Ok(RenderedPage { width: 0, height: 0, pixels: vec![] })
    }
}
