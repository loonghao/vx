//! ImageMagick runtime implementations
//!
//! This module provides runtime implementations for:
//! - Magick: The unified ImageMagick command-line tool (since v7)
//! - Convert: Legacy convert command (bundled with magick)
//!
//! ImageMagick is a powerful image manipulation software suite.
//! Homepage: https://imagemagick.org/
//!
//! # Installation Flow
//!
//! ImageMagick installation is platform-dependent:
//! - **Linux**: Direct download of AppImage binary (handled by this runtime)
//! - **macOS**: Requires Homebrew (dependency declared in provider.toml)
//! - **Windows**: Requires Chocolatey or Scoop (dependency declared in provider.toml)
//!
//! Platform-specific package manager dependencies are declared in `provider.toml`
//! under `system_deps.pre_depends`. The resolver automatically ensures these
//! dependencies are installed before attempting ImageMagick installation.

use crate::config::ImageMagickUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use vx_runtime::{
    Ecosystem, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo,
};

/// Magick runtime implementation (main ImageMagick CLI)
#[derive(Debug, Clone, Default)]
pub struct MagickRuntime;

impl MagickRuntime {
    /// Create a new Magick runtime
    pub fn new() -> Self {
        Self
    }

    /// Parse versions from GitHub releases
    fn parse_version(tag_name: &str) -> Option<String> {
        // ImageMagick versions are like "7.1.2-12"
        // We keep the full version string
        let re = Regex::new(r"^(\d+\.\d+\.\d+-\d+)$").ok()?;
        if re.is_match(tag_name) {
            Some(tag_name.to_string())
        } else {
            None
        }
    }
}

#[async_trait]
impl Runtime for MagickRuntime {
    fn name(&self) -> &str {
        "magick"
    }

    fn description(&self) -> &str {
        "ImageMagick unified command-line tool for image manipulation"
    }

    fn aliases(&self) -> &[&str] {
        &["imagemagick"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://imagemagick.org/".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/ImageMagick/ImageMagick".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://imagemagick.org/script/command-line-tools.php".to_string(),
        );
        meta.insert("category".to_string(), "image-processing".to_string());
        meta.insert("license".to_string(), "Apache-2.0".to_string());
        meta
    }

    fn executable_dir_path(&self, _version: &str, _platform: &Platform) -> Option<String> {
        // On Linux (AppImage), the binary goes to bin/
        Some("bin".to_string())
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch from GitHub releases
        let url = "https://api.github.com/repos/ImageMagick/ImageMagick/releases?per_page=30";

        let response = ctx.http.get_json_value(url).await?;

        let versions: Vec<VersionInfo> = response
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid response from GitHub API"))?
            .iter()
            .filter_map(|release| {
                let tag_name = release.get("tag_name")?.as_str()?;
                let prerelease = release.get("prerelease")?.as_bool().unwrap_or(false);

                // Skip prereleases
                if prerelease {
                    return None;
                }

                let version = Self::parse_version(tag_name)?;
                let published_at = release
                    .get("published_at")
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string());

                Some(
                    VersionInfo::new(&version)
                        .with_lts(false)
                        .with_prerelease(false)
                        .with_release_date(published_at.unwrap_or_default()),
                )
            })
            .collect();

        if versions.is_empty() {
            // Fallback to known versions
            return Ok(vec![
                VersionInfo::new("7.1.2-12"),
                VersionInfo::new("7.1.2-11"),
                VersionInfo::new("7.1.2-10"),
                VersionInfo::new("7.1.1-43"),
            ]);
        }

