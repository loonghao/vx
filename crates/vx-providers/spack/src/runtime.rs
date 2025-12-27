//! Spack runtime implementation
//!
//! Spack is a flexible package manager designed for supercomputers, Linux, and macOS.
//! It makes installing scientific software easy by automatically handling dependencies,
//! multiple versions, configurations, platforms, and compilers.
//!
//! **Note**: Spack does not natively support Windows. On Windows, use WSL (Windows
//! Subsystem for Linux) to run Spack.
//!
//! Homepage: https://spack.io
//! Repository: https://github.com/spack/spack

use crate::config::SpackUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VerificationResult,
    VersionInfo,
};

/// Spack runtime implementation
#[derive(Debug, Clone, Default)]
pub struct SpackRuntime;

impl SpackRuntime {
    /// Create a new Spack runtime
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for SpackRuntime {
    fn name(&self) -> &str {
        "spack"
    }

    fn description(&self) -> &str {
        "Spack - A flexible package manager for HPC and scientific computing"
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        // Spack is a system tool for HPC/scientific computing
        Ecosystem::System
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        // Spack is a Python-based tool that uses Unix shell scripts.
        // It does not natively support Windows (requires WSL).
        Platform::unix_only()
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://spack.io".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/spack/spack".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://spack.readthedocs.io".to_string(),
        );
        meta.insert("category".to_string(), "hpc".to_string());
        meta.insert("license".to_string(), "Apache-2.0 OR MIT".to_string());
        meta
    }

    fn executable_relative_path(&self, version: &str, platform: &Platform) -> String {
        // Spack extracts to spack-{version}/ directory
        // The main executable is bin/spack
        let exe_name = SpackUrlBuilder::get_executable_name(platform);
        format!("spack-{}/bin/{}", version, exe_name)
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        ctx.fetch_github_releases(
            "spack",
            "spack",
            "spack",
            GitHubReleaseOptions::new()
                .strip_v_prefix(true) // Spack uses 'v' prefix in tags
                .skip_prereleases(true),
        )
        .await
    }

    async fn download_url(&self, version: &str, _platform: &Platform) -> Result<Option<String>> {
        // Spack is platform-independent (Python-based)
        Ok(SpackUrlBuilder::download_url(version))
    }

    fn verify_installation(
        &self,
        version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_path = install_path.join(self.executable_relative_path(version, platform));

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "Spack executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec![
                    "Try reinstalling the runtime with: vx install spack".to_string(),
                    "Check if the download completed successfully".to_string(),
                    "Ensure Python 3.6+ is installed on your system".to_string(),
                ],
            )
        }
    }
}
