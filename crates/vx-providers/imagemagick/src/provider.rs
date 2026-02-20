//! imagemagick provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// imagemagick provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct ImageMagickProvider;

impl Provider for ImageMagickProvider {
    fn name(&self) -> &str {
        "imagemagick"
    }

    fn description(&self) -> &str {
        "ImageMagick - image processing tools"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "magick",
            "magick",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(ImageMagickProvider)
}
