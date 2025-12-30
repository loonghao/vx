//! UV runtime implementations
//!
//! This module provides runtime implementations for:
//! - UV Python package installer
//! - UVX Python application runner

use crate::config::UvUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info, warn};
use vx_runtime::{Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VersionInfo};

/// UV Python package installer runtime
#[derive(Debug, Clone, Default)]
pub struct UvRuntime;

impl UvRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for UvRuntime {
    fn name(&self) -> &str {
        "uv"
    }

    fn description(&self) -> &str {
        "An extremely fast Python package installer and resolver"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Python
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://docs.astral.sh/uv/".to_string(),
        );
        meta.insert("ecosystem".to_string(), "python".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/astral-sh/uv".to_string(),
        );
        meta.insert("license".to_string(), "MIT OR Apache-2.0".to_string());
        meta
    }

    /// UV archives have different structures per platform:
    /// - Windows (zip): uv.exe (direct, no subdirectory)
    /// - Linux/macOS (tar.gz): uv-{platform}/uv (in subdirectory)
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        if platform.is_windows() {
            // Windows zip extracts directly to install directory
            platform.exe_name("uv")
        } else {
            // Linux/macOS tar.gz extracts to a subdirectory
            let platform_str = UvUrlBuilder::get_platform_string(platform);
            format!("uv-{}/uv", platform_str)
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // UV tags don't have 'v' prefix (e.g., "0.5.0")
        ctx.fetch_github_releases(
            "uv",
            "astral-sh",
            "uv",
            GitHubReleaseOptions::new().strip_v_prefix(false),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(UvUrlBuilder::download_url(version, platform))
    }

    /// Pre-run hook for uv commands
    ///
    /// For "uv run" commands, ensures project dependencies are synced first.
    /// This is essential for CI environments where .venv doesn't exist yet.
    async fn pre_run(&self, args: &[String], executable: &Path) -> Result<bool> {
        // Only handle "uv run" commands
        if args.first().is_some_and(|a| a == "run") {
            self.ensure_sync(executable).await?;
        }
        Ok(true) // Continue with execution
    }
}

impl UvRuntime {
    /// Ensure project dependencies are synced before running
    async fn ensure_sync(&self, executable: &Path) -> Result<()> {
        // Check if pyproject.toml exists in current directory
        let pyproject = Path::new("pyproject.toml");
        if !pyproject.exists() {
            debug!("No pyproject.toml found, skipping uv sync");
            return Ok(());
        }

        // Check if .venv exists - if not, we need to sync
        let venv = Path::new(".venv");
        if venv.exists() {
            debug!(".venv exists, assuming dependencies are synced");
            return Ok(());
        }

        info!("Running 'uv sync' to install project dependencies...");

        let status = Command::new(executable)
            .arg("sync")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await?;

        if !status.success() {
            warn!("'uv sync' failed, continuing anyway...");
        }

        Ok(())
    }
}

/// UVX Python application runner runtime
#[derive(Debug, Clone, Default)]
pub struct UvxRuntime;

impl UvxRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for UvxRuntime {
    fn name(&self) -> &str {
        "uvx"
    }

    fn description(&self) -> &str {
        "Python application runner"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Python
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://docs.astral.sh/uv/".to_string(),
        );
        meta.insert("ecosystem".to_string(), "python".to_string());
        meta.insert("bundled_with".to_string(), "uv".to_string());
        meta
    }

    /// UVX archives have different structures per platform:
    /// - Windows (zip): uvx.exe (direct, no subdirectory)
    /// - Linux/macOS (tar.gz): uv-{platform}/uvx (in subdirectory)
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        if platform.is_windows() {
            // Windows zip extracts directly to install directory
            platform.exe_name("uvx")
        } else {
            // Linux/macOS tar.gz extracts to a subdirectory
            let platform_str = UvUrlBuilder::get_platform_string(platform);
            format!("uv-{}/uvx", platform_str)
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // UVX is bundled with UV
        let uv_runtime = UvRuntime::new();
        uv_runtime.fetch_versions(ctx).await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // UVX is bundled with UV
        Ok(UvUrlBuilder::download_url(version, platform))
    }
}
