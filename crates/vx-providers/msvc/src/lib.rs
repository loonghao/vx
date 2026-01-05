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

#[cfg(target_os = "windows")]
mod config;
#[cfg(target_os = "windows")]
mod installer;
#[cfg(target_os = "windows")]
mod provider;
#[cfg(target_os = "windows")]
mod runtime;

#[cfg(target_os = "windows")]
pub use config::{MsvcInstallConfig, PlatformHelper};
#[cfg(target_os = "windows")]
pub use installer::{MsvcInstallInfo, MsvcInstaller};
#[cfg(target_os = "windows")]
pub use provider::MsvcProvider;
#[cfg(target_os = "windows")]
pub use runtime::MsvcRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new MSVC provider instance
///
/// This is the main entry point for the provider, used by the registry.
#[cfg(target_os = "windows")]
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(MsvcProvider::new())
}

// ============================
// Non-Windows fallback (stub)
// ============================
#[cfg(not(target_os = "windows"))]
mod unsupported {
    use super::*;
    use anyhow::anyhow;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use vx_runtime::{
        Ecosystem, InstallResult, Platform, Runtime, RuntimeContext, VerificationResult,
    };

    #[derive(Debug, Default)]
    pub struct UnsupportedRuntime;

    #[async_trait]
    impl Runtime for UnsupportedRuntime {
        fn name(&self) -> &str {
            "msvc"
        }

        fn description(&self) -> &str {
            "MSVC Build Tools (Windows-only; disabled on this platform)"
        }

        fn aliases(&self) -> &[&str] {
            &["cl", "nmake"]
        }

        fn executable_name(&self) -> &str {
            "cl"
        }

        fn ecosystem(&self) -> Ecosystem {
            Ecosystem::System
        }

        fn metadata(&self) -> HashMap<String, String> {
            HashMap::new()
        }

        fn supported_platforms(&self) -> Vec<Platform> {
            vec![]
        }

        fn executable_relative_path(&self, _version: &str, _platform: &Platform) -> String {
            String::new()
        }

        async fn fetch_versions(
            &self,
            _ctx: &RuntimeContext,
        ) -> anyhow::Result<Vec<vx_runtime::VersionInfo>> {
            Ok(vec![])
        }

        async fn download_url(
            &self,
            _version: &str,
            _platform: &Platform,
        ) -> anyhow::Result<Option<String>> {
            Ok(None)
        }

        async fn install(
            &self,
            _version: &str,
            _ctx: &RuntimeContext,
        ) -> anyhow::Result<InstallResult> {
            Err(anyhow!("MSVC Build Tools is only available on Windows"))
        }

        fn verify_installation(
            &self,
            _version: &str,
            _install_path: &std::path::Path,
            _platform: &Platform,
        ) -> VerificationResult {
            VerificationResult::failure(
                vec!["MSVC Build Tools is only available on Windows".to_string()],
                vec!["Use a Windows system to install MSVC Build Tools".to_string()],
            )
        }
    }

    #[derive(Debug, Default)]
    pub struct UnsupportedProvider;

    impl Provider for UnsupportedProvider {
        fn name(&self) -> &str {
            "msvc"
        }

        fn description(&self) -> &str {
            "MSVC Build Tools (Windows-only; disabled on this platform)"
        }

        fn runtimes(&self) -> Vec<Arc<dyn vx_runtime::Runtime>> {
            vec![Arc::new(UnsupportedRuntime)]
        }

        fn supports(&self, name: &str) -> bool {
            matches!(name, "msvc" | "cl" | "msvc-tools" | "vs-build-tools")
        }

        fn get_runtime(&self, name: &str) -> Option<Arc<dyn vx_runtime::Runtime>> {
            if self.supports(name) {
                Some(Arc::new(UnsupportedRuntime))
            } else {
                None
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub use unsupported::{UnsupportedProvider as MsvcProvider, UnsupportedRuntime as MsvcRuntime};

#[cfg(not(target_os = "windows"))]
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(MsvcProvider)
}
