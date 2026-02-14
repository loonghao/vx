//! Hadolint runtime implementation
//!
//! Hadolint is a Dockerfile linter that helps you build best practice Docker images.
//! It parses Dockerfiles and checks them against proven best practices.
//!
//! Homepage: https://github.com/hadolint/hadolint
//! License: GPL-3.0

use crate::config::HadolintUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo,
    layout::{BinaryLayout, DownloadType, ExecutableLayout},
};
use vx_version_fetcher::VersionFetcherBuilder;

/// Hadolint runtime implementation
#[derive(Debug, Clone, Default)]
pub struct HadolintRuntime;

impl HadolintRuntime {
    /// Create a new Hadolint runtime
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for HadolintRuntime {
    fn name(&self) -> &str {
        "hadolint"
    }

    fn description(&self) -> &str {
        "Hadolint - Dockerfile linter for best practices"
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Custom("devtools".to_string())
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://github.com/hadolint/hadolint".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/hadolint/hadolint".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://github.com/hadolint/hadolint#readme".to_string(),
        );
        meta.insert("category".to_string(), "linter".to_string());
        meta.insert("license".to_string(), "GPL-3.0".to_string());
        meta
    }

    /// Hadolint is a single binary download, placed under bin/
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        format!("bin/{}", platform.exe_name("hadolint"))
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        VersionFetcherBuilder::github_releases("hadolint", "hadolint")
            .tool_name("hadolint")
            .strip_v_prefix()
            .skip_prereleases()
            .limit(50)
            .build()
            .fetch(ctx)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(HadolintUrlBuilder::download_url(version, platform))
    }

    fn executable_layout(&self) -> Option<ExecutableLayout> {
        let mut binary = HashMap::new();

        // Keys must match format!("{}-{}", Os::as_str(), Arch::as_str())
        // Os: windows, darwin, linux; Arch: x64, arm64
        binary.insert(
            "windows-x64".to_string(),
            BinaryLayout {
                source_name: "hadolint-windows-x86_64.exe".to_string(),
                target_name: "hadolint.exe".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: None,
            },
        );
        binary.insert(
            "darwin-x64".to_string(),
            BinaryLayout {
                source_name: "hadolint-macos-x86_64".to_string(),
                target_name: "hadolint".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: Some("755".to_string()),
            },
        );
        binary.insert(
            "darwin-arm64".to_string(),
            BinaryLayout {
                source_name: "hadolint-macos-arm64".to_string(),
                target_name: "hadolint".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: Some("755".to_string()),
            },
        );
        binary.insert(
            "linux-x64".to_string(),
            BinaryLayout {
                source_name: "hadolint-linux-x86_64".to_string(),
                target_name: "hadolint".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: Some("755".to_string()),
            },
        );
        binary.insert(
            "linux-arm64".to_string(),
            BinaryLayout {
                source_name: "hadolint-linux-arm64".to_string(),
                target_name: "hadolint".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: Some("755".to_string()),
            },
        );

        Some(ExecutableLayout {
            download_type: DownloadType::Binary,
            binary: Some(binary),
            archive: None,
            windows: None,
            macos: None,
            linux: None,
            msi: None,
        })
    }

    fn verify_installation(
        &self,
        version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_path = install_path.join(self.executable_relative_path(version, platform));

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "Hadolint executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling the runtime".to_string()],
            )
        }
    }
}