        Ok(versions)
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // Returns None if no direct download available (Windows/macOS)
        // Platform-specific package manager dependencies are handled by vx-resolver
        // based on system_deps.pre_depends in provider.toml
        Ok(ImageMagickUrlBuilder::download_url(version, platform))
    }

    // Note: install() uses the default Runtime implementation
    // Platform-specific dependencies (brew on macOS, choco/scoop on Windows)
    // are declared in provider.toml and resolved automatically by vx-resolver

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_name = ImageMagickUrlBuilder::get_executable_name(platform);
        let exe_path = install_path.join("bin").join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            // Check if it's in root directory (fallback)
            let alt_path = install_path.join(ImageMagickUrlBuilder::get_executable_name(platform));
            if alt_path.exists() {
                return VerificationResult::success(alt_path);
            }

            VerificationResult::failure(
                vec![format!(
                    "ImageMagick executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec![
                    "For Linux: Try reinstalling via vx".to_string(),
                    "For macOS: Run 'brew install imagemagick'".to_string(),
                    "For Windows: Run 'choco install imagemagick' or 'scoop install imagemagick'"
                        .to_string(),
                ],
            )
        }
    }

    /// Check if ImageMagick is installed (either via vx or system)
    async fn is_installed(&self, version: &str, ctx: &RuntimeContext) -> Result<bool> {
        // First check vx store
        let install_path = ctx.paths.version_store_dir(self.store_name(), version);
        if ctx.fs.exists(&install_path) {
            return Ok(true);
        }

        // Then check system installation (only for "system" version)
        if version == "system" {
            return Ok(which::which("magick").is_ok());
        }

        Ok(false)
    }

    /// Get installed versions (including system installation)
    async fn installed_versions(&self, ctx: &RuntimeContext) -> Result<Vec<String>> {
        let mut versions = Vec::new();

        // Check vx store for installed versions
        let runtime_dir = ctx.paths.runtime_store_dir(self.store_name());
        if ctx.fs.exists(&runtime_dir) {
            let entries = ctx.fs.read_dir(&runtime_dir)?;
            for entry in entries {
                if ctx.fs.is_dir(&entry) {
                    if let Some(name) = entry.file_name().and_then(|n| n.to_str()) {
                        versions.push(name.to_string());
                    }
                }
            }
        }

        // Check system installation
        if let Ok(path) = which::which("magick") {
            // Try to detect version
            if let Ok(output) = Command::new(&path).arg("--version").output() {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    // Parse version from output like "Version: ImageMagick 7.1.1-43 Q16-HDRI..."
                    if let Some(version) = Self::extract_version_from_output(&stdout) {
                        // Only add if not already in the list (to avoid duplicates)
                        if !versions.contains(&version) {
                            versions.push(format!("{} (system)", version));
                        }
                    } else {
                        // Couldn't parse version, just indicate system installation
                        versions.push("system".to_string());
                    }
                }
            } else {
                versions.push("system".to_string());
            }
        }

        // Sort versions (newest first, but keep system at the end)
        versions.sort_by(|a, b| {
            let a_is_system = a.contains("system");
            let b_is_system = b.contains("system");
            match (a_is_system, b_is_system) {
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                _ => b.cmp(a),
            }
        });

        Ok(versions)
    }

    /// Uninstall ImageMagick
    async fn uninstall(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        // Check if this is a system installation
        if version == "system" || version.contains("(system)") {
            let platform = Platform::current();
            let uninstall_cmd = match platform.os_name() {
                "windows" => "choco uninstall imagemagick  OR  scoop uninstall imagemagick",
                "macos" => "brew uninstall imagemagick",
                "linux" => "sudo apt remove imagemagick  OR  sudo dnf remove ImageMagick",
                _ => "Use your system package manager to uninstall ImageMagick",
            };

            return Err(anyhow::anyhow!(
                "Cannot uninstall system-installed ImageMagick via vx.\n\
                 Use your system package manager:\n  {}",
                uninstall_cmd
            ));
        }

        // Uninstall from vx store
        let install_path = ctx.paths.version_store_dir(self.store_name(), version);
        if ctx.fs.exists(&install_path) {
            ctx.fs.remove_dir_all(&install_path)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Version {} is not installed via vx",
                version
            ))
        }
    }

    /// Custom version resolver for ImageMagick's special version format (e.g., 7.1.2-12)
    ///
    /// ImageMagick uses a version format like "7.1.2-12" where the "-12" is NOT a prerelease
    /// tag but rather a build/patch number. The default vx version resolver interprets "-"
    /// as a prerelease separator, which causes "latest" to fail finding any stable versions.
    async fn resolve_version(&self, version: &str, ctx: &RuntimeContext) -> Result<String> {
        let versions = self.fetch_versions(ctx).await?;

        if versions.is_empty() {
            return Err(anyhow::anyhow!("No versions available for magick"));
        }

        let trimmed = version.trim();

        // Handle "latest" - return the first (newest) version
        if trimmed.is_empty() || trimmed == "latest" {
            return Ok(versions[0].version.clone());
        }

        // Handle exact match
        if let Some(v) = versions.iter().find(|v| v.version == trimmed) {
            return Ok(v.version.clone());
        }

        // Handle partial match (e.g., "7.1" matches "7.1.2-12")
        if let Some(v) = versions.iter().find(|v| v.version.starts_with(trimmed)) {
            return Ok(v.version.clone());
        }

        // No match found
        let available_range = if versions.len() > 1 {
            format!(
                "{} to {}",
                versions.last().map(|v| v.version.as_str()).unwrap_or("?"),
                versions.first().map(|v| v.version.as_str()).unwrap_or("?")
            )
        } else {
            versions
                .first()
                .map(|v| v.version.clone())
                .unwrap_or_default()
        };

        Err(anyhow::anyhow!(
            "No version found for magick matching '{}'. Available versions: {}",
            version,
            available_range
        ))
    }
}

