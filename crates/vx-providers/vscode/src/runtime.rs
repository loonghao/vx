//! VSCode runtime implementations
//!
//! This module provides runtime implementations for:
//! - Visual Studio Code editor

use crate::config::VscodeUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    layout::{ArchiveLayout, DownloadType, ExecutableLayout, PlatformLayout},
    Ecosystem, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo,
};

/// Visual Studio Code runtime
#[derive(Debug, Clone, Default)]
pub struct VscodeRuntime;

impl VscodeRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for VscodeRuntime {
    fn name(&self) -> &str {
        "code"
    }

    fn description(&self) -> &str {
        "Visual Studio Code - Code editing. Redefined."
    }

    fn aliases(&self) -> &[&str] {
        &["vscode", "vs-code", "visual-studio-code"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://code.visualstudio.com/".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/microsoft/vscode".to_string(),
        );
        meta.insert("license".to_string(), "MIT".to_string());
        meta.insert("category".to_string(), "editor".to_string());
        meta
    }

    /// VSCode archives have different structures per platform:
    /// - Windows (zip): bin/code.cmd (CLI wrapper) or Code.exe (GUI)
    /// - macOS (zip): Visual Studio Code.app/Contents/Resources/app/bin/code
    /// - Linux (tar.gz): VSCode-linux-x64/bin/code (needs strip_prefix)
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        use vx_runtime::Os;

