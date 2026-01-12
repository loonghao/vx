//! FFmpeg provider for vx
//!
//! This crate provides the FFmpeg media processing tools provider for vx.
//! FFmpeg is a complete, cross-platform solution to record, convert and stream audio and video.
//!
//! This provider includes:
//! - `ffmpeg` - The main multimedia framework
//! - `ffprobe` - Multimedia stream analyzer
//! - `ffplay` - Simple media player
//!
//! # Download Sources
//!
//! Since FFmpeg doesn't provide official prebuilt binaries, this provider uses
//! trusted third-party sources:
//!
//! - **Windows**: [gyan.dev](https://www.gyan.dev/ffmpeg/builds/) - GPL/LGPL builds
//! - **macOS**: [evermeet.cx](https://evermeet.cx/ffmpeg/) - Static builds
//! - **Linux**: [johnvansickle.com](https://johnvansickle.com/ffmpeg/) - Static builds
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_ffmpeg::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "ffmpeg");
//! ```
//!
//! # Usage
//!
//! ```bash
//! # Install FFmpeg
//! vx install ffmpeg
//!
//! # Convert video
//! vx ffmpeg -i input.mp4 output.mp3
//!
//! # Analyze media file
//! vx ffprobe -v quiet -print_format json input.mp4
//!
//! # Play media file
//! vx ffplay input.mp4
//! ```

pub mod config;
mod provider;
mod runtime;

pub use config::{FfmpegBuild, FfmpegUrlBuilder};
pub use provider::FfmpegProvider;
pub use runtime::{FfmpegRuntime, FfplayRuntime, FfprobeRuntime};

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new FFmpeg provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(FfmpegProvider::new())
}
