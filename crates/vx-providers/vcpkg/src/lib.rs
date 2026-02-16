//! vcpkg provider for vx
//!
//! This crate provides the vcpkg C++ package manager provider for vx.
//!
//! vcpkg-tool is the standalone binary for the vcpkg package manager,
//! downloaded from https://github.com/microsoft/vcpkg-tool/releases.
//!
//! # Features
//!
//! - Download pre-built vcpkg binary for each platform
//! - Install C++ packages (e.g., winpty for node-pty)
//! - Set up environment variables for native builds
//! - Date-based versioning (e.g., 2025-12-16 → 2025.12.16)
//!
//! # Usage
//!
//! ```bash
//! # Install vcpkg
//! vx install vcpkg
//!
//! # Install a C++ package
//! vx vcpkg install winpty
//!
//! # Use in node-pty builds
//! vx npm install node-pty
//! ```
//!
//! # Environment Variables
//!
//! - `VCPKG_ROOT`: Path to vcpkg installation
//! - `VCPKG_DOWNLOADS`: Downloads cache directory (vx managed)
//! - `VCPKG_DEFAULT_BINARY_CACHE`: Binary cache directory (vx managed)
//! - `VCPKG_DEFAULT_TRIPLET`: Default triplet (e.g., x64-windows)
//!
//! # References
//!
//! - [vcpkg Documentation](https://vcpkg.io/)
//! - [vcpkg-tool GitHub](https://github.com/microsoft/vcpkg-tool)

mod provider;
mod runtime;

pub use provider::VcpkgProvider;
pub use runtime::VcpkgRuntime;
pub use runtime::native_packages;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new vcpkg provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(VcpkgProvider::new())
}
