//! Python runtime implementation
//!
//! Uses python-build-standalone for portable Python distributions.
//! Downloads prebuilt Python binaries directly from GitHub releases.
//!
//! Supports Python 3.7 to 3.13 versions.
//! - Python 3.8-3.13: Uses python-build-standalone (all platforms)
//! - Python 3.7: Uses Python.org embeddable (Windows only)

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::OnceLock;
use tracing::debug;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};

/// Python runtime using python-build-standalone
#[derive(Debug, Clone, Default)]
pub struct PythonRuntime;

impl PythonRuntime {
    pub fn new() -> Self {
        Self
    }

    /// Get platform string for python-build-standalone
    fn get_platform_string(platform: &Platform) -> Option<&'static str> {
        match (platform.os.as_str(), platform.arch.as_str()) {
            ("windows", "x86_64") | ("windows", "x64") => Some("x86_64-pc-windows-msvc"),
            ("windows", "aarch64") | ("windows", "arm64") => Some("aarch64-pc-windows-msvc"),
            ("darwin", "x86_64") | ("macos", "x86_64") | ("darwin", "x64") | ("macos", "x64") => {
                Some("x86_64-apple-darwin")
            }
            ("darwin", "aarch64")
            | ("macos", "aarch64")
            | ("darwin", "arm64")
            | ("macos", "arm64") => Some("aarch64-apple-darwin"),
            ("linux", "x86_64") | ("linux", "x64") => Some("x86_64-unknown-linux-gnu"),
            ("linux", "aarch64") | ("linux", "arm64") => Some("aarch64-unknown-linux-gnu"),
            _ => None,
        }
    }

    /// Get built-in version list with their release dates
    ///
    /// Format: (version, is_prerelease, release_date)
    /// The release_date is from python-build-standalone releases.
    /// For Python 3.7.x, release_date is "pythonorg" indicating Python.org source.
    ///
    /// Note:
    /// - Python 3.7 is EOL and only available from Python.org (Windows only)
    /// - Python 3.8 is EOL (Oct 2024) but recent releases still include it.
    /// - Python 3.9 is EOL (Oct 2024) - only versions up to 3.9.21 are available.
    fn get_builtin_versions() -> Vec<(String, bool, &'static str)> {
        vec![
            // Python 3.13.x (latest stable)
            ("3.13.4".to_string(), false, "20250610"),
            ("3.13.3".to_string(), false, "20250508"),
            ("3.13.2".to_string(), false, "20250212"),
            ("3.13.1".to_string(), false, "20241219"),
            ("3.13.0".to_string(), false, "20241008"),
            // Python 3.12.x (LTS)
            ("3.12.11".to_string(), false, "20250610"),
            ("3.12.10".to_string(), false, "20250508"),
            ("3.12.9".to_string(), false, "20250212"),
            ("3.12.8".to_string(), false, "20241219"),
            ("3.12.7".to_string(), false, "20241002"),
            // Python 3.11.x
            ("3.11.13".to_string(), false, "20250610"),
            ("3.11.12".to_string(), false, "20250508"),
            ("3.11.11".to_string(), false, "20241206"),
            ("3.11.10".to_string(), false, "20240909"),
            ("3.11.9".to_string(), false, "20240415"),
            // Python 3.10.x
            ("3.10.18".to_string(), false, "20250610"),
            ("3.10.17".to_string(), false, "20250508"),
            ("3.10.16".to_string(), false, "20241206"),
            ("3.10.15".to_string(), false, "20240909"),
            ("3.10.14".to_string(), false, "20240415"),
            // Python 3.9.x (EOL Oct 2024)
            ("3.9.21".to_string(), false, "20241206"),
            ("3.9.20".to_string(), false, "20240909"),
            // Python 3.8.x (EOL Oct 2024 - only recent releases have install_only_stripped)
            ("3.8.20".to_string(), false, "20241002"),
            ("3.8.19".to_string(), false, "20240814"),
            // Python 3.7.x (EOL - Windows only from Python.org)
            ("3.7.9".to_string(), false, "pythonorg"),
            ("3.7.8".to_string(), false, "pythonorg"),
            ("3.7.7".to_string(), false, "pythonorg"),
            ("3.7.6".to_string(), false, "pythonorg"),
            ("3.7.5".to_string(), false, "pythonorg"),
        ]
    }

    /// Find the release date for a given version
    fn get_release_date(version: &str) -> Option<&'static str> {
        let versions = Self::get_builtin_versions();
        for (v, _, date) in &versions {
            if v.as_str() == version {
                return Some(date);
            }
        }
        None
    }

    /// Check if version is Python 3.7.x
    pub fn is_python_37(version: &str) -> bool {
        version.starts_with("3.7.")
    }

    /// Build download URL for Python
    ///
    /// For Python 3.8+: Uses python-build-standalone
    /// Format: https://github.com/astral-sh/python-build-standalone/releases/download/{date}/cpython-{version}+{date}-{platform}-install_only_stripped.tar.gz
    ///
    /// For Python 3.7: Uses Python.org embeddable (Windows only)
    /// Format: https://www.python.org/ftp/python/{version}/python-{version}-embed-{arch}.zip
    fn build_download_url(version: &str, platform: &Platform) -> Option<String> {
        if Self::is_python_37(version) {
            // Python 3.7: Only Windows supported via Python.org embeddable
            if !platform.is_windows() {
                return None;
            }

            // Only specific 3.7 versions have embeddable downloads on Python.org
            // Check if version is in our supported list
            let supported_37_versions = ["3.7.9", "3.7.8", "3.7.7", "3.7.6", "3.7.5"];
            if !supported_37_versions.contains(&version) {
                return None;
            }

            // Python.org embeddable format
            // amd64 for x86_64, win32 for x86
            let arch = match platform.arch.as_str() {
                "x86_64" | "x64" | "amd64" => "amd64",
                "x86" | "i686" => "win32",
                "aarch64" | "arm64" => "arm64", // Not available for 3.7, but include for completeness
                _ => return None,
            };
            Some(format!(
                "https://www.python.org/ftp/python/{version}/python-{version}-embed-{arch}.zip"
            ))
        } else {
            // Python 3.8+: Use python-build-standalone
            let platform_str = Self::get_platform_string(platform)?;
            let date = Self::get_release_date(version)?;

            // Use stripped version for smaller download
            // Format: cpython-3.12.8+20241219-x86_64-pc-windows-msvc-install_only_stripped.tar.gz
            Some(format!(
                "https://github.com/astral-sh/python-build-standalone/releases/download/{date}/cpython-{version}+{date}-{platform_str}-install_only_stripped.tar.gz"
            ))
        }
    }
}

