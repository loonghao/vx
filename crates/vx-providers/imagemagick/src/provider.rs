//! imagemagick provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// imagemagick provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct ImageMagickProvider;

impl Provider for ImageMagickProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("magick")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("ImageMagick - image processing tools")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("magick", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(ImageMagickProvider)
}
