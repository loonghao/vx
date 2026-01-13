//! Azure CLI runtime implementation

use crate::config::AzCliUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VerificationResult,
    VersionInfo,
};

/// Azure CLI runtime
#[derive(Debug, Clone)]
pub struct AzCliRuntime;

impl AzCliRuntime {
    /// Create a new Azure CLI runtime
    pub fn new() -> Self {
        Self
    }

    /// Find a file recursively in a directory using a predicate (Windows helper)
    #[cfg(target_os = "windows")]
    fn find_file_recursive<F>(dir: &Path, predicate: F) -> Option<std::path::PathBuf>
    where
        F: Fn(&Path) -> bool + Copy,
    {
        use std::fs;

        if !dir.exists() {
            return None;
        }

        for entry in fs::read_dir(dir).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();

            if path.is_file() && predicate(&path) {
                return Some(path);
            } else if path.is_dir() {
                if let Some(found) = Self::find_file_recursive(&path, predicate) {
                    return Some(found);
                }
            }
        }

        None
    }

    /// Find an executable file by name recursively (Windows helper)
    #[cfg(target_os = "windows")]
    fn find_executable_recursive(dir: &Path, exe_name: &str) -> Option<std::path::PathBuf> {
        Self::find_file_recursive(dir, |p| p.file_name().is_some_and(|n| n == exe_name))
    }
}

impl Default for AzCliRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for AzCliRuntime {
    fn name(&self) -> &str {
        "az"
    }

    fn description(&self) -> &str {
        "Azure CLI - Microsoft Azure command-line interface"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Custom("cloud".to_string())
    }

    fn aliases(&self) -> &[&str] {
        &["azcli", "azure-cli", "azure"]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://docs.microsoft.com/cli/azure/".to_string(),
        );
        meta.insert("ecosystem".to_string(), "cloud".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/Azure/azure-cli".to_string(),
        );
        meta.insert("license".to_string(), "MIT".to_string());
        meta
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        Platform::all_common()
    }

    /// Azure CLI executable path varies by platform
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        use vx_runtime::Os;

        match &platform.os {
            // Linux/macOS: extracted to bin/az
            Os::Linux | Os::MacOS => "bin/az".to_string(),
            // Windows: after msi extraction
            Os::Windows => "Microsoft SDKs/Azure/CLI2/wbin/az.cmd".to_string(),
            _ => "az".to_string(),
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch from Azure/azure-cli GitHub releases
        // Azure CLI tags are in format "azure-cli-X.Y.Z"
        ctx.fetch_github_releases(
            "az",
            "Azure",
            "azure-cli",
            GitHubReleaseOptions::new()
                .strip_v_prefix(false)
                .tag_prefix("azure-cli-"),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(AzCliUrlBuilder::download_url(version, platform))
    }

    /// Custom post-extract for Azure CLI on Windows
    ///
    /// Uses msiexec to install MSI silently to a custom directory
    fn post_extract(&self, version: &str, install_path: &std::path::PathBuf) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;

            // Find the MSI file - it may be in root or in bin/ subdirectory
            let msi_filename = format!("azure-cli-{}-x64.msi", version);
            let arm64_filename = format!("azure-cli-{}-arm64.msi", version);

            // Check multiple possible locations
            let possible_paths = vec![
                install_path.join(&msi_filename),
                install_path.join(&arm64_filename),
                install_path.join("bin").join(&msi_filename),
                install_path.join("bin").join(&arm64_filename),
            ];

            let msi_file = possible_paths
                .iter()
                .find(|p| p.exists())
                .cloned()
                .or_else(|| {
                    // Recursively search for any .msi file
                    Self::find_file_recursive(install_path, |p| {
                        p.extension().map(|ext| ext == "msi").unwrap_or(false)
                    })
                })
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "MSI file not found in {}. Expected: {} or {}",
                        install_path.display(),
                        msi_filename,
                        arm64_filename
                    )
                })?;

            eprintln!("📦 Installing Azure CLI using msiexec...");
            eprintln!("   MSI file: {}", msi_file.display());
            eprintln!("   Target directory: {}", install_path.display());
            eprintln!("   This may take a moment...");

            // Use msiexec to install silently to a custom directory
            // /a = administrative install (extract files without system registration)
            // /qn = quiet, no UI
            let output = Command::new("msiexec.exe")
                .arg("/a")
                .arg(&msi_file)
                .arg("/qn")
                .arg(format!("TARGETDIR={}", install_path.display()))
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                return Err(anyhow::anyhow!(
                    "Failed to install Azure CLI via msiexec\nstdout: {}\nstderr: {}",
                    stdout,
                    stderr
                ));
            }

            // Clean up MSI file after extraction
            if msi_file.exists() {
                let _ = std::fs::remove_file(&msi_file);
            }

            eprintln!("✓ Azure CLI installed successfully");
        }

        #[cfg(not(target_os = "windows"))]
        {
            // On Linux/macOS, the tar.gz extracts directly
            let _ = version; // Suppress unused warning
        }

        Ok(())
    }

    #[allow(unused_variables)]
    fn verify_installation(
        &self,
        version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        #[cfg(target_os = "windows")]
        {
            // On Windows, after MSI installation, search for az.cmd in the install directory
            let possible_paths = vec![
                install_path
                    .join("Microsoft SDKs")
                    .join("Azure")
                    .join("CLI2")
                    .join("wbin")
                    .join("az.cmd"),
                install_path.join("wbin").join("az.cmd"),
                install_path.join("bin").join("az.cmd"),
                install_path.join("az.cmd"),
            ];

            for exe_path in possible_paths {
                if exe_path.exists() {
                    return VerificationResult::success(exe_path);
                }
            }

            // If not found, search recursively
            if let Some(found) = Self::find_executable_recursive(install_path, "az.cmd") {
                return VerificationResult::success(found);
            }

            VerificationResult::failure(
                vec![format!(
                    "Azure CLI executable not found in {}. Searched standard locations.",
                    install_path.display()
                )],
                vec![],
            )
        }

        #[cfg(not(target_os = "windows"))]
        {
            let exe_path = install_path.join(self.executable_relative_path(version, platform));
            if exe_path.exists() {
                VerificationResult::success(exe_path)
            } else {
                VerificationResult::failure(
                    vec![format!(
                        "Azure CLI executable not found at {}",
                        exe_path.display()
                    )],
                    vec![],
                )
            }
        }
    }
}
