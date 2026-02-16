//! vcpkg runtime implementation
//!
//! vcpkg-tool is the standalone binary for the vcpkg C++ package manager.
//! It is downloaded from https://github.com/microsoft/vcpkg-tool/releases
//! as a pre-built binary for each platform.
//!
//! # Version Format
//!
//! vcpkg-tool uses date-based release tags (e.g., "2025-12-16").
//! For semver compatibility, we convert these to dot-separated format:
//! - GitHub tag: "2025-12-16" → internal version: "2025.12.16"
//! - When constructing download URLs, we convert back: "2025.12.16" → "2025-12-16"
//!
//! # Storage Architecture
//! - Installation: `~/.vx/store/vcpkg/<version>/` (e.g., `~/.vx/store/vcpkg/2025.12.16/`)
//! - Downloads cache: `~/.vx/cache/vcpkg/downloads/`
//! - Binary cache: `~/.vx/cache/vcpkg/archives/`

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use vx_runtime::{
    Ecosystem, InstallResult, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo,
};

/// The vcpkg registry repository URL.
/// vcpkg needs the full registry (ports, scripts, triplets, versions)
/// to function properly. We shallow-clone this at install time.
///
/// NOTE: vcpkg requires `git` to be available on PATH for the bootstrap clone.
/// The provider.toml declares `git` as a hard dependency.
const VCPKG_REGISTRY_REPO: &str = "https://github.com/microsoft/vcpkg.git";

/// Directories excluded during sparse checkout to save disk space.
/// These are not needed for vcpkg to function.
const SPARSE_CHECKOUT_EXCLUDES: &[&str] = &[
    "docs",
    ".github",
    "NOTICE.txt",
    "CONTRIBUTING.md",
    "README.md",
    "README_zh_CN.md",
    "LICENSE.txt",
    "CHANGELOG.md",
];

/// Convert a date-based tag (e.g., "2025-12-16") to a semver-compatible version ("2025.12.16")
fn tag_to_version(tag: &str) -> String {
    tag.replace('-', ".")
}

/// Convert a semver-compatible version (e.g., "2025.12.16") back to a date tag ("2025-12-16")
fn version_to_tag(version: &str) -> String {
    // Only convert the first two dots (YYYY.MM.DD -> YYYY-MM-DD)
    let parts: Vec<&str> = version.splitn(3, '.').collect();
    if parts.len() == 3 {
        format!("{}-{}-{}", parts[0], parts[1], parts[2])
    } else {
        version.replace('.', "-")
    }
}

