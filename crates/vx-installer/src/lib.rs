//! # vx-installer
//!
//! Installation utilities and helpers for the vx universal tool manager.
//!
//! This crate provides a unified interface for downloading, extracting, and installing
//! development tools across different platforms and archive formats.
//!
//! ## Features
//!
//! - **Unified Installation API**: Single interface for all installation operations
//! - **Multiple Archive Formats**: Support for ZIP, TAR.GZ, TAR.XZ, and binary files
//! - **Progress Tracking**: Built-in progress bars for downloads and extractions
//! - **Platform Agnostic**: Works across Windows, macOS, and Linux
//! - **Async Support**: Fully async API for non-blocking operations
//! - **CDN Acceleration**: Optional CDN optimization via turbo-cdn
//!
//! ## Example
//!
//! ```rust,no_run
//! use vx_installer::{Installer, InstallConfig, InstallMethod, ArchiveFormat};
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let installer = Installer::new().await?;
//!
//!     let config = InstallConfig::builder()
//!         .tool_name("node")
//!         .version("18.17.0")
//!         .download_url("https://nodejs.org/dist/v18.17.0/node-v18.17.0-linux-x64.tar.gz")
//!         .install_method(InstallMethod::Archive {
//!             format: ArchiveFormat::TarGz
//!         })
//!         .install_dir(PathBuf::from("/opt/vx/tools/node/18.17.0"))
//!         .build();
//!
//!     let executable_path = installer.install(&config).await?;
//!     println!("Installed to: {}", executable_path.display());
//!
//!     Ok(())
//! }
//! ```

pub mod cdn;
pub mod downloader;
pub mod error;
pub mod formats;
pub mod installer;
pub mod progress;

// Re-export main types for convenience
pub use cdn::{CdnConfig, CdnOptimizer};
pub use downloader::Downloader;
pub use error::{Error, Result};
pub use installer::{ArchiveFormat, InstallConfig, InstallConfigBuilder, InstallMethod, Installer};
pub use progress::{ProgressReporter, ProgressStyle};

// Re-export format handlers
pub use formats::{ArchiveExtractor, FormatHandler};

/// Version information for the vx-installer crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default user agent for HTTP requests
pub const USER_AGENT: &str = concat!("vx-installer/", env!("CARGO_PKG_VERSION"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert!(USER_AGENT.contains("vx-installer"));
    }
}
