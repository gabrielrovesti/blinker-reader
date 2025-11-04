use blinker_core_common::Result;

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
