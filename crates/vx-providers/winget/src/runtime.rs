//! Windows Package Manager runtime implementation

use crate::config::WingetConfig;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use vx_runtime::{
    Ecosystem, ExecutionContext, ExecutionPrep, GitHubReleaseOptions, InstallResult, Platform,
    Runtime, RuntimeContext, VerificationResult, VersionInfo,
};

/// Windows Package Manager runtime
#[derive(Debug, Clone)]
pub struct WingetRuntime;

impl WingetRuntime {
    /// Create a new winget runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for WingetRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for WingetRuntime {
    fn name(&self) -> &str {
        "winget"
    }

    fn description(&self) -> &str {
        "Windows Package Manager - Official package manager for Windows"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn aliases(&self) -> &[&str] {
        &["winget-cli"]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://learn.microsoft.com/windows/package-manager/".to_string(),
        );
        meta.insert("ecosystem".to_string(), "windows".to_string());
        meta.insert("category".to_string(), "package-manager".to_string());
        meta
    }

    /// winget only supports Windows
    fn supported_platforms(&self) -> Vec<Platform> {
        Platform::windows_only()
    }

    /// winget is typically installed via App Installer, not directly downloadable
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        WingetConfig::get_executable_name(platform).to_string()
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // winget releases are tagged like "v1.9.25200"
        ctx.fetch_github_releases(
            "winget",
            "microsoft",
            "winget-cli",
            GitHubReleaseOptions::new().tag_prefix("v"),
        )
        .await
    }

    /// winget can be installed from GitHub releases
    fn is_version_installable(&self, _version: &str) -> bool {
        true
    }

    /// Install winget using Add-AppxPackage
    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        if !self.is_platform_supported(&Platform::current()) {
            return Err(anyhow::anyhow!("winget is only supported on Windows"));
        }

        // Download the msixbundle from GitHub releases
        let download_url = format!(
            "https://github.com/microsoft/winget-cli/releases/download/v{}/Microsoft.DesktopAppInstaller_8wekyb3d8bbwe.msixbundle",
            version
        );

        let temp_dir = std::env::temp_dir().join(format!("vx-winget-{}", version));
        let bundle_path = temp_dir.join("winget.msixbundle");

        // Download the bundle
        ctx.http
            .download(&download_url, &bundle_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to download winget: {}", e))?;

        // Install using Add-AppxPackage
        let script = format!(
            r#"Add-AppxPackage -Path "{}" -ForceApplicationShutdown"#,
            bundle_path.display()
        );

        let output = Command::new("powershell")
            .args(["-Command", &script])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run PowerShell: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Failed to install winget via Add-AppxPackage: {}",
                stderr
            ));
        }

        // Find the installed winget path
        let exe_path = which::which("winget").ok();

        Ok(InstallResult::system_installed(
            format!("system (v{})", version),
            exe_path,
        ))
    }

    /// Prepare execution for winget using system installation
    async fn prepare_execution(
        &self,
        _version: &str,
        _ctx: &ExecutionContext,
    ) -> Result<ExecutionPrep> {
        // Try to find winget using 'where' command (most reliable on Windows)
        let output = Command::new("where").arg("winget").output();

        if let Ok(output) = output
            && output.status.success()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout.lines().next() {
                let path = PathBuf::from(line.trim());
                if path.exists() {
                    return Ok(ExecutionPrep {
                        executable_override: Some(path),
                        proxy_ready: true,
                        message: Some("Using system winget".to_string()),
                        ..Default::default()
                    });
                }
            }
        }

        Err(anyhow::anyhow!(
            "winget not found. Run 'vx install winget' to install from GitHub releases,\n\
            or install 'App Installer' from Microsoft Store"
        ))
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // Return the msixbundle download URL
        if !self.is_platform_supported(platform) {
            return Ok(None);
        }
        Ok(Some(format!(
            "https://github.com/microsoft/winget-cli/releases/download/v{}/Microsoft.DesktopAppInstaller_8wekyb3d8bbwe.msixbundle",
            version
        )))
    }

    fn verify_installation(
        &self,
        _version: &str,
        _install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        // winget is installed via system (App Installer), not in vx directory
        if !self.is_platform_supported(platform) {
            return VerificationResult::failure(
                vec!["winget is only supported on Windows".to_string()],
                vec!["Use a Windows 10/11 system".to_string()],
            );
        }

        // Check system paths for winget
        let system_paths = [
            r"C:\Users\Default\AppData\Local\Microsoft\WindowsApps\winget.exe",
            r"C:\Program Files\WindowsApps\Microsoft.DesktopAppInstaller_*\winget.exe",
        ];

        for path_pattern in &system_paths {
            // For glob patterns, just check if the base path exists
            let base_path = path_pattern.split('*').next().unwrap_or(path_pattern);
            if Path::new(base_path).exists() || std::fs::metadata(path_pattern).is_ok() {
                return VerificationResult::success(Path::new(path_pattern).to_path_buf());
            }
        }

        VerificationResult::failure(
            vec!["winget not found in system paths".to_string()],
            vec![
                "Install 'App Installer' from Microsoft Store".to_string(),
                "Or download from https://github.com/microsoft/winget-cli/releases".to_string(),
            ],
        )
    }
}
