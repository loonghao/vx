//! jq runtime implementation
//!
//! jq is a lightweight and flexible command-line JSON processor.
//! https://github.com/jqlang/jq

use crate::config::JqUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VerificationResult,
    VersionInfo,
    layout::{BinaryLayout, DownloadType, ExecutableLayout},
};

/// jq runtime implementation
#[derive(Debug, Clone, Default)]
pub struct JqRuntime;

impl JqRuntime {
    /// Create a new jq runtime
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for JqRuntime {
    fn name(&self) -> &str {
        "jq"
    }

    fn description(&self) -> &str {
        "A lightweight and flexible command-line JSON processor"
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
            "https://jqlang.github.io/jq/".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://jqlang.github.io/jq/manual/".to_string(),
        );
        meta.insert("category".to_string(), "json-processor".to_string());
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // jq binary is placed in bin/ directory after installation
        format!("bin/{}", JqUrlBuilder::get_executable_name(platform))
    }

    fn executable_layout(&self) -> Option<ExecutableLayout> {
        let mut binary_configs = HashMap::new();

        // Windows x86_64 (platform key format: {os}-{arch} where os=windows, arch=x64)
        binary_configs.insert(
            "windows-x64".to_string(),
            BinaryLayout {
                source_name: "jq-windows-amd64.exe".to_string(),
                target_name: "jq.exe".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: None,
            },
        );

        // Windows ARM64
        binary_configs.insert(
            "windows-arm64".to_string(),
            BinaryLayout {
                source_name: "jq-windows-arm64.exe".to_string(),
                target_name: "jq.exe".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: None,
            },
        );

        // Windows x86
        binary_configs.insert(
            "windows-x86".to_string(),
            BinaryLayout {
                source_name: "jq-windows-i386.exe".to_string(),
                target_name: "jq.exe".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: None,
            },
        );

        // macOS x86_64 (os=darwin, arch=x64)
        binary_configs.insert(
            "darwin-x64".to_string(),
            BinaryLayout {
                source_name: "jq-macos-amd64".to_string(),
                target_name: "jq".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: Some("755".to_string()),
            },
        );

        // macOS ARM64 (os=darwin, arch=arm64)
        binary_configs.insert(
            "darwin-arm64".to_string(),
            BinaryLayout {
                source_name: "jq-macos-arm64".to_string(),
                target_name: "jq".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: Some("755".to_string()),
            },
        );

        // Linux x86_64 (os=linux, arch=x64)
        binary_configs.insert(
            "linux-x64".to_string(),
            BinaryLayout {
                source_name: "jq-linux-amd64".to_string(),
                target_name: "jq".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: Some("755".to_string()),
            },
        );

        // Linux ARM64 (os=linux, arch=arm64)
        binary_configs.insert(
            "linux-arm64".to_string(),
            BinaryLayout {
                source_name: "jq-linux-arm64".to_string(),
                target_name: "jq".to_string(),
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
        // jq uses 'jq-X.Y.Z' tag format
        ctx.fetch_github_releases(
            "jq",
            "jqlang",
            "jq",
            GitHubReleaseOptions::new()
                .strip_v_prefix(false)
                .tag_prefix("jq-")
                .skip_prereleases(true),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(JqUrlBuilder::download_url(version, platform))
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_name = JqUrlBuilder::get_executable_name(platform);
        let exe_path = install_path.join("bin").join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "jq executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling the runtime".to_string()],
            )
        }
    }
}
