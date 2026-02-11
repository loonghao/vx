//! yq runtime implementation
//!
//! yq is a portable command-line YAML, JSON, XML, CSV, TOML and properties processor.
//! https://github.com/mikefarah/yq

use crate::config::YqUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    layout::{BinaryLayout, DownloadType, ExecutableLayout},
    Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VerificationResult,
    VersionInfo,
};

/// yq runtime implementation
#[derive(Debug, Clone, Default)]
pub struct YqRuntime;

impl YqRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for YqRuntime {
    fn name(&self) -> &str {
        "yq"
    }

    fn description(&self) -> &str {
        "A portable command-line YAML, JSON, XML, CSV, TOML and properties processor"
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://github.com/mikefarah/yq".to_string(),
        );
        meta.insert("license".to_string(), "MIT".to_string());
        meta.insert("category".to_string(), "data-processor".to_string());
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        format!("bin/{}", YqUrlBuilder::get_executable_name(platform))
    }

    fn executable_layout(&self) -> Option<ExecutableLayout> {
        let mut binary_configs = HashMap::new();

        binary_configs.insert(
            "windows-x64".to_string(),
            BinaryLayout {
                source_name: "yq_windows_amd64.exe".to_string(),
                target_name: "yq.exe".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: None,
            },
        );

        binary_configs.insert(
            "windows-arm64".to_string(),
            BinaryLayout {
                source_name: "yq_windows_arm64.exe".to_string(),
                target_name: "yq.exe".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: None,
            },
        );

        binary_configs.insert(
            "darwin-x64".to_string(),
            BinaryLayout {
                source_name: "yq_darwin_amd64".to_string(),
                target_name: "yq".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: Some("755".to_string()),
            },
        );

        binary_configs.insert(
            "darwin-arm64".to_string(),
            BinaryLayout {
                source_name: "yq_darwin_arm64".to_string(),
                target_name: "yq".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: Some("755".to_string()),
            },
        );

        binary_configs.insert(
            "linux-x64".to_string(),
            BinaryLayout {
                source_name: "yq_linux_amd64".to_string(),
                target_name: "yq".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: Some("755".to_string()),
            },
        );

        binary_configs.insert(
            "linux-arm64".to_string(),
            BinaryLayout {
                source_name: "yq_linux_arm64".to_string(),
                target_name: "yq".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: Some("755".to_string()),
            },
        );

        Some(ExecutableLayout {
            download_type: DownloadType::Binary,
            binary: Some(binary_configs),
            archive: None,
            msi: None,
            windows: None,
            macos: None,
            linux: None,
        })
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        ctx.fetch_github_releases(
            "yq",
            "mikefarah",
            "yq",
            GitHubReleaseOptions::new()
                .strip_v_prefix(true)
                .skip_prereleases(true),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(YqUrlBuilder::download_url(version, platform))
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_name = YqUrlBuilder::get_executable_name(platform);
        let exe_path = install_path.join("bin").join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "yq executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling the runtime".to_string()],
            )
        }
    }
}
