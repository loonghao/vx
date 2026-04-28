//! Conda runtime implementations
//!
//! This module provides runtime implementations for:
//! - Micromamba: Minimal standalone mamba (single binary, recommended)
//! - Conda/Mamba: Full installation via Miniforge

use crate::config::CondaUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use vx_runtime::{
    Ecosystem, Runtime, RuntimeContext, VersionInfo,
    layout::{ArchiveLayout, DownloadType, ExecutableLayout},
    platform::Platform,
};

/// Fetch versions from GitHub releases API.
async fn fetch_github_releases(
    ctx: &RuntimeContext,
    _tool_name: &str,
    owner: &str,
    repo: &str,
    skip_prereleases: bool,
) -> Result<Vec<VersionInfo>> {
    let url = format!("https://api.github.com/repos/{}/{}/releases", owner, repo);
    let response = ctx.http.get_json_value(&url).await?;

    let mut versions = Vec::new();
    if let Some(releases) = response.as_array() {
        for release in releases {
            if skip_prereleases && release.get("prerelease").and_then(|v| v.as_bool()).unwrap_or(false) {
                continue;
            }
            if let Some(tag) = release.get("tag_name").and_then(|v| v.as_str()) {
                let mut info = VersionInfo::new(tag);
                info.prerelease = release.get("prerelease").and_then(|v| v.as_bool()).unwrap_or(false);
                versions.push(info);
            }
        }
    }
    Ok(versions)
}

// ---------------------------------------------------------------------------
// Micromamba Runtime
// ---------------------------------------------------------------------------

/// Micromamba minimal standalone mamba runtime
///
/// Recommended for vx because:
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

    fn aliases(&self) -> Vec<&str> {
        vec!["umamba"]
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
        meta.insert(
            "repository".to_string(),
            "https://github.com/mamba-org/micromamba-releases".to_string(),
        );
        meta.insert("license".to_string(), "BSD-3-Clause".to_string());
        meta
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        fetch_github_releases(
            ctx,
            "micromamba",
            "mamba-org",
            "micromamba-releases",
            true,
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(CondaUrlBuilder::micromamba_download_url(version, platform))
    }

    fn executable_layout(&self) -> Option<ExecutableLayout> {
        Some(ExecutableLayout {
            download_type: DownloadType::Archive,
            binary: None,
            archive: Some(ArchiveLayout {
                executable_paths: vec![
                    "Library/bin/micromamba.exe".to_string(),
                    "bin/micromamba".to_string(),
                ],
                strip_prefix: None,
                permissions: None,
            }),
            windows: None,
            macos: None,
            linux: None,
            msi: None,
        })
    }

    fn post_extract(&self, _version: &str, _install_path: &std::path::Path) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let exe = _install_path.join("bin/micromamba");
            if exe.exists() {
                let mut perms = std::fs::metadata(&exe)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&exe, perms)?;
                tracing::debug!("Set executable permissions for micromamba");
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Conda Runtime (via Miniforge)
// ---------------------------------------------------------------------------

/// Conda package and environment manager runtime (via Miniforge)
///
/// Note: Miniforge provides a full Conda installation including conda and mamba.
/// The installer (.sh/.exe) needs to be executed, which is more complex than
/// micromamba's direct binary approach.
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
        "Conda package and environment manager (via Miniforge)"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["miniforge", "miniconda"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Python
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://conda.io/".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/conda-forge/miniforge".to_string(),
        );
        meta.insert("license".to_string(), "BSD-3-Clause".to_string());
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        if platform.is_windows() {
            "Scripts\\conda.exe".to_string()
        } else {
            "bin/conda".to_string()
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        fetch_github_releases(
            ctx,
            "conda",
            "conda-forge",
            "miniforge",
            true,
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(CondaUrlBuilder::conda_download_url(version, platform))
    }
}

// ---------------------------------------------------------------------------
// Mamba Runtime (bundled with Miniforge)
// ---------------------------------------------------------------------------

/// Mamba fast package manager (bundled with Miniforge)
///
/// Mamba is a fast, robust, and cross-platform package manager.
/// It's automatically included when installing Conda via Miniforge.
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
        "Fast conda-compatible package manager (bundled with Miniforge)"
    }

    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Python
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://mamba.readthedocs.io/".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/mamba-org/mamba".to_string(),
        );
        meta.insert("license".to_string(), "BSD-3-Clause".to_string());
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        if platform.is_windows() {
            "Scripts\\mamba.exe".to_string()
        } else {
            "bin/mamba".to_string()
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Mamba is bundled with Miniforge, use same versions
        fetch_github_releases(
            ctx,
            "mamba",
            "conda-forge",
            "miniforge",
            true,
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // Mamba is bundled with Miniforge
        Ok(CondaUrlBuilder::conda_download_url(version, platform))
    }
}
