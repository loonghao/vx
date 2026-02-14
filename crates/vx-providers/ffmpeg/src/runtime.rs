//! FFmpeg runtime implementations
//!
//! This module provides runtime implementations for:
//! - FFmpeg: The main multimedia framework
//! - FFprobe: Multimedia stream analyzer
//! - FFplay: Simple media player
//!
//! Uses BtbN/FFmpeg-Builds for Windows and Linux binaries.

use crate::config::{FfmpegBuild, FfmpegUrlBuilder};
use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::debug;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo};

/// FFmpeg runtime implementation
#[derive(Debug, Clone, Default)]
pub struct FfmpegRuntime;

impl FfmpegRuntime {
    /// Create a new FFmpeg runtime
    pub fn new() -> Self {
        Self
    }

    /// Parse versions from BtbN/FFmpeg-Builds release assets
    ///
    /// Asset names follow pattern: ffmpeg-n{version}-latest-{platform}-{license}-{version}.{ext}
    /// Example: ffmpeg-n8.0-latest-win64-gpl-8.0.zip
    fn parse_versions_from_assets(assets: &[serde_json::Value]) -> Vec<String> {
        let re = Regex::new(r"ffmpeg-n(\d+\.\d+)-latest-").ok();
        let mut versions = Vec::new();

        if let Some(regex) = re {
            for asset in assets {
                if let Some(name) = asset.get("name").and_then(|n| n.as_str())
                    && let Some(cap) = regex.captures(name)
                    && let Some(version) = cap.get(1)
                {
                    let v = version.as_str().to_string();
                    if !versions.contains(&v) {
                        versions.push(v);
                    }
                }
            }
        }

        // Sort by version (newest first)
        versions.sort_by(|a, b| {
            let parse =
                |s: &str| -> Vec<u32> { s.split('.').filter_map(|p| p.parse().ok()).collect() };
            parse(b).cmp(&parse(a))
        });

        versions
    }
}

#[async_trait]
impl Runtime for FfmpegRuntime {
    fn name(&self) -> &str {
        "ffmpeg"
    }

    fn description(&self) -> &str {
        "FFmpeg - A complete, cross-platform solution to record, convert and stream audio and video"
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://ffmpeg.org".to_string());
        meta.insert(
            "documentation".to_string(),
            "https://ffmpeg.org/documentation.html".to_string(),
        );
        meta.insert("category".to_string(), "media-processing".to_string());
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        FfmpegUrlBuilder::get_executable_relative_path("ffmpeg", platform)
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch from BtbN/FFmpeg-Builds GitHub releases
        // This is a more reliable source than gyan.dev
        let url = "https://api.github.com/repos/BtbN/FFmpeg-Builds/releases/tags/latest";

        match ctx.http.get_json_value(url).await {
            Ok(release) => {
                if let Some(assets) = release.get("assets").and_then(|a| a.as_array()) {
                    let versions = Self::parse_versions_from_assets(assets);
                    if !versions.is_empty() {
                        // Add "latest" as an option for master builds
                        let mut result: Vec<VersionInfo> = vec![VersionInfo::new("latest")];
                        result.extend(versions.into_iter().map(VersionInfo::new));
                        return Ok(result);
                    }
                }
            }
            Err(e) => {
                debug!("Failed to fetch versions from GitHub: {}", e);
            }
        }

        // Fallback: provide known stable versions
        Ok(vec![
            VersionInfo::new("latest"),
            VersionInfo::new("8.0"),
            VersionInfo::new("7.1"),
            VersionInfo::new("7.0"),
            VersionInfo::new("6.1"),
            VersionInfo::new("6.0"),
        ])
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(FfmpegUrlBuilder::download_url(
            version,
            platform,
            FfmpegBuild::Gpl,
        ))
    }

