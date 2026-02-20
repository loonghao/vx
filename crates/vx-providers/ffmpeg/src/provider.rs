//! ffmpeg provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// ffmpeg provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct FfmpegProvider;

impl Provider for FfmpegProvider {
    fn name(&self) -> &str {
        "ffmpeg"
    }

    fn description(&self) -> &str {
        "FFmpeg - A complete, cross-platform solution to record, convert and stream audio and video"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "ffmpeg",
            "ffmpeg",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(FfmpegProvider)
}
