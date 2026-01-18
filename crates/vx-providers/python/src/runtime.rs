//! Python runtime implementation
//!
//! Uses python-build-standalone for portable Python distributions.
//! Downloads prebuilt Python binaries directly from GitHub releases.
//!
//! Supports Python 3.9 to 3.15 versions (3.7 and 3.8 are EOL and no longer available).

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
    /// The release_date is from python-build-standalone releases
    ///
    /// Note: Python 3.7 and 3.8 are EOL and no longer available in python-build-standalone.
    /// The last release with 3.8 was 20241008.
    fn get_builtin_versions() -> Vec<(String, bool, &'static str)> {
        vec![
            // Python 3.15.x (alpha)
            ("3.15.0a5".to_string(), true, "20260114"),
            // Python 3.14.x (beta/rc)
            ("3.14.0a4".to_string(), true, "20250121"),
            // Python 3.13.x (latest stable)
            ("3.13.4".to_string(), false, "20250610"),
            ("3.13.3".to_string(), false, "20250508"),
            ("3.13.2".to_string(), false, "20250212"),
            ("3.13.1".to_string(), false, "20241219"),
            ("3.13.0".to_string(), false, "20241008"),
            // Python 3.12.x (LTS)
            ("3.12.12".to_string(), false, "20260114"),
            ("3.12.11".to_string(), false, "20250610"),
            ("3.12.10".to_string(), false, "20250508"),
            ("3.12.9".to_string(), false, "20250212"),
            ("3.12.8".to_string(), false, "20241219"),
            ("3.12.7".to_string(), false, "20241002"),
            // Python 3.11.x
            ("3.11.14".to_string(), false, "20260114"),
            ("3.11.13".to_string(), false, "20250610"),
            ("3.11.12".to_string(), false, "20250508"),
            ("3.11.11".to_string(), false, "20241206"),
            ("3.11.10".to_string(), false, "20240909"),
            ("3.11.9".to_string(), false, "20240415"),
            // Python 3.10.x
            ("3.10.19".to_string(), false, "20260114"),
            ("3.10.18".to_string(), false, "20250610"),
            ("3.10.17".to_string(), false, "20250508"),
            ("3.10.16".to_string(), false, "20241206"),
            ("3.10.15".to_string(), false, "20240909"),
            ("3.10.14".to_string(), false, "20240415"),
            // Python 3.9.x (EOL but still available)
            ("3.9.23".to_string(), false, "20260114"),
            ("3.9.22".to_string(), false, "20250610"),
            ("3.9.21".to_string(), false, "20241206"),
            ("3.9.20".to_string(), false, "20240909"),
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

    /// Build download URL for python-build-standalone
    ///
    /// Format: https://github.com/astral-sh/python-build-standalone/releases/download/{date}/cpython-{version}+{date}-{platform}-install_only_stripped.tar.gz
    fn build_download_url(version: &str, platform: &Platform) -> Option<String> {
        let platform_str = Self::get_platform_string(platform)?;
        let date = Self::get_release_date(version)?;

        // Use stripped version for smaller download
        // Format: cpython-3.12.8+20241219-x86_64-pc-windows-msvc-install_only_stripped.tar.gz
        Some(format!(
            "https://github.com/astral-sh/python-build-standalone/releases/download/{date}/cpython-{version}+{date}-{platform_str}-install_only_stripped.tar.gz"
        ))
    }
}

#[async_trait]
impl Runtime for PythonRuntime {
    fn name(&self) -> &str {
        "python"
    }

    fn description(&self) -> &str {
        "Python programming language (3.9 - 3.15)"
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
            "3.9, 3.10, 3.11, 3.12, 3.13, 3.14, 3.15".to_string(),
        );
        meta
    }

    fn store_name(&self) -> &str {
        "python"
    }

    /// Python executable path within the extracted archive
    ///
    /// python-build-standalone extracts to: python/bin/python3 (Unix) or python/python.exe (Windows)
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        if platform.is_windows() {
            "python/python.exe".to_string()
        } else {
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
        meta.insert("homepage".to_string(), "https://pip.pypa.io/".to_string());
        meta.insert("ecosystem".to_string(), "python".to_string());
        meta.insert("bundled_with".to_string(), "python".to_string());
        meta
    }

    fn store_name(&self) -> &str {
        "python"
    }

    /// Pip executable path within Python installation
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        if platform.is_windows() {
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
