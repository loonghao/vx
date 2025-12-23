//! Rez runtime implementation
//!
//! Rez is a cross-platform package manager with a difference.
//! It provides a deterministic environment for software development.
//!
//! This runtime installs Rez from PyPI as an isolated tool.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use tracing::debug;
use vx_runtime::{
    compare_semver, Ecosystem, InstallMethod, InstallResult, PackageRuntime, PathProvider,
    Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo,
};

/// Rez runtime implementation
///
/// Rez is installed as a pip package in an isolated virtual environment,
/// similar to how pipx works.
#[derive(Debug, Clone, Default)]
pub struct RezRuntime;

impl RezRuntime {
    /// Create a new Rez runtime
    pub fn new() -> Self {
        Self
    }

    /// pip package name for rez
    const PACKAGE_NAME: &'static str = "rez";

    /// Create the production install marker file to disable pip warning
    ///
    /// Rez checks for `.rez_production_install` file in the bin directory.
    /// If this file doesn't exist, it shows a warning about pip-based installation.
    /// Since vx provides isolated environments, we create this marker to suppress the warning.
    ///
    /// See: https://rez.readthedocs.io/en/stable/installation.html#why-not-pip-for-production
    fn create_production_marker(&self, bin_dir: &Path) -> Result<()> {
        let marker_file = bin_dir.join(".rez_production_install");

        if marker_file.exists() {
            debug!(
                "Production marker already exists: {}",
                marker_file.display()
            );
            return Ok(());
        }

        debug!("Creating rez production marker: {}", marker_file.display());

        // Create the marker file with a comment explaining why it exists
        let content = "# Created by vx to mark this as a production rez installation.\n\
                       # This suppresses the pip-based installation warning.\n\
                       # See: https://rez.readthedocs.io/en/stable/installation.html#why-not-pip-for-production\n";

        std::fs::write(&marker_file, content)?;
        debug!("Successfully created rez production marker");

        Ok(())
    }

    /// Fix SyntaxWarning in rez's vendored memcache library
    ///
    /// The memcache.py file contains an invalid escape sequence `\ ` (backslash-space)
    /// in a docstring, which triggers a SyntaxWarning in Python 3.12+.
    /// This method patches the file to fix the warning.
    fn fix_memcache_syntax_warning(&self, install_dir: &Path) -> Result<()> {
        // Path to the memcache.py file in the rez vendor directory
        let memcache_path = if cfg!(windows) {
            install_dir.join("venv/Lib/site-packages/rez/vendor/memcache/memcache.py")
        } else {
            // On Unix, we need to find the python version directory
            let lib_dir = install_dir.join("venv/lib");
            if let Ok(entries) = std::fs::read_dir(&lib_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with("python3.") {
                        let path = entry
                            .path()
                            .join("site-packages/rez/vendor/memcache/memcache.py");
                        if path.exists() {
                            return self.patch_memcache_file(&path);
                        }
                    }
                }
            }
            return Ok(());
        };

        if memcache_path.exists() {
            self.patch_memcache_file(&memcache_path)?;
        }

        Ok(())
    }

    /// Patch the memcache.py file to fix the invalid escape sequence
    fn patch_memcache_file(&self, path: &Path) -> Result<()> {
        debug!("Patching memcache.py at: {}", path.display());

        let content = std::fs::read_to_string(path)?;

        // The problematic line contains `debuglog,\ set` - the backslash-space is invalid
        // We need to replace it with just a space (removing the backslash)
        if content.contains("debuglog,\\ set") {
            let patched = content.replace("debuglog,\\ set", "debuglog, set");
            std::fs::write(path, patched)?;
            debug!("Successfully patched memcache.py");
        } else {
            debug!("memcache.py doesn't contain the problematic pattern, skipping");
        }

        Ok(())
    }
}

#[async_trait]
impl Runtime for RezRuntime {
    fn name(&self) -> &str {
        "rez"
    }

    fn description(&self) -> &str {
        "Rez - Cross-platform package manager (installed via pip)"
    }

