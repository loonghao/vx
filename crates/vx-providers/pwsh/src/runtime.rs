//! PowerShell runtime implementation
//!
//! PowerShell 7+ is a cross-platform shell and scripting language.
//! https://github.com/PowerShell/PowerShell

use crate::config::PwshUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use vx_runtime::{
    Ecosystem, ExecutionResult, GitHubReleaseOptions, Platform, Runtime, RuntimeContext,
    VersionInfo,
};

/// PowerShell runtime - cross-platform shell and scripting language
#[derive(Debug, Clone, Default)]
pub struct PwshRuntime;

impl PwshRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for PwshRuntime {
    fn name(&self) -> &str {
        "pwsh"
    }

    fn description(&self) -> &str {
        "PowerShell - Cross-platform shell and scripting language"
    }

    fn aliases(&self) -> &[&str] {
        &["powershell", "ps"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        ctx.fetch_github_releases(
            "pwsh",
            "PowerShell",
            "PowerShell",
            GitHubReleaseOptions::new()
                .strip_v_prefix(true)
                .skip_prereleases(true),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(PwshUrlBuilder::download_url(version, platform))
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        format!("{}", PwshUrlBuilder::get_executable_name(platform))
    }

    async fn execute(
        &self,
        args: &[String],
        _ctx: &vx_runtime::ExecutionContext,
    ) -> Result<ExecutionResult> {
        use std::process::Command;

        let output = Command::new("pwsh")
            .args(args)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute pwsh: {}", e))?;

        Ok(ExecutionResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: Some(String::from_utf8_lossy(&output.stdout).to_string()),
            stderr: Some(String::from_utf8_lossy(&output.stderr).to_string()),
        })
    }
}