#[async_trait]
impl Runtime for PythonRuntime {
    fn name(&self) -> &str {
        "python"
    }

    fn description(&self) -> &str {
        "Python programming language (3.7 - 3.13)"
    }

    fn aliases(&self) -> &[&str] {
        &["python3", "py"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Python
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://www.python.org/".to_string(),
        );
        meta.insert("ecosystem".to_string(), "python".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/python/cpython".to_string(),
        );
        meta.insert("license".to_string(), "PSF-2.0".to_string());
        meta.insert(
            "source".to_string(),
            "python-build-standalone (astral-sh)".to_string(),
        );
        meta.insert(
            "supported_versions".to_string(),
            "3.7 (Windows only), 3.8, 3.9, 3.10, 3.11, 3.12, 3.13".to_string(),
        );
        meta
    }

    fn store_name(&self) -> &str {
        "python"
    }

    /// Python executable path within the extracted archive
    ///
    /// python-build-standalone extracts to:
    /// cpython-{version}+{date}-{platform}-install_only_stripped/python/python.exe (Windows)
    /// cpython-{version}+{date}-{platform}-install_only_stripped/python/bin/python3 (Unix)
    ///
    /// Python.org embeddable (3.7) extracts to: python.exe (flat structure)
    ///
    /// Note: We use glob patterns in the manifest to search for the executable
    /// This method returns a reasonable fallback for verification
    fn executable_relative_path(&self, version: &str, platform: &Platform) -> String {
        if Self::is_python_37(version) {
            // Python.org embeddable has flat structure
            "python.exe".to_string()
        } else if platform.is_windows() {
            // Windows: python/python.exe relative to extracted root
            // But the root directory name varies, so we search with glob in manifest
            "python/python.exe".to_string()
        } else {
            // Unix: python/bin/python3 relative to extracted root
            "python/bin/python3".to_string()
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Try to fetch from jsdelivr (GitHub proxy)
        let url = "https://data.jsdelivr.com/v1/package/gh/astral-sh/python-build-standalone";

        match ctx
            .get_cached_or_fetch_with_url("python-build-standalone", url, || async {
                ctx.http.get_json_value(url).await
            })
            .await
        {
            Ok(response) => {
                // Parse release dates from versions like "20260114"
                if let Some(versions) = response.get("versions").and_then(|v| v.as_array()) {
                    let release_dates: Vec<String> = versions
                        .iter()
                        .filter_map(|v| v.as_str())
                        .filter(|v| v.chars().all(|c| c.is_ascii_digit()) && v.len() == 8)
                        .map(|v| v.to_string())
                        .collect();

                    debug!(
                        "Found {} release dates from python-build-standalone",
                        release_dates.len()
                    );
                    // We have the dates, but we still use builtin versions
                    // because we need the Python versions, not release dates
                }
            }
            Err(e) => {
                debug!(
                    "Failed to fetch from jsdelivr: {}. Using builtin versions.",
                    e
                );
            }
        }

        // Use builtin version list
        let versions = Self::get_builtin_versions()
            .into_iter()
            .map(|(version, is_prerelease, _)| {
                VersionInfo::new(&version).with_prerelease(is_prerelease)
            })
            .collect();

        Ok(versions)
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(Self::build_download_url(version, platform))
    }
}

// ============================================================================
// Pip Runtime - Bundled with Python
// ============================================================================

/// Pip runtime - Python package installer (bundled with Python)
#[derive(Debug, Clone, Default)]
pub struct PipRuntime;

impl PipRuntime {
    pub fn new() -> Self {
        Self
    }
}

/// Static dependency on python
static PYTHON_DEPENDENCY: OnceLock<[vx_runtime::RuntimeDependency; 1]> = OnceLock::new();

fn get_python_dependency() -> &'static [vx_runtime::RuntimeDependency; 1] {
    PYTHON_DEPENDENCY.get_or_init(|| {
        [vx_runtime::RuntimeDependency::required("python")
            .with_reason("pip is bundled with Python")]
    })
}