impl MagickRuntime {
    /// Extract version from ImageMagick --version output
    fn extract_version_from_output(output: &str) -> Option<String> {
        // Output format: "Version: ImageMagick 7.1.1-43 Q16-HDRI..."
        let re = Regex::new(r"ImageMagick\s+(\d+\.\d+\.\d+-\d+)").ok()?;
        re.captures(output)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
    }
}

/// Convert runtime implementation (legacy, bundled with magick)
#[derive(Debug, Clone, Default)]
pub struct ConvertRuntime;

impl ConvertRuntime {
    /// Create a new Convert runtime
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for ConvertRuntime {
    fn name(&self) -> &str {
        "convert"
    }

    fn description(&self) -> &str {
        "ImageMagick convert tool (legacy, use 'magick convert' instead)"
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://imagemagick.org/".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://imagemagick.org/script/convert.php".to_string(),
        );
        meta.insert("category".to_string(), "image-processing".to_string());
        meta.insert("bundled_with".to_string(), "magick".to_string());
        meta.insert(
            "note".to_string(),
            "In ImageMagick 7+, use 'magick convert' instead of 'convert'".to_string(),
        );
        meta
    }

    /// Convert is bundled with magick, so store under "magick" directory
    fn store_name(&self) -> &str {
        "magick"
    }

    fn executable_dir_path(&self, _version: &str, _platform: &Platform) -> Option<String> {
        Some("bin".to_string())
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Convert is bundled with magick, use magick's versions
        MagickRuntime::new().fetch_versions(ctx).await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // Convert is bundled with magick, delegate to MagickRuntime
        MagickRuntime::new().download_url(version, platform).await
    }

    // Note: install() uses default Runtime implementation (delegates to MagickRuntime via bundled_with)

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        // In ImageMagick 7+, convert is typically a symlink to magick
        // or accessed via 'magick convert'
        let magick_exe = ImageMagickUrlBuilder::get_executable_name(platform);
        let magick_path = install_path.join("bin").join(magick_exe);

        if magick_path.exists() {
            // If magick exists, convert functionality is available
            VerificationResult::success(magick_path)
        } else {
            VerificationResult::failure(
                vec!["Convert requires ImageMagick (magick) to be installed".to_string()],
                vec![
                    "Install ImageMagick: vx install magick".to_string(),
                    "Note: In ImageMagick 7+, use 'magick convert' instead of 'convert'"
                        .to_string(),
                ],
            )
        }
    }

    /// Check if convert is installed (via magick)
    async fn is_installed(&self, version: &str, ctx: &RuntimeContext) -> Result<bool> {
        // Convert is bundled with magick
        MagickRuntime::new().is_installed(version, ctx).await
    }

    /// Get installed versions (same as magick)
    async fn installed_versions(&self, ctx: &RuntimeContext) -> Result<Vec<String>> {
        MagickRuntime::new().installed_versions(ctx).await
    }

    /// Uninstall convert (same as uninstalling magick)
    async fn uninstall(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        MagickRuntime::new().uninstall(version, ctx).await
    }

    /// Custom version resolver (delegates to MagickRuntime)
    async fn resolve_version(&self, version: &str, ctx: &RuntimeContext) -> Result<String> {
        MagickRuntime::new().resolve_version(version, ctx).await
    }
}
