//! imagemagick provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// imagemagick provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct ImageMagickProvider;

impl Provider for ImageMagickProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("imagemagick")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("ImageMagick - image processing tools")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("magick", "magick", ProviderSource::BuiltIn)
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "imagemagick",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(ImageMagickProvider)
}