#[async_trait]
impl Runtime for PipRuntime {
    fn name(&self) -> &str {
        "pip"
    }

    fn description(&self) -> &str {
        "Python package installer (bundled with Python)"
    }

    fn aliases(&self) -> &[&str] {
        &["pip3"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Python
    }

    fn dependencies(&self) -> &[vx_runtime::RuntimeDependency] {
        get_python_dependency()
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "provider".to_string(),
            "python-build-standalone".to_string(),
        );
        meta.insert(
            "source".to_string(),
            "https://github.com/astral-sh/python-build-standalone".to_string(),
        );
        meta
    }

    fn possible_bin_dirs(&self) -> Vec<&str> {
        vec!["python", "bin"]
    }

    fn store_name(&self) -> &str {
        "python"
    }

    /// Pip executable path within Python installation
    ///
    /// Note: Python 3.7 from Python.org embeddable does NOT include pip.
    /// For Python 3.7, pip needs to be installed separately.
    fn executable_relative_path(&self, version: &str, platform: &Platform) -> String {
        if PythonRuntime::is_python_37(version) {
            // Python.org embeddable doesn't include pip
            // Return the path where pip would be if installed via get-pip.py
            "Scripts/pip.exe".to_string()
        } else if platform.is_windows() {
            "python/Scripts/pip.exe".to_string()
        } else {
            "python/bin/pip3".to_string()
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Pip is bundled with Python, use Python versions
        let python_runtime = PythonRuntime::new();
        python_runtime.fetch_versions(ctx).await
    }

    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        // Pip is bundled with Python, no separate download
        Ok(None)
    }
}
