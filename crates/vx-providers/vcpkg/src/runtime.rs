//! vcpkg runtime implementation
//!
//! vcpkg is a C++ library manager that helps install C++ dependencies
//! for native Node.js modules and other projects.

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use vx_runtime::{
    Ecosystem, InstallResult, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo,
};

/// vcpkg runtime implementation
#[derive(Debug, Clone, Default)]
pub struct VcpkgRuntime {
    /// Default triplet for the current platform
    default_triplet: String,
}

impl VcpkgRuntime {
    /// Create a new vcpkg runtime
    pub fn new() -> Self {
        let default_triplet = Self::detect_default_triplet();
        Self { default_triplet }
    }

    /// Detect the default triplet based on the current platform
    fn detect_default_triplet() -> String {
        let os = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "osx"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else {
            "windows"
        };

        let arch = if cfg!(target_arch = "x86_64") {
            "x64"
        } else if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            "x64"
        };

        format!("{}-{}", arch, os)
    }

    /// Get the path to vcpkg executable
    fn get_vcpkg_exe(&self, install_path: &Path) -> PathBuf {
        if cfg!(windows) {
            install_path.join("vcpkg.exe")
        } else {
            install_path.join("vcpkg")
        }
    }

    /// Get the path to the bootstrap script
    fn get_bootstrap_script(&self, install_path: &Path) -> PathBuf {
        if cfg!(windows) {
            install_path.join("bootstrap-vcpkg.bat")
        } else {
            install_path.join("bootstrap-vcpkg.sh")
        }
    }

    /// Bootstrap vcpkg after cloning
    async fn bootstrap(&self, install_path: &Path) -> Result<()> {
        let bootstrap_script = self.get_bootstrap_script(install_path);

        if !bootstrap_script.exists() {
            return Err(anyhow::anyhow!(
                "Bootstrap script not found at {}",
                bootstrap_script.display()
            ));
        }

        info!("Bootstrapping vcpkg...");

        let status = if cfg!(windows) {
            std::process::Command::new(&bootstrap_script)
                .current_dir(install_path)
                .status()
        } else {
            std::process::Command::new("sh")
                .arg(&bootstrap_script)
                .current_dir(install_path)
                .status()
        }
        .context("Failed to run vcpkg bootstrap")?;

        if status.success() {
            info!("vcpkg bootstrapped successfully");
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "vcpkg bootstrap failed with status: {}",
                status
            ))
        }
    }

    /// Install a C++ package
    pub async fn install_package(&self, package: &str, install_path: &Path) -> Result<()> {
        let vcpkg_exe = self.get_vcpkg_exe(install_path);

        if !vcpkg_exe.exists() {
            return Err(anyhow::anyhow!(
                "vcpkg not found at {}. Run 'vx install vcpkg' first.",
                vcpkg_exe.display()
            ));
        }

        info!("Installing package: {} with vcpkg", package);

        let status = std::process::Command::new(&vcpkg_exe)
            .args(["install", package])
            .current_dir(install_path)
            .status()
            .context("Failed to run vcpkg install")?;

        if status.success() {
            info!("Package {} installed successfully", package);
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to install package {} with status: {}",
                package,
                status
            ))
        }
    }

    /// Check if git is available
    fn has_git() -> bool {
        which::which("git").is_ok()
    }
}

#[async_trait]
impl Runtime for VcpkgRuntime {
    fn name(&self) -> &str {
        "vcpkg"
    }

    fn description(&self) -> &str {
        "vcpkg - C++ library manager for Windows, Linux, and macOS. Manages C++ dependencies for native Node.js modules."
    }

    fn aliases(&self) -> &[&str] {
        &["vcpkg-cli"]
    }

    fn executable_name(&self) -> &str {
        "vcpkg"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Custom("cpp".to_string())
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://vcpkg.io/".to_string());
        meta.insert(
            "documentation".to_string(),
            "https://vcpkg.readthedocs.io/".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/microsoft/vcpkg".to_string(),
        );
        meta.insert("category".to_string(), "package-manager".to_string());
        meta.insert("language".to_string(), "C++".to_string());
        meta.insert("default_triplet".to_string(), self.default_triplet.clone());
        meta
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        // vcpkg supports all major platforms
        Platform::all_common()
    }