    fn aliases(&self) -> &[&str] {
        &["rez-env", "rez-build", "rez-release"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Python
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://rez.readthedocs.io/".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://rez.readthedocs.io/en/stable/".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/AcademySoftwareFoundation/rez".to_string(),
        );
        meta.insert("category".to_string(), "package-manager".to_string());
        meta.insert("install_method".to_string(), "pip".to_string());
        meta.insert("pip_package".to_string(), Self::PACKAGE_NAME.to_string());
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // For pip packages, the executable is in the venv bin directory
        if platform.is_windows() {
            format!("venv/Scripts/{}", platform.exe_name("rez"))
        } else {
            format!("venv/bin/{}", platform.exe_name("rez"))
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch versions from PyPI
        self.fetch_package_versions(ctx).await
    }

    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        // pip packages don't have direct download URLs
        Ok(None)
    }

    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        // Use pip package installation
        let result = self.install_package(version, ctx).await?;

        // Create production marker to disable pip warning
        let bin_dir = ctx.paths.pip_tool_bin_dir(Self::PACKAGE_NAME, version);
        if let Err(e) = self.create_production_marker(&bin_dir) {
            debug!("Failed to create rez production marker: {}", e);
            // Don't fail the installation if marker creation fails
        }

        // Fix SyntaxWarning in vendored memcache library
        let install_dir = ctx.paths.pip_tool_version_dir(Self::PACKAGE_NAME, version);
        if let Err(e) = self.fix_memcache_syntax_warning(&install_dir) {
            debug!("Failed to fix memcache syntax warning: {}", e);
            // Don't fail the installation if patching fails
        }

        Ok(result)
    }

    async fn is_installed(&self, version: &str, ctx: &RuntimeContext) -> Result<bool> {
        let bin_dir = ctx.paths.pip_tool_bin_dir(Self::PACKAGE_NAME, version);
        let exe_name = if cfg!(windows) { "rez.exe" } else { "rez" };
        let exe_path = bin_dir.join(exe_name);
        Ok(exe_path.exists())
    }

    async fn installed_versions(&self, ctx: &RuntimeContext) -> Result<Vec<String>> {
        let tool_dir = ctx.paths.pip_tool_dir(Self::PACKAGE_NAME);
        if !ctx.fs.exists(&tool_dir) {
            return Ok(vec![]);
        }

        let entries = ctx.fs.read_dir(&tool_dir)?;
        let mut versions: Vec<String> = entries
            .into_iter()
            .filter(|p| ctx.fs.is_dir(p))
            .filter_map(|p| p.file_name().and_then(|n| n.to_str().map(String::from)))
            .collect();

        // Sort versions (newest first) using simple semver comparison
        versions.sort_by(|a, b| compare_semver(b, a));
        Ok(versions)
    }

    async fn uninstall(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        let install_dir = ctx.paths.pip_tool_version_dir(Self::PACKAGE_NAME, version);
        if ctx.fs.exists(&install_dir) {
            ctx.fs.remove_dir_all(&install_dir)?;
        }
        Ok(())
    }

    fn verify_installation(
        &self,
        version: &str,
        _install_path: &Path,
        _platform: &Platform,
    ) -> VerificationResult {
        // Use package verification instead
        let paths = vx_runtime::RealPathProvider::default();
        let bin_dir = paths.pip_tool_bin_dir(Self::PACKAGE_NAME, version);
        let exe_name = if cfg!(windows) { "rez.exe" } else { "rez" };
        let exe_path = bin_dir.join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "Rez executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling with: vx install rez".to_string()],
            )
        }
    }
}

#[async_trait]
impl PackageRuntime for RezRuntime {
    fn install_method(&self) -> InstallMethod {
        InstallMethod::pip(Self::PACKAGE_NAME)
    }

    fn required_runtime(&self) -> &str {
        "uv" // Use uv for faster Python package installation
    }

    fn required_runtime_version(&self) -> Option<&str> {
        None
    }
}
