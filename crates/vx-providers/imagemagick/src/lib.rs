//! ImageMagick provider for vx
//!
//! This crate provides the ImageMagick provider for vx.
//! ImageMagick is a powerful image manipulation software suite that can
//! create, edit, compose, or convert digital images.
//!
//! # Platform Support
//!
//! - **Linux**: Direct download via AppImage binary
//! - **macOS**: System installation recommended (brew install imagemagick)
//! - **Windows**: System installation recommended (choco install imagemagick)
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_imagemagick::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "imagemagick");
//! ```

mod config;
mod provider;
mod runtime;

pub use config::ImageMagickUrlBuilder;
pub use provider::ImageMagickProvider;
pub use runtime::{ConvertRuntime, MagickRuntime};

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new ImageMagick provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(ImageMagickProvider::new())
}
