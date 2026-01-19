//! ImageMagick provider implementation
//!
//! Provides the ImageMagick runtimes (magick, convert).

use crate::runtime::{ConvertRuntime, MagickRuntime};
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// ImageMagick provider
#[derive(Debug, Default)]
pub struct ImageMagickProvider;

impl ImageMagickProvider {
    /// Create a new ImageMagick provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for ImageMagickProvider {
    fn name(&self) -> &str {
        "imagemagick"
    }

    fn description(&self) -> &str {
        "ImageMagick - A powerful image manipulation and conversion tool"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![
            Arc::new(MagickRuntime::new()),
            Arc::new(ConvertRuntime::new()),
        ]
    }

    fn supports(&self, name: &str) -> bool {
        matches!(name, "magick" | "imagemagick" | "convert")
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        match name {
            "magick" | "imagemagick" => Some(Arc::new(MagickRuntime::new())),
            "convert" => Some(Arc::new(ConvertRuntime::new())),
            _ => None,
        }
    }
}
