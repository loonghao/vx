//! 7-Zip file archiver provider for vx
//!
//! Provides 7-Zip support using the vx-runtime traits.
//! 7-Zip is a free and open-source file archiver with a high compression ratio.
//! It supports many archive formats including 7z, ZIP, TAR, GZ, XZ, BZ2, etc.

mod provider;
mod runtime;

pub use provider::SevenZipProvider;
pub use runtime::SevenZipRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new 7-Zip provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(SevenZipProvider::new())
}
