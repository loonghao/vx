//! Meson runtime implementation
//!
//! Meson is installed via pip/uv as it's a Python package.
//! https://github.com/mesonbuild/meson

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, InstallResult, Os, Platform, Runtime, RuntimeContext, RuntimeDependency,
    VerificationResult, VersionInfo,
};

/// Meson runtime implementation
#[derive(Debug, Clone, Default)]
pub struct MesonRuntime;

impl MesonRuntime {
    /// Create a new Meson runtime
    pub fn new() -> Self {
        Self
    }

    /// PyPI API URL for meson
    const PYPI_API_URL: &'static str = "https://pypi.org/pypi/meson/json";
}

#[async_trait]
impl Runtime for MesonRuntime {
    fn name(&self) -> &str {
        "meson"
    }

    fn description(&self) -> &str {
        "Meson - An extremely fast and user friendly build system"
    }

    fn aliases(&self) -> &[&str] {
        &["mesonbuild"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Python
    }

    fn dependencies(&self) -> &[RuntimeDependency] {
        // Meson requires uv (or pip) to install
        // Note: We can't return a static slice with runtime-created values,
        // so we'll use the default implementation which returns empty slice.
        // The dependency on uv is handled by the install method.
        &[]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://mesonbuild.com/".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://mesonbuild.com/Manual.html".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/mesonbuild/meson".to_string(),
        );
        meta.insert("category".to_string(), "build-system".to_string());
        meta.insert("license".to_string(), "Apache-2.0".to_string());
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // Installed via uv tool, executable is in bin/
        match platform.os {
            Os::Windows => "bin/meson.exe".to_string(),
            _ => "bin/meson".to_string(),
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch versions from PyPI
        let json = ctx.http.get_json_value(Self::PYPI_API_URL).await?;

        let releases = json
            .get("releases")
            .and_then(|r| r.as_object())
            .ok_or_else(|| anyhow::anyhow!("Invalid PyPI response"))?;

        let mut versions: Vec<VersionInfo> = releases
            .keys()
            .filter(|v| {
                !v.contains("rc") && !v.contains("dev") && !v.contains('a') && !v.contains('b')
            })
            .map(|v| VersionInfo::new(v))
            .collect();

        // Sort versions in descending order (simple string comparison)
        versions.sort_by(|a, b| b.version.cmp(&a.version));

        Ok(versions)
    }

    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        // Meson is installed via pip/uv, not direct download
        Ok(None)
    }

    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        use std::process::Command;

        let install_path = ctx.paths.version_store_dir(self.name(), version);
        std::fs::create_dir_all(&install_path)?;

        // Use uv tool install to install meson
        let status = Command::new("uv")
            .args([
                "tool",
                "install",
                "--force",
                &format!("meson=={}", version),
                "--target",
                &install_path.to_string_lossy(),
            ])
            .status()?;

        if !status.success() {
            return Err(anyhow::anyhow!(
                "Failed to install meson {} via uv",
                version
            ));
        }

        let platform = Platform::current();
        let exe_name = match platform.os {
            Os::Windows => "meson.exe",
            _ => "meson",
        };
        let exe_path = install_path.join("bin").join(exe_name);

        Ok(InstallResult::success(
            install_path,
            exe_path,
            version.to_string(),
        ))
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_name = match platform.os {
            Os::Windows => "meson.exe",
            _ => "meson",
        };
        let exe_path = install_path.join("bin").join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "Meson executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling with: vx install meson".to_string()],
            )
        }
    }
}
