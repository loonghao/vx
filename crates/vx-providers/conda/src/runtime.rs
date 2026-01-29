//! Conda runtime implementations
//!
//! This module provides runtime implementations for:
//! - Micromamba: Minimal standalone mamba (single binary, recommended)
//! - Conda/Mamba: Full installation via Miniforge (requires installer execution)
//!
//! Note: Micromamba is the recommended choice for vx as it's a single binary
//! that can be directly downloaded and used without running an installer.

use crate::config::CondaUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::debug;
use vx_runtime::{Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VersionInfo};

/// Micromamba minimal standalone mamba runtime
///
/// This is the **recommended** conda-compatible tool for vx because:
/// - Single binary, no installer needed
/// - Fast and lightweight (~10MB)
/// - Fully compatible with conda environments and packages
/// - Can install PyTorch, TensorFlow, CUDA, etc.
#[derive(Debug, Clone, Default)]
pub struct MicromambaRuntime;

impl MicromambaRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for MicromambaRuntime {
    fn name(&self) -> &str {
        "micromamba"
    }

    fn description(&self) -> &str {
        "Fast, minimal conda package manager (single binary)"
    }

    fn aliases(&self) -> &[&str] {
        &["umamba"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Python
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://mamba.readthedocs.io/en/latest/user_guide/micromamba.html".to_string(),
        );
        meta.insert("ecosystem".to_string(), "python".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/mamba-org/micromamba-releases".to_string(),
        );
        meta.insert("license".to_string(), "BSD-3-Clause".to_string());
        meta.insert(
            "note".to_string(),
            "Recommended for ML/AI development with PyTorch, TensorFlow, CUDA".to_string(),
        );
        meta
    }

    /// Micromamba is a single binary
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        if platform.is_windows() {
            "Library/bin/micromamba.exe".to_string()
        } else {
            "bin/micromamba".to_string()
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Micromamba has its own release cycle
        ctx.fetch_github_releases(
            "micromamba",
            "mamba-org",
            "micromamba-releases",
            GitHubReleaseOptions::new().strip_v_prefix(false),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(CondaUrlBuilder::micromamba_download_url(version, platform))
    }

    /// Post-extract hook for micromamba
    /// Set executable permissions on Unix
    fn post_extract(&self, _version: &str, install_path: &PathBuf) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let exe = install_path.join("bin/micromamba");
            if exe.exists() {
                let mut perms = std::fs::metadata(&exe)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&exe, perms)?;
                debug!("Set executable permissions for micromamba");
            }
        }
        Ok(())
    }
}

/// Conda package and environment manager runtime (via Miniforge)
///
/// Note: Miniforge requires running an installer script, which is not yet
/// fully supported by vx. For now, we recommend using `micromamba` instead.
///
/// If you need full conda, you can:
/// 1. Install Miniforge manually from https://github.com/conda-forge/miniforge
/// 2. Use `vx micromamba` which is fully compatible with conda environments
#[derive(Debug, Clone, Default)]
pub struct CondaRuntime;

impl CondaRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for CondaRuntime {
    fn name(&self) -> &str {
        "conda"
    }

    fn description(&self) -> &str {
        "Package and environment management (via Miniforge installer)"
    }

