//! Bun runtime implementation

use crate::config::BunUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};

/// Bun runtime
#[derive(Debug, Clone)]
pub struct BunRuntime;

impl BunRuntime {
    /// Create a new Bun runtime
    pub fn new() -> Self {
        Self
    }

    /// Get the platform-specific directory name inside the zip
    fn get_archive_dir_name(platform: &Platform) -> &'static str {
        use vx_runtime::{Arch, Os};
        match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => "bun-windows-x64",
            (Os::MacOS, Arch::X86_64) => "bun-darwin-x64",
            (Os::MacOS, Arch::Aarch64) => "bun-darwin-aarch64",
            (Os::Linux, Arch::X86_64) => "bun-linux-x64",
            (Os::Linux, Arch::Aarch64) => "bun-linux-aarch64",
            _ => "bun-linux-x64",
        }
    }
}

impl Default for BunRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for BunRuntime {
    fn name(&self) -> &str {
        "bun"
    }

    fn description(&self) -> &str {
        "Incredibly fast JavaScript runtime, bundler, test runner, and package manager"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    /// Bun archives extract to `bun-{platform}/bun`
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        let dir_name = Self::get_archive_dir_name(platform);
        let exe_name = if platform.os == vx_runtime::Os::Windows {
            "bun.exe"
        } else {
            "bun"
        };
        format!("{}/{}", dir_name, exe_name)
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Bun versions - would fetch from GitHub API in production
        Ok(vec![
            VersionInfo::new("1.3.4").with_lts(true), // Latest stable
            VersionInfo::new("1.2.0"),
            VersionInfo::new("1.1.0"),
        ])
    }

    async fn download_url(&self, version: &str, _platform: &Platform) -> Result<Option<String>> {
        let (platform, arch) = BunUrlBuilder::get_platform_string();
        Ok(BunUrlBuilder::download_url(version, platform, arch))
    }
}

/// Bunx runtime (package runner)
#[derive(Debug, Clone)]
pub struct BunxRuntime;

impl BunxRuntime {
    /// Create a new Bunx runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for BunxRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for BunxRuntime {
    fn name(&self) -> &str {
        "bunx"
    }

    fn description(&self) -> &str {
        "Bun package runner (like npx)"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::NodeJs
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    /// Bunx is bundled with Bun, same archive structure
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        let dir_name = BunRuntime::get_archive_dir_name(platform);
        let exe_name = if platform.os == vx_runtime::Os::Windows {
            "bunx.exe"
        } else {
            "bunx"
        };
        format!("{}/{}", dir_name, exe_name)
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Bunx uses the same versions as Bun
        BunRuntime::new().fetch_versions(ctx).await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        // Bunx is part of Bun installation
        BunRuntime::new().download_url(version, platform).await
    }
}