    fn executable_relative_path(&self, _version: &str, _platform: &Platform) -> String {
        if cfg!(windows) {
            "vcpkg.exe".to_string()
        } else {
            "vcpkg".to_string()
        }
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // vcpkg doesn't have traditional version releases
        // It's continuously updated via git
        // Return a single "latest" version to indicate the current state
        Ok(vec![
            VersionInfo::new("latest")
                .with_lts(true)
                .with_release_notes("vcpkg is continuously updated via git".to_string()),
        ])
    }

    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        // vcpkg is installed via git clone, not downloaded as an archive
        Ok(None)
    }

    /// Custom installation for vcpkg via git clone
    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        let install_path = ctx.paths.version_store_dir(self.name(), version);
        let platform = Platform::current();

        // Check platform support
        if !self.is_platform_supported(&platform) {
            return Err(anyhow::anyhow!("vcpkg is not supported on this platform"));
        }

        // Check if already installed
        if install_path.exists() {
            let vcpkg_exe = self.get_vcpkg_exe(&install_path);
            if vcpkg_exe.exists() {
                debug!("vcpkg already installed at {}", install_path.display());
                return Ok(InstallResult::already_installed(
                    install_path,
                    vcpkg_exe,
                    "latest".to_string(),
                ));
            }
        }

        // Check for git
        if !Self::has_git() {
            return Err(anyhow::anyhow!(
                "git is required to install vcpkg. Please install git first: vx install git"
            ));
        }

        info!("Installing vcpkg via git clone...");

        // Clone vcpkg repository
        let status = std::process::Command::new("git")
            .args([
                "clone",
                "https://github.com/microsoft/vcpkg.git",
                install_path.to_str().unwrap(),
            ])
            .status()
            .context("Failed to clone vcpkg repository")?;

        if !status.success() {
            return Err(anyhow::anyhow!(
                "Failed to clone vcpkg with status: {}",
                status
            ));
        }

        // Bootstrap vcpkg
        self.bootstrap(&install_path).await?;

        let vcpkg_exe = self.get_vcpkg_exe(&install_path);

        info!("vcpkg installed successfully at {}", install_path.display());

        Ok(InstallResult::success(
            install_path,
            vcpkg_exe,
            "latest".to_string(),
        ))
    }

    /// Prepare environment variables for vcpkg
    ///
    /// Sets up:
    /// - VCPKG_ROOT: Path to vcpkg installation
    /// - CMAKE_TOOLCHAIN_FILE: Path to vcpkg.cmake
    /// - VCPKG_DEFAULT_TRIPLET: Default triplet for the platform
    async fn prepare_environment(
        &self,
        version: &str,
        ctx: &RuntimeContext,
    ) -> Result<HashMap<String, String>> {
        let mut env = HashMap::new();
        let install_path = ctx.paths.version_store_dir(self.name(), version);

        if !install_path.exists() {
            debug!("vcpkg not installed, skipping environment setup");
            return Ok(env);
        }

        // Set VCPKG_ROOT
        env.insert(
            "VCPKG_ROOT".to_string(),
            install_path.to_string_lossy().to_string(),
        );

        // Set CMAKE_TOOLCHAIN_FILE for CMake integration
        let toolchain_file = install_path
            .join("scripts")
            .join("buildsystems")
            .join("vcpkg.cmake");
        if toolchain_file.exists() {
            env.insert(
                "CMAKE_TOOLCHAIN_FILE".to_string(),
                toolchain_file.to_string_lossy().to_string(),
            );
        }

        // Set default triplet
        env.insert(
            "VCPKG_DEFAULT_TRIPLET".to_string(),
            self.default_triplet.clone(),
        );

        // Add vcpkg to PATH
        let current_path = std::env::var("PATH").unwrap_or_default();
        env.insert(
            "PATH".to_string(),
            format!("{};{}", install_path.to_string_lossy(), current_path),
        );

        // Set VX_VCPKG_INSTALLED_DIR for easy access to installed packages
        let installed_dir = install_path.join("installed").join(&self.default_triplet);
        if installed_dir.exists() {
            env.insert(
                "VX_VCPKG_INSTALLED_DIR".to_string(),
                installed_dir.to_string_lossy().to_string(),
            );

            // Add bin and lib paths for Windows
            let bin_dir = installed_dir.join("bin");
            let lib_dir = installed_dir.join("lib");
            let include_dir = installed_dir.join("include");

            if bin_dir.exists() {
                let path = env.get("PATH").map(|p| p.as_str()).unwrap_or(&current_path);
                env.insert(
                    "PATH".to_string(),
                    format!("{};{}", bin_dir.to_string_lossy(), path),
                );
            }

            if lib_dir.exists() {
                env.insert("LIB".to_string(), lib_dir.to_string_lossy().to_string());
            }

            if include_dir.exists() {
                env.insert(
                    "INCLUDE".to_string(),
                    include_dir.to_string_lossy().to_string(),
                );
            }
        }

        debug!("vcpkg environment prepared with {} variables", env.len());
        Ok(env)
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        _platform: &Platform,
    ) -> VerificationResult {
        let vcpkg_exe = self.get_vcpkg_exe(install_path);

        if vcpkg_exe.exists() {
            VerificationResult::success(vcpkg_exe)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "vcpkg executable not found at {}",
                    vcpkg_exe.display()
                )],
                vec![
                    "Try reinstalling: vx install vcpkg".to_string(),
                    "Ensure git is installed and accessible".to_string(),
                ],
            )
        }
    }
}

/// Common C++ packages for native Node.js modules
pub mod native_packages {
    /// winpty - Required by node-pty for Windows terminal emulation
    pub const WINPTY: &str = "winpty";

    /// sqlite3 - SQLite database
    pub const SQLITE3: &str = "sqlite3";

    /// openssl - OpenSSL library
    pub const OPENSSL: &str = "openssl";

    /// libpng - PNG library
    pub const LIBPNG: &str = "libpng";

    /// libjpeg - JPEG library
    pub const LIBJPEG: &str = "libjpeg-turbo";

    /// zstd - Zstandard compression
    pub const ZSTD: &str = "zstd";
}
