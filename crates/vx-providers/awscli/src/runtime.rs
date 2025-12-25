//! AWS CLI runtime implementation

use crate::config::AwsCliUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Arch, Ecosystem, GitHubReleaseOptions, Os, Platform, Runtime, RuntimeContext,
    VerificationResult, VersionInfo,
};

/// AWS CLI runtime
#[derive(Debug, Clone)]
pub struct AwsCliRuntime;

impl AwsCliRuntime {
    /// Create a new AWS CLI runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for AwsCliRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for AwsCliRuntime {
    fn name(&self) -> &str {
        "aws"
    }

    fn description(&self) -> &str {
        "AWS CLI v2 - Amazon Web Services command-line interface"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Custom("cloud".to_string())
    }

    fn aliases(&self) -> &[&str] {
        &["awscli", "aws-cli"]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://aws.amazon.com/cli/".to_string(),
        );
        meta.insert("ecosystem".to_string(), "cloud".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/aws/aws-cli".to_string(),
        );
        meta.insert("license".to_string(), "Apache-2.0".to_string());
        meta
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        vec![
            Platform::new(Os::Linux, Arch::X86_64),
            Platform::new(Os::Linux, Arch::Aarch64),
            Platform::new(Os::MacOS, Arch::X86_64),
            Platform::new(Os::MacOS, Arch::Aarch64),
            Platform::new(Os::Windows, Arch::X86_64),
        ]
    }

    /// AWS CLI executable path varies by platform
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        use vx_runtime::Os;

        match &platform.os {
            // Linux: extracted to aws/dist/aws
            Os::Linux => "aws/dist/aws".to_string(),
            // macOS: after pkg extraction, aws is in a specific location
            Os::MacOS => "aws-cli/aws".to_string(),
            // Windows: after msi extraction
            Os::Windows => "Amazon/AWSCLIV2/aws.exe".to_string(),
            _ => "aws".to_string(),
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch from aws/aws-cli GitHub releases
        ctx.fetch_github_releases(
            "aws",
            "aws",
            "aws-cli",
            GitHubReleaseOptions::new().strip_v_prefix(false),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(AwsCliUrlBuilder::download_url(version, platform))
    }

    /// Custom post-install for AWS CLI
    /// AWS CLI requires running an installer script on Linux
    async fn post_install(&self, _version: &str, _ctx: &RuntimeContext) -> Result<()> {
        // On Linux, users may need to run the install script manually
        // The install script is at aws/install
        Ok(())
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
                    "AWS CLI executable not found at {}",
                    exe_path.display()
                )],
                vec![],
            )
        }
    }
}
