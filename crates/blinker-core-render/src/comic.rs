use blinker_core_common::{BlinkerError, Result};
use std::path::Path;
use crate::{DocumentRenderer, RenderedPage};
use std::io::Read;

pub struct ComicRenderer {
    images: Vec<(String, Vec<u8>)>, // (filename, image data)
}

impl ComicRenderer {
    /// Validate that the archive path doesn't contain directory traversal
    fn validate_archive_path(path: &str) -> Result<()> {
        if path.contains("..") || path.starts_with('/') || path.starts_with('\\') {
            return Err(BlinkerError::Security(
                format!("Path traversal attempt detected: {}", path)
            ));
        }
        Ok(())
    }

    /// Check if a filename is a supported image format
    fn is_image_file(filename: &str) -> bool {
        let lower = filename.to_lowercase();
        lower.ends_with(".jpg") ||
        lower.ends_with(".jpeg") ||
        lower.ends_with(".png") ||
        lower.ends_with(".gif") ||
        lower.ends_with(".bmp") ||
        lower.ends_with(".webp")
    }

    /// Sort image filenames naturally (e.g., page1, page2, ..., page10)
    fn sort_filenames(filenames: &mut [(String, Vec<u8>)]) {
        filenames.sort_by(|a, b| {
            // Natural sort - extract numbers from filenames
            natord::compare(&a.0, &b.0)
        });
    }
}

impl DocumentRenderer for ComicRenderer {
    fn open(path: &Path) -> Result<Self> {
        tracing::info!("Opening Comic archive: {:?}", path);

        let file = std::fs::File::open(path)
            .map_err(|e| BlinkerError::Io(e))?;

        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| BlinkerError::Parsing(format!("Failed to open CBZ archive: {}", e)))?;

        let mut images = Vec::new();

        // Extract all images from the archive
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| BlinkerError::Parsing(format!("Failed to read archive entry: {}", e)))?;

            let filename = file.name().to_string();

            // Security check: validate path
            Self::validate_archive_path(&filename)?;

            // Only process image files
            if Self::is_image_file(&filename) && !file.is_dir() {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)
                    .map_err(|e| BlinkerError::Io(e))?;

                images.push((filename, buffer));
            }
        }

        // Sort images by filename
        Self::sort_filenames(&mut images);

        tracing::debug!("Comic archive loaded with {} images", images.len());

        Ok(Self { images })
    }

    fn page_count(&self) -> Result<usize> {
        Ok(self.images.len())
    }

    fn render_page(&self, page: usize) -> Result<RenderedPage> {
        let page_index = page.saturating_sub(1);

        if page_index >= self.images.len() {
            return Err(BlinkerError::Rendering(format!("Invalid page index: {}", page)));
        }

        let (filename, image_data) = &self.images[page_index];

        tracing::debug!("Rendering comic page {}: {}", page, filename);

        // Decode the image
        let img = image::load_from_memory(image_data)
            .map_err(|e| BlinkerError::Rendering(format!("Failed to decode image: {}", e)))?;

        // Convert to RGBA8
        let rgba = img.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();
        let pixels = rgba.into_raw();

        Ok(RenderedPage { width, height, pixels })
    }

    fn search(&self, _query: &str, _limit: usize) -> Result<Vec<crate::RenderSearchMatch>> {
        // Comics are images, no text search available
        Ok(vec![])
    }
}