        match platform.os {
            Os::Windows => "bin/code.cmd".to_string(),
            Os::MacOS => "Visual Studio Code.app/Contents/Resources/app/bin/code".to_string(),
            // After strip_prefix is applied, the path is just bin/code
            _ => "bin/code".to_string(),
        }
    }

    /// Layout configuration for archive extraction
    /// Linux tar.gz extracts to VSCode-linux-x64/ which needs to be stripped
    fn executable_layout(&self) -> Option<ExecutableLayout> {
        Some(ExecutableLayout {
            download_type: DownloadType::Archive,
            binary: None,
            archive: Some(ArchiveLayout {
                executable_paths: vec![
                    "bin/code.cmd".to_string(),
                    "bin/code".to_string(),
                    "Visual Studio Code.app/Contents/Resources/app/bin/code".to_string(),
                ],
                strip_prefix: None, // Default, overridden by platform-specific
                permissions: Some("755".to_string()),
            }),
            msi: None,
            windows: Some(PlatformLayout {
                executable_paths: vec!["bin/code.cmd".to_string(), "Code.exe".to_string()],
                strip_prefix: None, // Windows archive extracts directly
                permissions: None,
            }),
            macos: Some(PlatformLayout {
                executable_paths: vec![
                    "Visual Studio Code.app/Contents/Resources/app/bin/code".to_string(),
                ],
                strip_prefix: None, // macOS archive extracts directly
                permissions: Some("755".to_string()),
            }),
            linux: Some(PlatformLayout {
                executable_paths: vec!["bin/code".to_string()],
                strip_prefix: Some("VSCode-linux-x64".to_string()), // Linux needs strip
                permissions: Some("755".to_string()),
            }),
        })
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch version info from VSCode official API
        // This API returns { "products": [...] } with all platform builds
        let url = "https://code.visualstudio.com/sha";

        let response = ctx
            .get_cached_or_fetch("vscode", || async { ctx.http.get_json_value(url).await })
            .await?;

        let mut versions: Vec<VersionInfo> = Vec::new();
        let mut seen_versions = std::collections::HashSet::new();

        // Parse the response - it's an object with "products" array
        // Each entry has: url, name, version, productVersion, hash, timestamp, sha256hash, build, platform
        let entries = response
            .get("products")
            .and_then(|p| p.as_array())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid response format from VSCode API: expected object with 'products' array"
                )
            })?;

        for entry in entries {
            // Only include stable versions (skip insider builds)
            let build = entry.get("build").and_then(|b| b.as_str()).unwrap_or("");
            if build != "stable" {
                continue;
            }

            // Get product version (e.g., "1.107.1")
            if let Some(version) = entry.get("productVersion").and_then(|v| v.as_str()) {
                // Deduplicate versions (same version appears for multiple platforms)
                if seen_versions.contains(version) {
                    continue;
                }
                seen_versions.insert(version.to_string());

                let timestamp = entry
                    .get("timestamp")
                    .and_then(|t| t.as_i64())
                    .map(|ts| chrono::DateTime::from_timestamp(ts / 1000, 0).unwrap_or_default());

                versions.push(VersionInfo {
                    version: version.to_string(),
                    released_at: timestamp,
                    prerelease: false,
                    lts: false,
                    download_url: None,
                    checksum: None,
                    metadata: HashMap::new(),
                });
            }
        }

        if versions.is_empty() {
            return Err(anyhow::anyhow!(
                "No stable versions found in VSCode API response"
            ));
        }

        Ok(versions)
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(VscodeUrlBuilder::download_url(version, platform))
    }

    fn verify_installation(
        &self,
        version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        // Try the primary expected path first
        let primary = install_path.join(self.executable_relative_path(version, platform));
        if primary.exists() {
            return VerificationResult::success(primary);
        }

        // Common alternate layouts:
        // - Windows archive may unpack into VSCode-win32-x64/...
        // - Linux tar.gz may unpack into VSCode-linux-x64/...
        // - macOS may unpack into Visual Studio Code.app/...
        let platform_id = VscodeUrlBuilder::get_platform_string(platform);

        let mut candidates: Vec<String> = Vec::new();
        match platform.os {
            vx_runtime::Os::Windows => {
                candidates.push("bin/code.cmd".to_string());
                candidates.push(format!("VSCode-{}/bin/code.cmd", platform_id));
                candidates.push("Code.exe".to_string());
                candidates.push(format!("VSCode-{}/Code.exe", platform_id));
            }
            vx_runtime::Os::MacOS => {
                candidates.push(
                    "Visual Studio Code.app/Contents/Resources/app/bin/code".to_string(),
                );
                candidates.push(format!(
                    "VSCode-{}/Visual Studio Code.app/Contents/Resources/app/bin/code",
                    platform_id
                ));
            }
            _ => {
                candidates.push("bin/code".to_string());
                candidates.push(format!("VSCode-{}/bin/code", platform_id));
            }
        }

        for rel in &candidates {
            let p = install_path.join(rel);
            if p.exists() {
                return VerificationResult::success(p);
            }
        }

        // Last resort: shallow search for a matching executable path.
        fn search_executable(root: &Path, file_name: &str, depth: usize, max_depth: usize) -> Option<std::path::PathBuf> {
            if depth > max_depth {
                return None;
            }

            let entries = std::fs::read_dir(root).ok()?;
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if path.file_name().and_then(|n| n.to_str()) == Some(file_name) {
                        return Some(path);
                    }
                } else if path.is_dir() {
                    if let Some(found) = search_executable(&path, file_name, depth + 1, max_depth) {
                        return Some(found);
                    }
                }
            }
            None
        }

        let (needle, hint) = match platform.os {
            vx_runtime::Os::Windows => ("code.cmd", "On Windows, use the VS Code archive build (win32-*-archive)."),
            vx_runtime::Os::MacOS => ("code", "On macOS, the archive should contain 'Visual Studio Code.app'."),
            _ => ("code", "On Linux, the archive should contain a 'bin/code' launcher."),
        };

        if let Some(found) = search_executable(install_path, needle, 0, 4) {
            return VerificationResult::success(found);
        }

        VerificationResult::failure(
            vec![format!(
                "VS Code executable not found after installation (tried primary path: {}). {}",
                primary.display(),
                hint
            )],
            vec![
                "Try reinstalling with: vx install code".to_string(),
                "If this keeps failing, install VS Code via your system package manager/installer.".to_string(),
            ],
        )
    }
}
