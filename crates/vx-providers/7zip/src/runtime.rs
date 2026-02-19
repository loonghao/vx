//! 7-Zip runtime implementation

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use vx_runtime::{
    Ecosystem, ExecutionContext, ExecutionPrep, GitHubReleaseOptions, InstallResult, Platform,
    Runtime, RuntimeContext, VerificationResult, VersionInfo,
};

/// 7-Zip runtime
#[derive(Debug, Clone)]
pub struct SevenZipRuntime;

impl SevenZipRuntime {
    /// Create a new 7-Zip runtime
    pub fn new() -> Self {
        Self
    }

    /// Get the executable name for the current platform
    fn exe_name(platform: &Platform) -> &'static str {
        if platform.is_windows() {
            "7z.exe"
        } else {
            "7z"
        }
    }
}

impl Default for SevenZipRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for SevenZipRuntime {
    fn name(&self) -> &str {
        "7zip"
    }

    fn description(&self) -> &str {
        "7-Zip - High compression ratio file archiver supporting 7z, ZIP, TAR, GZ, XZ and more"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn aliases(&self) -> &[&str] {
        &["7z", "7za", "7zz"]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://www.7-zip.org".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/ip7z/7zip".to_string(),
        );
        meta.insert("category".to_string(), "archive".to_string());
        meta.insert("license".to_string(), "LGPL-2.1".to_string());
        meta
    }

    fn executable_name(&self) -> &str {
        "7z"
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        Self::exe_name(platform).to_string()
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // 7-Zip releases on GitHub: tags like "24.09", "23.01", etc.
        ctx.fetch_github_releases("7zip", "ip7z", "7zip", GitHubReleaseOptions::new())
            .await
    }

    fn is_version_installable(&self, _version: &str) -> bool {
        true
    }

    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        let platform = Platform::current();
        let url = self
            .download_url(version, &platform)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("No download URL for 7-Zip {} on {:?}", version, platform)
            })?;

        let install_path = ctx
            .paths
            .version_store_dir("7zip", version)
            .join(platform.as_str());

        ctx.installer
            .download_and_extract(&url, &install_path)
            .await?;

        let exe_path = install_path.join(Self::exe_name(&platform));
        Ok(InstallResult::success(
            install_path,
            exe_path,
            version.to_string(),
        ))
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // 7-Zip GitHub releases: https://github.com/ip7z/7zip/releases/download/24.09/
        // Normalize version: "24.09" -> "2409"
        let ver_compact = version.replace('.', "");

        let url = match (platform.os_name(), platform.arch.as_str()) {
            // Windows: use MSI installer (extracted via msiexec /a, no registry changes)
            ("windows", "x64") => Some(format!(
                "https://github.com/ip7z/7zip/releases/download/{}/7z{}-x64.msi",
                version, ver_compact
            )),
            ("windows", "x86") => Some(format!(
                "https://github.com/ip7z/7zip/releases/download/{}/7z{}.msi",
                version, ver_compact
            )),
            // macOS: universal binary tar.xz
            ("macos", _) => Some(format!(
                "https://github.com/ip7z/7zip/releases/download/{}/7z{}-mac.tar.xz",
                version, ver_compact
            )),
            // Linux: platform-specific tar.xz
            ("linux", "x64") => Some(format!(
                "https://github.com/ip7z/7zip/releases/download/{}/7z{}-linux-x64.tar.xz",
                version, ver_compact
            )),
            ("linux", "arm64") => Some(format!(
                "https://github.com/ip7z/7zip/releases/download/{}/7z{}-linux-arm64.tar.xz",
                version, ver_compact
            )),
            _ => None,
        };

        Ok(url)
    }

    /// Prepare execution by finding 7z in PATH or known system locations
    async fn prepare_execution(
        &self,
        _version: &str,
        _ctx: &ExecutionContext,
    ) -> Result<ExecutionPrep> {
        let platform = Platform::current();
        let exe = Self::exe_name(&platform);

        // Try PATH first
        if let Ok(path) = which::which(exe) {
            return Ok(ExecutionPrep {
                executable_override: Some(path),
                proxy_ready: true,
                message: Some("Using system 7z".to_string()),
                ..Default::default()
            });
        }

        // Try known Windows paths
        #[cfg(windows)]
        {
            let candidates = [
                r"C:\Program Files\7-Zip\7z.exe",
                r"C:\Program Files (x86)\7-Zip\7z.exe",
            ];
            for candidate in &candidates {
                let path = PathBuf::from(candidate);
                if path.exists() {
                    return Ok(ExecutionPrep {
                        executable_override: Some(path),
                        proxy_ready: true,
                        message: Some("Using system 7z".to_string()),
                        ..Default::default()
                    });
                }
            }
        }

        Err(anyhow::anyhow!(
            "7z not found. Install it with:\n  \
            Windows: winget install 7zip.7zip\n  \
            macOS:   brew install sevenzip\n  \
            Linux:   sudo apt install p7zip-full"
        ))
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe = install_path.join(Self::exe_name(platform));
        if exe.exists() {
            VerificationResult::success(exe)
        } else if let Ok(system_path) = which::which("7z") {
            VerificationResult::success(system_path)
        } else {
            VerificationResult::failure(
                vec!["7z executable not found".to_string()],
                vec![
                    "Install via package manager: winget install 7zip.7zip".to_string(),
                    "Or download from https://www.7-zip.org/download.html".to_string(),
                ],
            )
        }
    }
}
