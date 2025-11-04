use blinker_core_common::Result;

pub struct PdfRenderer {
    // TODO: PDFium instance
}

impl PdfRenderer {
    pub fn new() -> Result<Self> {
        // TODO: Initialize PDFium
        // - Disable JavaScript
        // - Disable risky actions (launch, URI, etc.)
        Ok(Self {})
    }

    pub fn render_page(&self, _page_num: usize) -> Result<Vec<u8>> {
        // TODO: Render page to bitmap
        Ok(vec![])
    }
}
