//! MSBuild runtime implementation
//!
//! ## RFC 0028: Bundled Runtime
//!
//! MSBuild is bundled with .NET SDK and cannot be installed independently via vx.
//! Instead, vx delegates to the .NET SDK's `dotnet msbuild` command.
//!
//! ## Detection Priority
//!
//! 1. **vx-managed .NET SDK**: Preferred for version consistency
//! 2. **System .NET SDK**: `dotnet msbuild` via PATH
//! 3. **Visual Studio MSBuild**: Windows-only standalone MSBuild.exe
//!
//! ## Version Mapping
//!
//! MSBuild versions correspond to .NET SDK versions:
//! - .NET 9.0.x SDK → MSBuild 17.x
//! - .NET 8.0.x SDK → MSBuild 17.x
//! - .NET 6.0.x SDK → MSBuild 17.x
//! - .NET 5.0.x SDK → MSBuild 16.x

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{debug, info};
use vx_runtime::{
    Ecosystem, ExecutionContext, ExecutionPrep, Platform, Runtime, RuntimeContext, VersionInfo,
};

/// MSBuild runtime implementation
///
/// MSBuild is a bundled tool that comes with .NET SDK.
/// It cannot be installed independently through vx.
#[derive(Debug, Clone, Default)]
pub struct MsbuildRuntime;

impl MsbuildRuntime {
    /// Create a new MSBuild runtime
    pub fn new() -> Self {
        Self
    }

    /// Find the dotnet executable from vx store or system PATH
    async fn find_dotnet_executable() -> Option<PathBuf> {
        // First, try vx-managed dotnet
        if let Ok(Some(dotnet_root)) = vx_paths::get_latest_runtime_root("dotnet") {
            if dotnet_root.executable_exists() {
                debug!(
                    "Found vx-managed dotnet {} at: {}",
                    dotnet_root.version,
                    dotnet_root.executable_path().display()
                );
                return Some(dotnet_root.executable_path().to_path_buf());
            }
        }

        // Fallback to system PATH
        if let Ok(path) = which::which("dotnet") {
            debug!("Found system dotnet at: {}", path.display());
            return Some(path);
        }

        None
    }

    /// Find Visual Studio MSBuild.exe on Windows
    #[cfg(windows)]
    fn find_vs_msbuild() -> Option<PathBuf> {
        // Visual Studio 2022 paths
        let vs_paths = [
            // VS 2022
            r"C:\Program Files\Microsoft Visual Studio\2022\Enterprise\MSBuild\Current\Bin\MSBuild.exe",
            r"C:\Program Files\Microsoft Visual Studio\2022\Professional\MSBuild\Current\Bin\MSBuild.exe",
            r"C:\Program Files\Microsoft Visual Studio\2022\Community\MSBuild\Current\Bin\MSBuild.exe",
            r"C:\Program Files\Microsoft Visual Studio\2022\BuildTools\MSBuild\Current\Bin\MSBuild.exe",
            // VS 2019
            r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Enterprise\MSBuild\Current\Bin\MSBuild.exe",
            r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Professional\MSBuild\Current\Bin\MSBuild.exe",
            r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\MSBuild\Current\Bin\MSBuild.exe",
            r"C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\MSBuild\Current\Bin\MSBuild.exe",
        ];

        for path in &vs_paths {
            let p = PathBuf::from(path);
            if p.exists() {
                debug!("Found Visual Studio MSBuild at: {}", p.display());
                return Some(p);
            }
        }

        None
    }

    #[cfg(not(windows))]
    fn find_vs_msbuild() -> Option<PathBuf> {
        // Visual Studio MSBuild is Windows-only
        None
    }

