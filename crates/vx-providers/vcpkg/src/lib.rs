//! vcpkg provider for vx
//!
//! This crate provides the vcpkg C++ package manager provider for vx.
//!
//! vcpkg is a C++ library manager that simplifies the installation of
//! C++ libraries and their dependencies. It is particularly useful for
//! native Node.js modules that require C++ dependencies.
//!
//! # Features
//!
//! - Install vcpkg via git clone
//! - Bootstrap vcpkg automatically
//! - Install C++ packages (e.g., winpty for node-pty)
//! - Set up environment variables for native builds
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
//! - `CMAKE_TOOLCHAIN_FILE`: Path to vcpkg.cmake
//! - `VCPKG_DEFAULT_TRIPLET`: Default triplet (e.g., x64-windows)
//!
//! # References
//!
//! - [vcpkg Documentation](https://vcpkg.io/)
//! - [vcpkg GitHub](https://github.com/microsoft/vcpkg)

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