    fn aliases(&self) -> &[&str] {
        &["miniconda", "miniforge"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Python
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://conda.io/".to_string());
        meta.insert("ecosystem".to_string(), "python".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/conda-forge/miniforge".to_string(),
        );
        meta.insert("license".to_string(), "BSD-3-Clause".to_string());
        meta.insert(
            "note".to_string(),
            "Requires installer execution. Consider using 'micromamba' for simpler setup.".to_string(),
        );
        meta
    }

    /// Miniforge has a special installation structure
    /// Windows: Installs to a directory, conda.exe is in Scripts/
    /// Unix: Installs to a directory, conda is in bin/
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        CondaUrlBuilder::get_conda_executable_path(platform)
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Miniforge uses tags like "24.3.0-0"
        ctx.fetch_github_releases(
            "conda",
            "conda-forge",
            "miniforge",
            GitHubReleaseOptions::new().strip_v_prefix(false),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(CondaUrlBuilder::conda_download_url(version, platform))
    }

    /// Post-extract hook for Miniforge installation
    ///
    /// Note: Miniforge downloads are installer scripts (.sh on Unix, .exe on Windows)
    /// that need to be executed to complete the installation. This is a placeholder
    /// for future implementation.
    fn post_extract(&self, version: &str, install_path: &PathBuf) -> Result<()> {
        debug!(
            "Miniforge post-extract for version {} at {:?}",
            version, install_path
        );

        // TODO: Implement installer execution
        // For now, users should use micromamba or install Miniforge manually

        Ok(())
    }
}

/// Mamba fast package manager runtime (bundled with Miniforge)
///
/// Note: Mamba is bundled with Miniforge. Since Miniforge requires an installer,
/// we recommend using `micromamba` instead, which is a standalone binary.
#[derive(Debug, Clone, Default)]
pub struct MambaRuntime;

impl MambaRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for MambaRuntime {
    fn name(&self) -> &str {
        "mamba"
    }

    fn description(&self) -> &str {
        "Fast package manager (bundled with Miniforge)"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Python
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://mamba.readthedocs.io/".to_string());
        meta.insert("ecosystem".to_string(), "python".to_string());
        meta.insert("bundled_with".to_string(), "conda".to_string());
        meta.insert(
            "note".to_string(),
            "Bundled with Miniforge. Consider using 'micromamba' for standalone usage.".to_string(),
        );
        meta
    }

    /// Mamba is bundled with Miniforge, located in the same directory structure
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        CondaUrlBuilder::get_mamba_executable_path(platform)
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Mamba is bundled with Miniforge, use the same versions
        let conda_runtime = CondaRuntime::new();
        conda_runtime.fetch_versions(ctx).await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // Mamba is bundled with Miniforge
        Ok(CondaUrlBuilder::conda_download_url(version, platform))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vx_runtime::{Arch, Os};

    #[test]
    fn test_micromamba_runtime_name() {
        let runtime = MicromambaRuntime::new();
        assert_eq!(runtime.name(), "micromamba");
    }

    #[test]
    fn test_micromamba_runtime_aliases() {
        let runtime = MicromambaRuntime::new();
        assert!(runtime.aliases().contains(&"umamba"));
    }

    #[test]
    fn test_conda_runtime_name() {
        let runtime = CondaRuntime::new();
        assert_eq!(runtime.name(), "conda");
    }

    #[test]
    fn test_conda_runtime_aliases() {
        let runtime = CondaRuntime::new();
        assert!(runtime.aliases().contains(&"miniforge"));
        assert!(runtime.aliases().contains(&"miniconda"));
    }

    #[test]
    fn test_mamba_runtime_name() {
        let runtime = MambaRuntime::new();
        assert_eq!(runtime.name(), "mamba");
    }

    #[test]
    fn test_micromamba_executable_paths_windows() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };

        let micromamba = MicromambaRuntime::new();
        assert_eq!(
            micromamba.executable_relative_path("1.5.8-0", &platform),
            "Library/bin/micromamba.exe"
        );
    }

    #[test]
    fn test_micromamba_executable_paths_unix() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };

        let micromamba = MicromambaRuntime::new();
        assert_eq!(
            micromamba.executable_relative_path("1.5.8-0", &platform),
            "bin/micromamba"
        );
    }

    #[test]
    fn test_conda_executable_paths_windows() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };

        let conda = CondaRuntime::new();
        assert!(conda
            .executable_relative_path("24.3.0-0", &platform)
            .contains("conda.exe"));

        let mamba = MambaRuntime::new();
        assert!(mamba
            .executable_relative_path("24.3.0-0", &platform)
            .contains("mamba.exe"));
    }

    #[test]
    fn test_conda_executable_paths_unix() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };

        let conda = CondaRuntime::new();
        assert!(conda
            .executable_relative_path("24.3.0-0", &platform)
            .contains("bin/conda"));
    }
}
