//! AWS CLI runtime implementation

use crate::config::AwsCliUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Arch, Ecosystem, Os, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo,
};

/// AWS CLI runtime
#[derive(Debug, Clone)]
pub struct AwsCliRuntime;

impl AwsCliRuntime {
    /// Create a new AWS CLI runtime
    pub fn new() -> Self {
        Self
    }

    /// Find an executable file recursively in a directory (Windows helper)
    #[cfg(target_os = "windows")]
    fn find_executable_recursive(dir: &Path, exe_name: &str) -> Result<std::path::PathBuf> {
        use std::fs;

        if !dir.exists() {
            return Err(anyhow::anyhow!("Directory does not exist"));
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.file_name().is_some_and(|n| n == exe_name) {
                return Ok(path);
            } else if path.is_dir() {
                // Recursively search subdirectories (limit depth to 5)
                if let Ok(found) = Self::find_executable_recursive(&path, exe_name) {
                    return Ok(found);
                }
            }
        }

        Err(anyhow::anyhow!("Executable not found"))
    }
}

impl Default for AwsCliRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for AwsCliRuntime {
    fn name(&self) -> &str {
        "aws"
    }

    fn description(&self) -> &str {
        "AWS CLI v2 - Amazon Web Services command-line interface"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Custom("cloud".to_string())
    }

