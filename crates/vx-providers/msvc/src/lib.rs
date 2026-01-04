//! MSVC Build Tools provider for vx
//!
//! This crate provides the MSVC Build Tools provider for vx.
//! MSVC Build Tools includes the Microsoft Visual C++ compiler (cl.exe),
//! linker, and related tools for building native Windows applications.
//!
//! Unlike other providers that download pre-built binaries, this provider
//! downloads directly from Microsoft's official Visual Studio servers using
//! the VS manifest system.
//!
//! # Features
//!
//! - Downloads directly from Microsoft's official servers
//! - Supports multiple MSVC toolset versions (14.29 - 14.40+)
//! - Includes MSVC compiler, linker, and Windows SDK
//! - Portable installation (no admin required)
//! - Minimal footprint (only essential build tools)
//!
//! # Architecture Support
//!
//! - x64 (AMD64)
//! - x86 (32-bit)
//! - ARM64
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_msvc::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "msvc");
//! ```
//!
//! # Usage
//!
//! ```bash
//! # Install latest MSVC Build Tools
//! vx install msvc latest
//!
//! # Install specific version
//! vx install msvc 14.40.33807
//!
//! # Use the compiler
//! vx msvc cl /help
//! vx cl main.cpp /Fe:main.exe
//! ```
//!
//! # References
//!
//! - [Visual C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
//! - [PortableBuildTools](https://github.com/Data-Oriented-House/PortableBuildTools)
//! - [VS Manifest System](https://aka.ms/vs/17/release/channel)

mod config;
mod installer;
mod provider;
mod runtime;

pub use config::{MsvcInstallConfig, PlatformHelper};
pub use installer::{MsvcInstallInfo, MsvcInstaller};
pub use provider::MsvcProvider;
pub use runtime::MsvcRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new MSVC provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(MsvcProvider::new())
}
