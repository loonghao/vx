//! ffmpeg provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// ffmpeg provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct FfmpegProvider;

impl Provider for FfmpegProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("ffmpeg")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or(
            "A complete, cross-platform solution to record, convert and stream audio and video",
        )
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("ffmpeg", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(FfmpegProvider)
}
