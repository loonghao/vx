//! ffmpeg provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// ffmpeg provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct FfmpegProvider;

impl Provider for FfmpegProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("ffmpeg")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("FFmpeg - A complete, cross-platform solution to record, convert and stream audio and video")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("ffmpeg", "ffmpeg", ProviderSource::BuiltIn)
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "ffmpeg",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(FfmpegProvider)
}