/// Get the platform-specific asset name for vcpkg-tool binary
fn platform_asset_name(platform: &Platform) -> Option<&'static str> {
    use vx_runtime::{Arch, Os};

    match (&platform.os, &platform.arch) {
        (Os::Windows, Arch::X86_64) => Some("vcpkg.exe"),
        (Os::Windows, Arch::Aarch64) => Some("vcpkg-arm64.exe"),
        (Os::MacOS, _) => Some("vcpkg-macos"), // Universal binary
        (Os::Linux, Arch::X86_64) => Some("vcpkg-glibc"),
        (Os::Linux, Arch::Aarch64) => Some("vcpkg-glibc-arm64"),
        _ => None,
    }
}

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

    /// Get the expected executable name for the current platform
    fn executable_name_for_platform() -> &'static str {
        if cfg!(windows) {
            "vcpkg.exe"
        } else {
            "vcpkg"
        }
    }

    /// Get the vcpkg downloads cache directory within vx cache
    fn get_downloads_cache_dir(ctx: &RuntimeContext) -> PathBuf {
        ctx.paths.cache_dir().join("vcpkg").join("downloads")
    }

    /// Get the vcpkg binary cache directory within vx cache
    fn get_binary_cache_dir(ctx: &RuntimeContext) -> PathBuf {
        ctx.paths.cache_dir().join("vcpkg").join("archives")
    }

    /// Ensure vx cache directories exist for vcpkg
    fn ensure_cache_dirs(ctx: &RuntimeContext) -> Result<()> {
        let downloads_dir = Self::get_downloads_cache_dir(ctx);
        let binary_dir = Self::get_binary_cache_dir(ctx);

        std::fs::create_dir_all(&downloads_dir).with_context(|| {
            format!(
                "Failed to create vcpkg downloads cache: {}",
                downloads_dir.display()
            )
        })?;

        std::fs::create_dir_all(&binary_dir).with_context(|| {
            format!(
                "Failed to create vcpkg binary cache: {}",
                binary_dir.display()
            )
        })?;

        Ok(())
    }

    /// Bootstrap the vcpkg root directory by shallow-cloning the vcpkg registry.
    ///
    /// vcpkg-tool requires a full registry layout to function properly:
    /// - `.vcpkg-root` marker file
    /// - `triplets/` directory with platform triplet cmake files
    /// - `scripts/` directory with cmake scripts and tool definitions
    /// - `ports/` directory with port definitions
    /// - `versions/` directory with version database
    ///
    /// The most reliable way to get all of these is to shallow-clone the
    /// microsoft/vcpkg repository directly into the install path,
    /// then place our downloaded vcpkg-tool binary into it.
    ///
    /// This requires `git` to be available on the system PATH.
    fn bootstrap_vcpkg_root(install_path: &Path, exe_name: &str) -> Result<()> {
        info!("Bootstrapping vcpkg registry at {}", install_path.display());

        // Save the downloaded binary before cloning
        let exe_path = install_path.join(exe_name);
        let parent = install_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Install path has no parent directory"))?;
        let temp_exe = parent.join(format!("vcpkg_binary_{}.tmp", exe_name));

        if exe_path.exists() {
            std::fs::rename(&exe_path, &temp_exe)
                .context("Failed to save vcpkg binary to temp location")?;
        }

        // Remove the install directory entirely so git clone can create it fresh
        std::fs::remove_dir_all(install_path).ok();

        // Shallow clone with sparse checkout to exclude unnecessary files (docs, .github, etc.)
        // This reduces disk usage by ~30-50MB.
        info!("Cloning vcpkg registry (shallow, sparse)...");
        let clone_result = std::process::Command::new("git")
            .args([
                "clone",
                "--depth=1",
                "--filter=blob:none",
                "--sparse",
                VCPKG_REGISTRY_REPO,
                &install_path.to_string_lossy(),
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output();

        match clone_result {
            Ok(output) if output.status.success() => {
                debug!("vcpkg registry sparse clone succeeded");

                // Configure sparse-checkout to exclude documentation and CI files.
                // By default sparse-checkout uses cone mode (include everything),
                // so we add negation patterns to exclude unwanted directories.
                let sparse_exclude = SPARSE_CHECKOUT_EXCLUDES
                    .iter()
                    .map(|d| format!("!/{}", d))
                    .collect::<Vec<_>>()
                    .join("\n");

                let sparse_content = format!("/*\n{}\n", sparse_exclude);

                // Write sparse-checkout file directly (more reliable than `git sparse-checkout set`)
                let sparse_file = install_path.join(".git").join("info").join("sparse-checkout");
                if let Some(parent) = sparse_file.parent() {
                    std::fs::create_dir_all(parent).ok();
                }
                if let Err(e) = std::fs::write(&sparse_file, &sparse_content) {
                    warn!("Failed to write sparse-checkout file: {}, continuing with full checkout", e);
                } else {
                    // Enable sparse-checkout in git config
                    let _ = std::process::Command::new("git")
                        .args(["-C", &install_path.to_string_lossy(), "config", "core.sparseCheckout", "true"])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status();

                    // Re-read the working tree with the sparse-checkout config
                    let checkout_result = std::process::Command::new("git")
                        .args(["-C", &install_path.to_string_lossy(), "checkout"])
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .output();

                    if let Ok(output) = checkout_result {
                        if output.status.success() {
                            debug!("Sparse checkout applied, excluded: {:?}", SPARSE_CHECKOUT_EXCLUDES);
                        } else {
                            warn!("Sparse checkout failed, keeping full clone");
                        }
                    }
                }
            }
            Ok(output) => {
                // Sparse clone failed — fall back to simple shallow clone
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("Sparse clone failed ({}), trying full shallow clone", stderr.trim());

                let fallback = std::process::Command::new("git")
                    .args([
                        "clone",
                        "--depth=1",
                        VCPKG_REGISTRY_REPO,
                        &install_path.to_string_lossy(),
                    ])
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .output();

                match fallback {
                    Ok(out) if out.status.success() => {
                        debug!("Full shallow clone succeeded (fallback)");
                    }
                    Ok(out) => {
                        let fb_stderr = String::from_utf8_lossy(&out.stderr);
                        std::fs::create_dir_all(install_path).ok();
                        if temp_exe.exists() {
                            std::fs::rename(&temp_exe, &exe_path).ok();
                        }
                        return Err(anyhow::anyhow!(
                            "Failed to clone vcpkg registry (exit code: {:?}): {}\n\
                             Ensure 'git' is installed and you have internet access.",
                            out.status.code(),
                            fb_stderr.trim()
                        ));
                    }
                    Err(e) => {
                        std::fs::create_dir_all(install_path).ok();
                        if temp_exe.exists() {
                            std::fs::rename(&temp_exe, &exe_path).ok();
                        }
                        return Err(anyhow::anyhow!(
                            "Failed to execute git: {}.\n\
                             vcpkg requires git to clone its package registry.\n\
                             Please install git and ensure it is on your PATH.",
                            e
                        ));
                    }
                }
            }
            Err(e) => {
                // git not found
                std::fs::create_dir_all(install_path).ok();
                if temp_exe.exists() {
                    std::fs::rename(&temp_exe, &exe_path).ok();
                }
                return Err(anyhow::anyhow!(
                    "Failed to execute git: {}.\n\
                     vcpkg requires git to clone its package registry.\n\
                     Please install git and ensure it is on your PATH.",
                    e
                ));
            }
        }

        // Place vcpkg-tool binary into the cloned directory
        // This replaces any bootstrap-built binary
        if temp_exe.exists() {
            if exe_path.exists() {
                std::fs::remove_file(&exe_path).ok();
            }
            std::fs::rename(&temp_exe, &exe_path)
                .context("Failed to place vcpkg-tool binary into registry directory")?;

            // Make executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = std::fs::Permissions::from_mode(0o755);
                std::fs::set_permissions(&exe_path, perms).ok();
            }
        }

        info!("vcpkg registry bootstrapped successfully");
        Ok(())
    }
}