    /// Get MSBuild version from dotnet SDK
    async fn get_msbuild_version_from_dotnet(dotnet_exe: &Path) -> Option<String> {
        let output = Command::new(dotnet_exe)
            .args(["msbuild", "--version"])
            .output()
            .await
            .ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse version from output like "MSBuild version 17.8.5+1c7..."
            for line in stdout.lines() {
                if line.contains("MSBuild version") {
                    if let Some(version_part) = line.split_whitespace().nth(2) {
                        // Extract major.minor.patch before any suffix
                        let version = version_part.split('+').next()?;
                        return Some(version.to_string());
                    }
                }
                // Alternative format: just the version number
                if line
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                {
                    let version = line.split('+').next()?;
                    return Some(version.trim().to_string());
                }
            }
        }

        None
    }

    /// Get MSBuild version from standalone MSBuild.exe
    #[cfg(windows)]
    async fn get_msbuild_version_from_exe(msbuild_exe: &Path) -> Option<String> {
        let output = Command::new(msbuild_exe)
            .arg("-version")
            .output()
            .await
            .ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse version from the last line
            for line in stdout.lines().rev() {
                let trimmed = line.trim();
                if !trimmed.is_empty()
                    && trimmed
                        .chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                {
                    return Some(trimmed.to_string());
                }
            }
        }

        None
    }

    #[cfg(not(windows))]
    async fn get_msbuild_version_from_exe(_msbuild_exe: &Path) -> Option<String> {
        None
    }
}

#[async_trait]
impl Runtime for MsbuildRuntime {
    fn name(&self) -> &str {
        "msbuild"
    }

    fn description(&self) -> &str {
        "Microsoft Build Engine - bundled with .NET SDK"
    }

    fn aliases(&self) -> &[&str] {
        &["msbuild.exe"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://docs.microsoft.com/visualstudio/msbuild".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://learn.microsoft.com/visualstudio/msbuild/msbuild".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/dotnet/msbuild".to_string(),
        );
        meta.insert("category".to_string(), "build-tool".to_string());
        meta.insert("bundled_with".to_string(), "dotnet".to_string());
        meta
    }

    /// MSBuild is bundled - not directly installable
    fn is_version_installable(&self, _version: &str) -> bool {
        false
    }

    /// Fetch versions by detecting available MSBuild installations
    ///
    /// Since MSBuild is bundled, we detect versions from:
    /// 1. Installed .NET SDKs (via `dotnet --list-sdks`)
    /// 2. Visual Studio installations (Windows)
    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        let mut versions = Vec::new();
        let mut detected_versions = std::collections::HashSet::new();

        // Try to detect from vx-managed or system dotnet
        if let Some(dotnet_exe) = Self::find_dotnet_executable().await {
            if let Some(msbuild_version) = Self::get_msbuild_version_from_dotnet(&dotnet_exe).await
            {
                debug!("Detected MSBuild {} from dotnet SDK", msbuild_version);
                if detected_versions.insert(msbuild_version.clone()) {
                    let mut metadata = HashMap::new();
                    metadata.insert("source".to_string(), "dotnet_sdk".to_string());
                    metadata.insert("install_method".to_string(), "bundled".to_string());

                    versions.push(VersionInfo {
                        version: msbuild_version,
                        released_at: None,
                        prerelease: false,
                        lts: false,
                        download_url: None,
                        checksum: None,
                        metadata,
                    });
                }
            }
        }

        // On Windows, also check Visual Studio MSBuild
        #[cfg(windows)]
        {
            if let Some(msbuild_exe) = Self::find_vs_msbuild() {
                if let Some(msbuild_version) =
                    Self::get_msbuild_version_from_exe(&msbuild_exe).await
                {
                    debug!("Detected MSBuild {} from Visual Studio", msbuild_version);
                    if detected_versions.insert(msbuild_version.clone()) {
                        let mut metadata = HashMap::new();
                        metadata.insert("source".to_string(), "visual_studio".to_string());
                        metadata.insert("install_method".to_string(), "bundled".to_string());
                        metadata.insert(
                            "executable_path".to_string(),
                            msbuild_exe.display().to_string(),
                        );

                        versions.push(VersionInfo {
                            version: msbuild_version,
                            released_at: None,
                            prerelease: false,
                            lts: false,
                            download_url: None,
                            checksum: None,
                            metadata,
                        });
                    }
                }
            }
        }