    fn post_extract(&self, _version: &str, install_path: &PathBuf) -> Result<()> {
        // FFmpeg archives have dynamic directory names (e.g., ffmpeg-8.0.1-essentials_build)
        // We need to flatten the structure to: install_path/bin/ffmpeg.exe

        use std::fs;

        // Find the nested directory (should be only one)
        let entries: Vec<_> = fs::read_dir(install_path)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();

        if entries.len() != 1 {
            // If no nested directory or multiple directories, assume already correct structure
            return Ok(());
        }

        let nested_dir = entries[0].path();
        debug!("Found nested FFmpeg directory: {:?}", nested_dir);

        // Check if bin/ exists in nested dir
        let nested_bin = nested_dir.join("bin");
        if !nested_bin.exists() {
            debug!("No bin/ directory in nested dir, skipping reorganization");
            return Ok(());
        }

        // Move bin/ directory to install_path/bin
        let target_bin = install_path.join("bin");
        if target_bin.exists() {
            fs::remove_dir_all(&target_bin)?;
        }

        debug!("Moving {:?} to {:?}", nested_bin, target_bin);
        fs::rename(&nested_bin, &target_bin)?;

        // Also move doc/ and README if they exist
        for name in &["doc", "README.txt", "LICENSE"] {
            let nested_item = nested_dir.join(name);
            if nested_item.exists() {
                let target_item = install_path.join(name);
                let _ = fs::rename(&nested_item, &target_item); // Ignore errors
            }
        }

        // Remove the now-empty nested directory
        let _ = fs::remove_dir_all(&nested_dir); // Ignore errors

        debug!("FFmpeg directory reorganization completed");
        Ok(())
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_path = install_path.join(self.executable_relative_path("", platform));

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            // Try alternate paths
            let alt_paths = [
                install_path.join(FfmpegUrlBuilder::get_executable_name("ffmpeg", platform)),
                install_path
                    .join("bin")
                    .join(FfmpegUrlBuilder::get_executable_name("ffmpeg", platform)),
            ];

            for alt in alt_paths {
                if alt.exists() {
                    return VerificationResult::success(alt);
                }
            }

            VerificationResult::failure(
                vec![format!(
                    "FFmpeg executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling FFmpeg".to_string()],
            )
        }
    }
}

/// FFprobe runtime implementation (bundled with FFmpeg)
#[derive(Debug, Clone, Default)]
pub struct FfprobeRuntime;

impl FfprobeRuntime {
    /// Create a new FFprobe runtime
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for FfprobeRuntime {
    fn name(&self) -> &str {
        "ffprobe"
    }

    fn description(&self) -> &str {
        "FFprobe - Multimedia stream analyzer"
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://ffmpeg.org".to_string());
        meta.insert(
            "documentation".to_string(),
            "https://ffmpeg.org/ffprobe.html".to_string(),
        );
        meta.insert("category".to_string(), "media-processing".to_string());
        meta.insert("bundled_with".to_string(), "ffmpeg".to_string());
        meta
    }

    /// FFprobe is bundled with FFmpeg, so store under "ffmpeg" directory
    fn store_name(&self) -> &str {
        "ffmpeg"
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        FfmpegUrlBuilder::get_executable_relative_path("ffprobe", platform)
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // FFprobe is bundled with FFmpeg, use FFmpeg's versions
        FfmpegRuntime::new().fetch_versions(ctx).await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // FFprobe is bundled with FFmpeg
        FfmpegRuntime::new().download_url(version, platform).await
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_path = install_path.join(self.executable_relative_path("", platform));

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            let alt_paths = [
                install_path.join(FfmpegUrlBuilder::get_executable_name("ffprobe", platform)),
                install_path
                    .join("bin")
                    .join(FfmpegUrlBuilder::get_executable_name("ffprobe", platform)),
            ];

            for alt in alt_paths {
                if alt.exists() {
                    return VerificationResult::success(alt);
                }
            }

            VerificationResult::failure(
                vec![format!(
                    "FFprobe executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["FFprobe is bundled with FFmpeg. Try reinstalling FFmpeg.".to_string()],
            )
        }
    }
}

/// FFplay runtime implementation (bundled with FFmpeg)
#[derive(Debug, Clone, Default)]
pub struct FfplayRuntime;

impl FfplayRuntime {
    /// Create a new FFplay runtime
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for FfplayRuntime {
    fn name(&self) -> &str {
        "ffplay"
    }

    fn description(&self) -> &str {
        "FFplay - Simple media player"
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://ffmpeg.org".to_string());
        meta.insert(
            "documentation".to_string(),
            "https://ffmpeg.org/ffplay.html".to_string(),
        );
        meta.insert("category".to_string(), "media-processing".to_string());
        meta.insert("bundled_with".to_string(), "ffmpeg".to_string());
        meta
    }

    /// FFplay is bundled with FFmpeg, so store under "ffmpeg" directory
    fn store_name(&self) -> &str {
        "ffmpeg"
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        FfmpegUrlBuilder::get_executable_relative_path("ffplay", platform)
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // FFplay is bundled with FFmpeg, use FFmpeg's versions
        FfmpegRuntime::new().fetch_versions(ctx).await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // FFplay is bundled with FFmpeg
        FfmpegRuntime::new().download_url(version, platform).await
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_path = install_path.join(self.executable_relative_path("", platform));

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            let alt_paths = [
                install_path.join(FfmpegUrlBuilder::get_executable_name("ffplay", platform)),
                install_path
                    .join("bin")
                    .join(FfmpegUrlBuilder::get_executable_name("ffplay", platform)),
            ];

            for alt in alt_paths {
                if alt.exists() {
                    return VerificationResult::success(alt);
                }
            }

            VerificationResult::failure(
                vec![format!(
                    "FFplay executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["FFplay is bundled with FFmpeg. Try reinstalling FFmpeg.".to_string()],
            )
        }
    }
}
