//! x-cmd runtime implementation
//!
//! x-cmd is a compact and powerful command-line toolbox.
//!
//! Homepage: https://x-cmd.com
//! Repository: https://github.com/x-cmd/x-cmd

use crate::config::XCmdConfig;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, InstallResult, Platform, Runtime, RuntimeContext,
    VerificationResult, VersionInfo,
};

/// x-cmd runtime implementation
#[derive(Debug, Clone)]
pub struct XCmdRuntime;

impl XCmdRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl Default for XCmdRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for XCmdRuntime {
    fn name(&self) -> &str {
        "x-cmd"
    }

    fn description(&self) -> &str {
        "x-cmd - Compact and powerful command-line toolbox with AI integration"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn aliases(&self) -> &[&str] {
        &["xcmd", "x_cmd"]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://x-cmd.com".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/x-cmd/x-cmd".to_string(),
        );
        meta.insert("category".to_string(), "devtools".to_string());
        meta.insert("license".to_string(), "AGPL-3.0".to_string());
        meta
    }

    /// x-cmd is installed via shell script, no binary archive
    fn executable_relative_path(&self, _version: &str, _platform: &Platform) -> String {
        XCmdConfig::executable_name().to_string()
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        ctx.fetch_github_releases(
            "x-cmd",
            "x-cmd",
            "x-cmd",
            GitHubReleaseOptions::new().strip_v_prefix(true),
        )
        .await
    }

    /// x-cmd doesn't have a direct download URL - it's installed via shell script
    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        // x-cmd is installed via script, not a direct download
        Ok(None)
    }

    /// Override install() for script-based installation.
    ///
    /// x-cmd cannot be installed via binary download. Instead:
    /// 1. Check if already installed on the system
    /// 2. If not, run the appropriate install script (curl for Unix, PowerShell for Windows)
    async fn install(&self, version: &str, _ctx: &RuntimeContext) -> Result<InstallResult> {
        let platform = Platform::current();

        // First check if x-cmd is already installed on the system
        if let Ok(path) = which::which(XCmdConfig::executable_name()) {
            debug!("x-cmd already installed at: {}", path.display());
            return Ok(InstallResult::system_installed(
                version.to_string(),
                Some(path),
            ));
        }

        // Check common search paths
        for search_path in XCmdConfig::search_paths(&platform) {
            let x_path = Path::new(search_path).join(XCmdConfig::executable_name());
            if x_path.exists() {
                debug!("x-cmd found at: {}", x_path.display());
                return Ok(InstallResult::system_installed(
                    version.to_string(),
                    Some(x_path),
                ));
            }
        }

        // Not installed - run the install script
        info!("Installing x-cmd via script...");

        let install_result = match platform.os {
            vx_runtime::Os::Windows => {
                // PowerShell: iex (irm https://get.x-cmd.com/ps1)
                let status = tokio::process::Command::new("powershell")
                    .args([
                        "-NoProfile",
                        "-ExecutionPolicy",
                        "Bypass",
                        "-Command",
                        &format!("iex (irm {})", XCmdConfig::install_script_url_windows()),
                    ])
                    .stdin(std::process::Stdio::inherit())
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit())
                    .status()
                    .await;

                match status {
                    Ok(s) if s.success() => Ok(()),
                    Ok(s) => Err(anyhow::anyhow!(
                        "x-cmd install script exited with code {}",
                        s.code().unwrap_or(-1)
                    )),
                    Err(e) => Err(anyhow::anyhow!("Failed to run PowerShell: {}", e)),
                }
            }
            _ => {
                // Unix: eval "$(curl -fsSL https://get.x-cmd.com)"
                let status = tokio::process::Command::new("bash")
                    .args([
                        "-c",
                        &format!(
                            "eval \"$(curl -fsSL {})\"",
                            XCmdConfig::install_script_url()
                        ),
                    ])
                    .stdin(std::process::Stdio::inherit())
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit())
                    .status()
                    .await;

                match status {
                    Ok(s) if s.success() => Ok(()),
                    Ok(s) => Err(anyhow::anyhow!(
                        "x-cmd install script exited with code {}",
                        s.code().unwrap_or(-1)
                    )),
                    Err(e) => Err(anyhow::anyhow!("Failed to run bash: {}", e)),
                }
            }
        };

        if let Err(e) = install_result {
            warn!("Script installation failed: {}", e);
            return Err(anyhow::anyhow!(
                "Failed to install x-cmd. Please install manually:\n\n  {}\n\nVisit https://x-cmd.com for more information.",
                XCmdConfig::install_command(&platform)
            ));
        }

        // Verify installation after script completed
        if let Ok(path) = which::which(XCmdConfig::executable_name()) {
            info!("x-cmd successfully installed at: {}", path.display());
            return Ok(InstallResult::system_installed(
                version.to_string(),
                Some(path),
            ));
        }

        // Check search paths again
        for search_path in XCmdConfig::search_paths(&platform) {
            let x_path = Path::new(search_path).join(XCmdConfig::executable_name());
            if x_path.exists() {
                info!("x-cmd successfully installed at: {}", x_path.display());
                return Ok(InstallResult::system_installed(
                    version.to_string(),
                    Some(x_path),
                ));
            }
        }

        Err(anyhow::anyhow!(
            "x-cmd install script completed but executable not found.\n\
             Please install manually:\n\n  {}\n\n\
             Visit https://x-cmd.com for more information.",
            XCmdConfig::install_command(&platform)
        ))
    }

    fn verify_installation(
        &self,
        _version: &str,
        _install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        // Check common installation paths
        for search_path in XCmdConfig::search_paths(platform) {
            let x_path = Path::new(search_path).join(XCmdConfig::executable_name());
            if x_path.exists() {
                return VerificationResult::success(x_path);
            }
        }

        // Also check if 'x' is available on PATH via which
        if let Ok(path) = which::which(XCmdConfig::executable_name()) {
            return VerificationResult::success(path);
        }

        VerificationResult::failure(
            vec!["x-cmd not found in common installation paths".to_string()],
            vec![
                format!(
                    "Install x-cmd: {}",
                    XCmdConfig::install_command(platform)
                ),
                "Visit https://x-cmd.com for more information".to_string(),
            ],
        )
    }

    /// Check if x-cmd is already installed on the system
    async fn is_installed(&self, _version: &str, _ctx: &RuntimeContext) -> Result<bool> {
        Ok(which::which(XCmdConfig::executable_name()).is_ok())
    }
}