#[async_trait]
impl Runtime for VcpkgRuntime {
    fn name(&self) -> &str {
        "vcpkg"
    }

    fn description(&self) -> &str {
        "vcpkg - C++ library manager for Windows, Linux, and macOS (standalone binary from vcpkg-tool)"
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
            "https://learn.microsoft.com/en-us/vcpkg/".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/microsoft/vcpkg-tool".to_string(),
        );
        meta.insert("category".to_string(), "package-manager".to_string());
        meta.insert("language".to_string(), "C++".to_string());
        meta.insert("default_triplet".to_string(), self.default_triplet.clone());
        meta
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        Platform::all_common()
    }

    fn executable_relative_path(&self, _version: &str, _platform: &Platform) -> String {
        Self::executable_name_for_platform().to_string()
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch releases from GitHub API for microsoft/vcpkg-tool
        let url = "https://api.github.com/repos/microsoft/vcpkg-tool/releases?per_page=30";

        let response = ctx
            .http
            .get_json_value(url)
            .await
            .context("Failed to fetch vcpkg-tool releases from GitHub")?;

        let mut versions = Vec::new();

        if let Some(releases) = response.as_array() {
            for release in releases {
                let is_draft = release.get("draft").and_then(|v| v.as_bool()).unwrap_or(false);
                let is_prerelease = release.get("prerelease").and_then(|v| v.as_bool()).unwrap_or(false);

                if is_draft {
                    continue;
                }

                if let Some(tag_name) = release.get("tag_name").and_then(|v| v.as_str()) {
                    // Convert date-based tag (e.g., "2025-12-16") to semver-compatible ("2025.12.16")
                    let version_str = tag_to_version(tag_name);

                    let mut version_info = VersionInfo::new(&version_str);
                    if is_prerelease {
                        version_info.prerelease = true;
                    }

                    // The latest non-prerelease is considered "LTS" (stable)
                    if !is_prerelease && versions.iter().all(|v: &VersionInfo| v.prerelease || !v.lts) {
                        version_info.lts = true;
                    }

                    if let Some(notes) = release.get("body").and_then(|v| v.as_str()) {
                        version_info = version_info.with_release_notes(notes.to_string());
                    }

                    versions.push(version_info);
                }
            }
        }

        if versions.is_empty() {
            // Fallback: return a known recent version
            warn!("Failed to fetch vcpkg-tool versions from GitHub, using fallback");
            versions.push(
                VersionInfo::new("2025.12.16")
                    .with_lts(true)
                    .with_release_notes("Fallback version".to_string()),
            );
        }

        debug!("Fetched {} vcpkg-tool versions", versions.len());
        Ok(versions)
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        let asset_name = match platform_asset_name(platform) {
            Some(name) => name,
            None => return Ok(None),
        };

        // Convert version back to date-based tag for the download URL
        let tag = version_to_tag(version);

        let url = format!(
            "https://github.com/microsoft/vcpkg-tool/releases/download/{}/{}",
            tag, asset_name
        );

        debug!("vcpkg download URL: {} (version={}, tag={})", url, version, tag);
        Ok(Some(url))
    }

    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        let install_path = ctx.paths.version_store_dir(self.name(), version);
        let platform = Platform::current();

        // Check platform support
        if !self.is_platform_supported(&platform) {
            return Err(anyhow::anyhow!("vcpkg is not supported on this platform"));
        }

        let exe_name = Self::executable_name_for_platform();
        let exe_path = install_path.join(exe_name);

        // Check if already installed
        if exe_path.exists() {
            debug!("vcpkg already installed at {}", install_path.display());
            return Ok(InstallResult::already_installed(
                install_path,
                exe_path,
                version.to_string(),
            ));
        }

        // Create install directory
        std::fs::create_dir_all(&install_path).with_context(|| {
            format!(
                "Failed to create vcpkg install directory: {}",
                install_path.display()
            )
        })?;

        // Ensure vx cache directories exist for vcpkg
        Self::ensure_cache_dirs(ctx)?;

        // Get download URL
        let url = self
            .download_url(version, &platform)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No vcpkg-tool binary available for platform: {:?}/{:?}",
                    platform.os,
                    platform.arch
                )
            })?;

        info!("Downloading vcpkg-tool from {}", url);

        // Download the binary directly
        let download_path = install_path.join(exe_name);
        ctx.http
            .download(&url, &download_path)
            .await
            .with_context(|| format!("Failed to download vcpkg-tool from {}", url))?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o755);
            std::fs::set_permissions(&download_path, perms)
                .context("Failed to set executable permissions on vcpkg binary")?;
        }

        // Bootstrap vcpkg root by shallow-cloning the vcpkg registry
        // This provides triplets/, scripts/, ports/, versions/ etc.
        // Without these, vcpkg commands like `install` will fail with "Invalid triplet"
        Self::bootstrap_vcpkg_root(&install_path, exe_name)?;

        info!("vcpkg-tool installed successfully at {}", install_path.display());
        info!("Using vx cache directories:");
        info!("  Downloads: {}", Self::get_downloads_cache_dir(ctx).display());
        info!("  Archives: {}", Self::get_binary_cache_dir(ctx).display());

        Ok(InstallResult::success(
            install_path,
            download_path,
            version.to_string(),
        ))
    }

    /// Prepare environment variables for vcpkg
    ///
    /// Sets up:
    /// - VCPKG_ROOT: Path to vcpkg installation directory
    /// - VCPKG_DOWNLOADS: Downloads cache directory (vx managed)
    /// - VCPKG_DEFAULT_BINARY_CACHE: Binary cache directory (vx managed)
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

        // Ensure cache directories exist
        if let Err(e) = Self::ensure_cache_dirs(ctx) {
            warn!("Failed to ensure vcpkg cache directories: {}", e);
        }

        // Set VCPKG_ROOT to the install directory
        env.insert(
            "VCPKG_ROOT".to_string(),
            install_path.to_string_lossy().to_string(),
        );

        // Set vx-managed cache directories
        env.insert(
            "VCPKG_DOWNLOADS".to_string(),
            Self::get_downloads_cache_dir(ctx)
                .to_string_lossy()
                .to_string(),
        );
        env.insert(
            "VCPKG_DEFAULT_BINARY_CACHE".to_string(),
            Self::get_binary_cache_dir(ctx)
                .to_string_lossy()
                .to_string(),
        );

        // Set default triplet
        env.insert(
            "VCPKG_DEFAULT_TRIPLET".to_string(),
            self.default_triplet.clone(),
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

        // Add vcpkg installed/<triplet>/include and /lib to INCLUDE/LIB for native builds
        let installed_dir = install_path.join("installed").join(&self.default_triplet);
        if installed_dir.exists() {
            let include_dir = installed_dir.join("include");
            if include_dir.exists() {
                let current = std::env::var("INCLUDE").unwrap_or_default();
                let sep = if cfg!(windows) { ";" } else { ":" };
                if current.is_empty() {
                    env.insert("INCLUDE".to_string(), include_dir.to_string_lossy().to_string());
                } else {
                    env.insert("INCLUDE".to_string(), format!("{}{}{}", include_dir.to_string_lossy(), sep, current));
                }
            }

            let lib_dir = installed_dir.join("lib");
            if lib_dir.exists() {
                let current = std::env::var("LIB").unwrap_or_default();
                let sep = if cfg!(windows) { ";" } else { ":" };
                if current.is_empty() {
                    env.insert("LIB".to_string(), lib_dir.to_string_lossy().to_string());
                } else {
                    env.insert("LIB".to_string(), format!("{}{}{}", lib_dir.to_string_lossy(), sep, current));
                }
            }
        }

        // Add vcpkg to PATH
        let current_path = std::env::var("PATH").unwrap_or_default();
        let path_separator = if cfg!(windows) { ";" } else { ":" };
        env.insert(
            "PATH".to_string(),
            format!(
                "{}{}{}",
                install_path.to_string_lossy(),
                path_separator,
                current_path
            ),
        );

        debug!("vcpkg environment prepared with {} variables", env.len());
        Ok(env)
    }

    /// Uninstall vcpkg
    ///
    /// Removes the vcpkg installation directory (including the git clone).
    /// The shared cache directories (~/.vx/cache/vcpkg/) are preserved
    /// since they may be shared across versions.
    async fn uninstall(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        let install_path = ctx.paths.version_store_dir(self.name(), version);

        if install_path.exists() {
            info!("Removing vcpkg installation at {}", install_path.display());

            // The install directory contains a full git clone, so remove_dir_all is needed
            std::fs::remove_dir_all(&install_path).with_context(|| {
                format!(
                    "Failed to remove vcpkg installation: {}",
                    install_path.display()
                )
            })?;

            info!("vcpkg {} uninstalled successfully", version);
            info!(
                "Note: Shared cache at ~/.vx/cache/vcpkg/ is preserved. \
                 Run 'rm -rf ~/.vx/cache/vcpkg' to clean it manually."
            );
        } else {
            debug!("vcpkg {} is not installed, nothing to uninstall", version);
        }

        Ok(())
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        _platform: &Platform,
    ) -> VerificationResult {
        let exe_name = Self::executable_name_for_platform();
        let vcpkg_exe = install_path.join(exe_name);

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
                    "Check your internet connection".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_to_version() {
        assert_eq!(tag_to_version("2025-12-16"), "2025.12.16");
        assert_eq!(tag_to_version("2025-01-01"), "2025.01.01");
    }

    #[test]
    fn test_version_to_tag() {
        assert_eq!(version_to_tag("2025.12.16"), "2025-12-16");
        assert_eq!(version_to_tag("2025.01.01"), "2025-01-01");
    }

    #[test]
    fn test_platform_asset_name() {
        use vx_runtime::{Arch, Os};

        let win_x64 = Platform::new(Os::Windows, Arch::X86_64);
        assert_eq!(platform_asset_name(&win_x64), Some("vcpkg.exe"));

        let win_arm64 = Platform::new(Os::Windows, Arch::Aarch64);
        assert_eq!(platform_asset_name(&win_arm64), Some("vcpkg-arm64.exe"));

        let macos = Platform::new(Os::MacOS, Arch::Aarch64);
        assert_eq!(platform_asset_name(&macos), Some("vcpkg-macos"));

        let linux_x64 = Platform::new(Os::Linux, Arch::X86_64);
        assert_eq!(platform_asset_name(&linux_x64), Some("vcpkg-glibc"));

        let linux_arm64 = Platform::new(Os::Linux, Arch::Aarch64);
        assert_eq!(platform_asset_name(&linux_arm64), Some("vcpkg-glibc-arm64"));
    }

    #[test]
    fn test_roundtrip_version_tag() {
        let tag = "2025-12-16";
        let version = tag_to_version(tag);
        let back = version_to_tag(&version);
        assert_eq!(back, tag);
    }
}
