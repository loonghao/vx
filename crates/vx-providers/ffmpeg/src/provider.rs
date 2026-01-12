//! FFmpeg provider implementation
//!
//! Provides the FFmpeg, FFprobe, and FFplay runtimes.

use crate::runtime::{FfmpegRuntime, FfplayRuntime, FfprobeRuntime};
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// FFmpeg provider
///
/// Provides media processing tools:
/// - `ffmpeg` - The main multimedia framework
/// - `ffprobe` - Multimedia stream analyzer
/// - `ffplay` - Simple media player
#[derive(Debug, Default)]
pub struct FfmpegProvider;

impl FfmpegProvider {
    /// Create a new FFmpeg provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for FfmpegProvider {
    fn name(&self) -> &str {
        "ffmpeg"
    }

    fn description(&self) -> &str {
        "Complete solution for recording, converting and streaming audio and video"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![
            Arc::new(FfmpegRuntime::new()),
            Arc::new(FfprobeRuntime::new()),
            Arc::new(FfplayRuntime::new()),
        ]
    }

    fn supports(&self, name: &str) -> bool {
        matches!(name, "ffmpeg" | "ffprobe" | "ffplay")
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        match name {
            "ffmpeg" => Some(Arc::new(FfmpegRuntime::new())),
            "ffprobe" => Some(Arc::new(FfprobeRuntime::new())),
            "ffplay" => Some(Arc::new(FfplayRuntime::new())),
            _ => None,
        }
    }
}
