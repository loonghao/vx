//! Go runtime implementation
//!
//! This module provides the Go programming language runtime.

use crate::config::GoUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info, warn};
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};

/// Go programming language runtime
#[derive(Debug, Clone, Default)]
pub struct GoRuntime;

impl GoRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for GoRuntime {
    fn name(&self) -> &str {
        "go"
    }

    fn description(&self) -> &str {
        "Go programming language"
    }

    fn aliases(&self) -> &[&str] {
        &["golang"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Go
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://golang.org/".to_string());
        meta.insert("ecosystem".to_string(), "go".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/golang/go".to_string(),
        );
        meta.insert("license".to_string(), "BSD-3-Clause".to_string());
        meta
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch from Go download page API with caching
        let url = "https://go.dev/dl/?mode=json";

        let response = ctx
            .get_cached_or_fetch("go", || async { ctx.http.get_json_value(url).await })
            .await?;

        let versions: Vec<VersionInfo> = response
            .as_array()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid response format from Go API. Response: {}",
                    serde_json::to_string_pretty(&response).unwrap_or_default()
                )
            })?
            .iter()
            .filter_map(|v| {
                let version_str = v.get("version")?.as_str()?;
                // Remove 'go' prefix
                let version = version_str.strip_prefix("go").unwrap_or(version_str);
                let stable = v.get("stable").and_then(|s| s.as_bool()).unwrap_or(false);

                Some(VersionInfo {
                    version: version.to_string(),
                    released_at: None,
                    prerelease: !stable,
                    lts: false,
                    download_url: None,
                    checksum: None,
                    metadata: HashMap::new(),
                })
            })
            .collect();

        Ok(versions)
    }

    /// Go archives extract to a `go/` subdirectory
    /// e.g., go1.21.0.darwin-arm64.tar.gz extracts to: go/bin/go
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        format!("go/bin/{}", platform.exe_name("go"))
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(GoUrlBuilder::download_url(version, platform))
    }

    /// Pre-run hook for go commands
    ///
    /// For "go run" commands, ensures module dependencies are downloaded first.
    async fn pre_run(&self, args: &[String], executable: &Path) -> Result<bool> {
        // Handle "go run" commands
        if args.first().is_some_and(|a| a == "run") {
            ensure_go_mod_downloaded(executable).await?;
        }
        Ok(true)
    }
}

/// Helper function to ensure go modules are downloaded before running commands
async fn ensure_go_mod_downloaded(executable: &Path) -> Result<()> {
    // Check if go.mod exists
    let go_mod = Path::new("go.mod");
    if !go_mod.exists() {
        debug!("No go.mod found, skipping go mod download");
        return Ok(());
    }

    // Check if go.sum exists - if it does, modules might already be cached
    // We'll still run download to ensure everything is available
    let go_sum = Path::new("go.sum");
    if !go_sum.exists() {
        debug!("No go.sum found, running go mod download");
    }

    // Check if vendor directory exists - if so, skip download
    let vendor = Path::new("vendor");
    if vendor.exists() {
        debug!("vendor directory exists, assuming dependencies are vendored");
        return Ok(());
    }

    info!("Downloading Go module dependencies...");

    let status = Command::new(executable)
        .args(["mod", "download"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?;

    if !status.success() {
        warn!("go mod download failed, continuing anyway...");
    }

    Ok(())
}