    fn aliases(&self) -> &[&str] {
        &["awscli", "aws-cli"]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://aws.amazon.com/cli/".to_string(),
        );
        meta.insert("ecosystem".to_string(), "cloud".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/aws/aws-cli".to_string(),
        );
        meta.insert("license".to_string(), "Apache-2.0".to_string());
        meta
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        vec![
            Platform::new(Os::Linux, Arch::X86_64),
            Platform::new(Os::Linux, Arch::Aarch64),
            Platform::new(Os::MacOS, Arch::X86_64),
            Platform::new(Os::MacOS, Arch::Aarch64),
            Platform::new(Os::Windows, Arch::X86_64),
        ]
    }

    /// AWS CLI executable path varies by platform
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        use vx_runtime::Os;

        match &platform.os {
            // Linux: executable is in aws/dist/ after extraction
            Os::Linux => "aws/dist/aws".to_string(),
            // macOS: executable is in aws-cli/ after extraction
            Os::MacOS => "aws-cli/aws".to_string(),
            // Windows: after MSI installation, check multiple possible locations
            // AWS CLI MSI may install to Program Files/Amazon/AWSCLIV2 or custom TARGETDIR
            Os::Windows => {
                // Try common locations
                // 1. Custom install dir: Amazon/AWSCLIV2/aws.exe
                // 2. Program Files: might be in different structure
                // We'll use the verification to find the actual location
                "Amazon/AWSCLIV2/aws.exe".to_string()
            }
            _ => "aws".to_string(),
        }
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // AWS CLI uses a single "latest" version on their download site
        // GitHub releases are for development versions only
        // We provide a hardcoded list of known stable versions
        let versions = vec![
            "2.32.25", "2.32.0", "2.31.0", "2.30.0", "2.29.0", "2.28.0", "2.27.0", "2.26.0",
            "2.25.0", "2.24.0", "2.23.0", "2.22.0", "2.21.0", "2.20.0", "2.19.0", "2.18.0",
            "2.17.0", "2.16.0", "2.15.0", "latest",
        ];

        Ok(versions.into_iter().map(VersionInfo::new).collect())
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(AwsCliUrlBuilder::download_url(version, platform))
    }

    /// Custom post-extract for AWS CLI
    ///
    /// This hook runs platform-specific installation procedures BEFORE verification:
    ///
    /// **Windows**: Uses msiexec to install MSI silently to a custom directory
    /// **Linux/macOS**: Runs the official `./aws/install` script with custom install directory
    fn post_extract(&self, version: &str, install_path: &std::path::PathBuf) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;

            // Determine MSI filename based on version
            let msi_filename = if version == "latest" {
                "AWSCLIV2.msi".to_string()
            } else {
                format!("AWSCLIV2-{}.msi", version)
            };

            // MSI file might be in root or in a subdirectory after download
            let possible_msi_paths = vec![
                install_path.join(&msi_filename),
                install_path.join("bin").join(&msi_filename),
            ];

            let msi_file = possible_msi_paths
                .iter()
                .find(|p| p.exists())
                .cloned()
                .ok_or_else(|| {
                    anyhow::anyhow!("MSI file not found. Searched: {:?}", possible_msi_paths)
                })?;

            eprintln!("📦 Installing AWS CLI using msiexec...");
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
                    "Failed to install AWS CLI via msiexec\nstdout: {}\nstderr: {}",
                    stdout,
                    stderr
                ));
            }

            // Clean up MSI file after extraction
            if msi_file.exists() {
                let _ = std::fs::remove_file(&msi_file);
            }

            eprintln!("✓ AWS CLI installed successfully");
        }

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            use std::process::Command;

            // AWS CLI zip contains an 'aws' directory with an 'install' script
            let aws_dir = install_path.join("aws");
            let install_script = aws_dir.join("install");

            if !install_script.exists() {
                return Err(anyhow::anyhow!(
                    "AWS CLI install script not found at {}",
                    install_script.display()
                ));
            }

            // Make install script executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&install_script)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&install_script, perms)?;
            }

            eprintln!("📦 Installing AWS CLI using official install script...");
            eprintln!("   Install directory: {}", install_path.display());
            eprintln!("   This may take a moment...");

            // Run: ./aws/install --install-dir /path/to/install --bin-dir /path/to/bin
            // We install to a custom directory to avoid requiring root privileges
            let bin_dir = install_path.join("bin");
            let cli_dir = install_path.join("aws-cli");

            // Create directories
            std::fs::create_dir_all(&bin_dir)?;
            std::fs::create_dir_all(&cli_dir)?;

            // Run install script without sudo (install to user directory)
            let output = Command::new(&install_script)
                .arg("--install-dir")
                .arg(&cli_dir)
                .arg("--bin-dir")
                .arg(&bin_dir)
                .current_dir(install_path)
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                return Err(anyhow::anyhow!(
                    "Failed to install AWS CLI\nstdout: {}\nstderr: {}",
                    stdout,
                    stderr
                ));
            }

            eprintln!("✓ AWS CLI installed successfully");
        }

        let _ = version; // Suppress unused warning on non-target platforms
        Ok(())
    }

    /// Called after successful installation (kept for backward compatibility)
    async fn post_install(&self, _version: &str, _ctx: &RuntimeContext) -> Result<()> {
        // Installation is now handled in post_extract
        Ok(())
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        _platform: &Platform,
    ) -> VerificationResult {
        #[cfg(target_os = "windows")]
        {
            // On Windows, after MSI installation, search for aws.exe in the install directory
            // AWS CLI MSI might install to various subdirectories
            let possible_paths = vec![
                install_path.join("Amazon").join("AWSCLIV2").join("aws.exe"),
                install_path.join("AWS CLI").join("bin").join("aws.exe"),
                install_path.join("bin").join("aws.exe"),
                install_path.join("aws.exe"),
            ];

            for exe_path in possible_paths {
                if exe_path.exists() {
                    return VerificationResult::success(exe_path);
                }
            }

            // If not found, search recursively
            if let Ok(found) = Self::find_executable_recursive(install_path, "aws.exe") {
                return VerificationResult::success(found);
            }

            VerificationResult::failure(
                vec![format!(
                    "AWS CLI executable not found in {}. Searched standard locations.",
                    install_path.display()
                )],
                vec![],
            )
        }

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            // On Linux/macOS, after running install script, check multiple possible locations
            let possible_paths = vec![
                // After install script: aws-cli/aws (symlink or binary)
                install_path.join("aws-cli").join("aws"),
                // In bin directory
                install_path.join("bin").join("aws"),
                // Fallback: in aws/dist/ (before install script runs)
                install_path.join("aws").join("dist").join("aws"),
            ];

            for exe_path in &possible_paths {
                if exe_path.exists() {
                    return VerificationResult::success(exe_path.clone());
                }
            }

            VerificationResult::failure(
                vec![format!(
                    "AWS CLI executable not found in {}. Expected at: {}",
                    install_path.display(),
                    possible_paths
                        .iter()
                        .map(|p| p.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )],
                vec![
                    "The AWS CLI install script may have failed.".to_string(),
                    "Try running the install script manually:".to_string(),
                    format!("  cd {} && sudo ./aws/install", install_path.display()),
                ],
            )
        }
    }
}
