//! PNPM runtime implementation

use crate::config::PnpmUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use vx_runtime::{Ecosystem, Os, Platform, Runtime, RuntimeContext, VersionInfo};

/// PNPM runtime
#[derive(Debug, Clone)]
pub struct PnpmRuntime;

impl PnpmRuntime {
    /// Create a new PNPM runtime
    pub fn new() -> Self {
        Self
    }

    /// Get the standard executable name for the current platform
    fn get_standard_exe_name(platform: &Platform) -> &'static str {
        if platform.os == Os::Windows {
            "pnpm.exe"
        } else {
            "pnpm"
        }
    }
}

impl Default for PnpmRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for PnpmRuntime {
    fn name(&self) -> &str {
        "pnpm"
    }

    fn description(&self) -> &str {
        "Fast, disk space efficient package manager"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    /// PNPM executable path - uses standard name after post_install rename
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        Self::get_standard_exe_name(platform).to_string()
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Would fetch from GitHub API
        Ok(vec![
            VersionInfo::new("9.0.0"),
            VersionInfo::new("8.15.0").with_lts(true),
        ])
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(PnpmUrlBuilder::download_url(version, platform))
    }

    /// Rename the downloaded file to standard name
    async fn post_install(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        let platform = Platform::current();
        let install_dir = ctx.paths.version_store_dir(self.name(), version);

        // Downloaded filename (e.g., pnpm-linux-x64, pnpm-win-x64.exe)
        let downloaded_name = PnpmUrlBuilder::get_filename(&platform);
        let downloaded_path = install_dir.join(&downloaded_name);

        // Standard filename (e.g., pnpm, pnpm.exe)
        let standard_name = Self::get_standard_exe_name(&platform);
        let standard_path = install_dir.join(standard_name);

        // Rename if the downloaded file exists and standard doesn't
        if downloaded_path.exists() && !standard_path.exists() {
            std::fs::rename(&downloaded_path, &standard_path)?;
        }

        Ok(())
    }
}
