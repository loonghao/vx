//! VSCode runtime implementations
//!
//! This module provides runtime implementations for:
//! - Visual Studio Code editor

use crate::config::VscodeUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use tracing;
use vx_runtime::{
    layout::{ArchiveLayout, DownloadType, ExecutableLayout, PlatformLayout},
    Ecosystem, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo,
};
use vx_version_fetcher::VersionFetcherBuilder;

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
    /// - Linux (tar.gz): VSCode-linux-{arch}/bin/code
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        use vx_runtime::{Arch, Os};

        match platform.os {
            Os::Windows => "bin/code.cmd".to_string(),
            Os::MacOS => "Visual Studio Code.app/Contents/Resources/app/bin/code".to_string(),
            // Linux: include the platform-specific directory prefix
            Os::Linux => {
                let arch_str = match platform.arch {
                    Arch::Aarch64 => "arm64",
                    _ => "x64",
                };
                format!("VSCode-linux-{}/bin/code", arch_str)
            }
            _ => "bin/code".to_string(),
        }
    }

    /// Layout configuration for archive extraction
    /// Linux tar.gz extracts to VSCode-linux-{arch}/ directory
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
                    "Visual Studio Code.app/Contents/Resources/app/bin/code".to_string()
                ],
                strip_prefix: None, // macOS archive extracts directly
                permissions: Some("755".to_string()),
            }),
            linux: Some(PlatformLayout {
                // Include both x64 and arm64 paths; installer will find the matching one
                executable_paths: vec![
                    "VSCode-linux-x64/bin/code".to_string(),
                    "VSCode-linux-arm64/bin/code".to_string(),
                    "bin/code".to_string(),
                ],
                strip_prefix: None, // Don't strip - we include full paths
                permissions: Some("755".to_string()),
            }),
        })
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Use version fetcher to get versions from GitHub Releases API
        // with automatic fallback to jsDelivr CDN on rate limiting
        let fetcher = VersionFetcherBuilder::github_releases("microsoft", "vscode")
            .strip_v_prefix()
            .skip_prereleases()
            .build();

        tracing::debug!("Fetching VSCode versions using version fetcher");

        let versions = fetcher.fetch(ctx).await?;
        tracing::info!("Fetched {} VSCode versions", versions.len());

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
                candidates
                    .push("Visual Studio Code.app/Contents/Resources/app/bin/code".to_string());
                candidates.push(format!(
                    "VSCode-{}/Visual Studio Code.app/Contents/Resources/app/bin/code",
                    platform_id
                ));
            }
            vx_runtime::Os::Linux => {
                // Try architecture-specific directories
                candidates.push("VSCode-linux-x64/bin/code".to_string());
                candidates.push("VSCode-linux-arm64/bin/code".to_string());
                candidates.push(format!("VSCode-{}/bin/code", platform_id));
                candidates.push("bin/code".to_string());
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
        fn search_executable(
            root: &Path,
            file_name: &str,
            depth: usize,
            max_depth: usize,
        ) -> Option<std::path::PathBuf> {
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
            vx_runtime::Os::Windows => (
                "code.cmd",
                "On Windows, use the VS Code archive build (win32-*-archive).",
            ),
            vx_runtime::Os::MacOS => (
                "code",
                "On macOS, the archive should contain 'Visual Studio Code.app'.",
            ),
            _ => (
                "code",
                "On Linux, the archive should contain a 'bin/code' launcher.",
            ),
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
                "If this keeps failing, install VS Code via your system package manager/installer."
                    .to_string(),
            ],
        )
    }
}
