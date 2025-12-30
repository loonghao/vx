//! AWS CLI runtime implementation

use crate::config::AwsCliUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Arch, Ecosystem, Os, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo,
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

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // AWS CLI uses a single "latest" version on their download site
        // GitHub releases are for development versions only
        // We provide a hardcoded list of known stable versions
        let versions = vec![
            "2.32.25", "2.32.0", "2.31.0", "2.30.0", "2.29.0",
            "2.28.0", "2.27.0", "2.26.0", "2.25.0", "2.24.0",
            "2.23.0", "2.22.0", "2.21.0", "2.20.0", "2.19.0",
            "2.18.0", "2.17.0", "2.16.0", "2.15.0", "latest",
        ];
        
        Ok(versions
            .into_iter()
            .map(|v| VersionInfo::new(v))
            .collect())
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(AwsCliUrlBuilder::download_url(version, platform))
    }

    /// Custom post-install for AWS CLI
    /// 
    /// **Windows Note**: AWS CLI v2 for Windows uses MSI installer format which requires
    /// admin privileges to install. vx currently downloads the MSI but cannot extract it
    /// automatically. Users should either:
    /// 1. Install AWS CLI system-wide manually: https://aws.amazon.com/cli/
    /// 2. Use `vx --use-system-path aws` to use system-installed version
    /// 
    /// **Linux Note**: Users may need to run the install script manually.
    /// The install script is at `aws/install` in the installation directory.
    async fn post_install(&self, _version: &str, _ctx: &RuntimeContext) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            eprintln!("\n⚠️  AWS CLI for Windows uses MSI installer format.");
            eprintln!("   vx has downloaded the MSI file but cannot install it automatically.");
            eprintln!("   Please install AWS CLI manually from: https://aws.amazon.com/cli/");
            eprintln!("   Or use the system-installed version with: vx --use-system-path aws\n");
        }
        
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