        // If no versions detected, provide available .NET SDK versions as a hint
        if versions.is_empty() {
            // Use the dotnet releases API to show what's available
            let url = "https://raw.githubusercontent.com/dotnet/core/main/release-notes/releases-index.json";

            if let Ok(response) = ctx.http.get_json_value(url).await {
                if let Some(releases) = response.get("releases-index").and_then(|v| v.as_array()) {
                    for release in releases {
                        let channel = release
                            .get("channel-version")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default();

                        let support_phase = release
                            .get("support-phase")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");

                        // Only include active versions
                        if support_phase == "active" && !channel.is_empty() {
                            // Map .NET SDK version to approximate MSBuild version
                            // .NET 8.0/9.0 → MSBuild 17.x
                            let msbuild_version =
                                if channel.starts_with("9.") || channel.starts_with("8.") {
                                    format!("17.{} (via .NET SDK {})", channel, channel)
                                } else if channel.starts_with("7.") || channel.starts_with("6.") {
                                    format!("17.x (via .NET SDK {})", channel)
                                } else {
                                    continue;
                                };

                            let mut metadata = HashMap::new();
                            metadata.insert("source".to_string(), "dotnet_sdk".to_string());
                            metadata.insert("install_method".to_string(), "bundled".to_string());
                            metadata.insert(
                                "hint".to_string(),
                                format!("Install .NET SDK {} to get this MSBuild version", channel),
                            );

                            if !detected_versions.contains(&msbuild_version) {
                                detected_versions.insert(msbuild_version.clone());
                                versions.push(VersionInfo {
                                    version: msbuild_version,
                                    released_at: None,
                                    prerelease: false,
                                    lts: release.get("release-type").and_then(|v| v.as_str())
                                        == Some("lts"),
                                    download_url: None,
                                    checksum: None,
                                    metadata,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(versions)
    }

    /// Prepare execution for MSBuild (bundled with dotnet)
    ///
    /// This method:
    /// 1. Finds the dotnet executable (vx-managed or system)
    /// 2. Returns configuration to execute via `dotnet msbuild`
    async fn prepare_execution(
        &self,
        _version: &str,
        _ctx: &ExecutionContext,
    ) -> Result<ExecutionPrep> {
        // First, try to find dotnet executable
        if let Some(dotnet_exe) = Self::find_dotnet_executable().await {
            info!("Using MSBuild via: dotnet msbuild");
            return Ok(ExecutionPrep {
                executable_override: Some(dotnet_exe.clone()),
                command_prefix: vec!["msbuild".to_string()],
                proxy_ready: true,
                message: Some(format!(
                    "Using MSBuild bundled with .NET SDK at: {}",
                    dotnet_exe.display()
                )),
                ..Default::default()
            });
        }

        // On Windows, try Visual Studio MSBuild as fallback
        #[cfg(windows)]
        {
            if let Some(msbuild_exe) = Self::find_vs_msbuild() {
                info!("Using Visual Studio MSBuild at: {}", msbuild_exe.display());
                return Ok(ExecutionPrep {
                    executable_override: Some(msbuild_exe.clone()),
                    proxy_ready: true,
                    message: Some(format!(
                        "Using Visual Studio MSBuild at: {}",
                        msbuild_exe.display()
                    )),
                    ..Default::default()
                });
            }
        }

        // No MSBuild found
        Err(anyhow::anyhow!(
            "MSBuild is not available. Please install .NET SDK first:\n\
            \n  vx install dotnet\n\n\
            Or on Windows, install Visual Studio with C++ build tools."
        ))
    }

    /// No direct download URL - MSBuild is bundled
    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        // MSBuild is bundled with .NET SDK, no direct download
        Ok(None)
    }
}
