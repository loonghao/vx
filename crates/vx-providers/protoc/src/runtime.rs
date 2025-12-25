//! Protoc runtime implementation
//!
//! protoc is the Protocol Buffers compiler.
//! https://github.com/protocolbuffers/protobuf

use crate::config::ProtocUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VerificationResult,
    VersionInfo,
};

/// Protoc runtime implementation
#[derive(Debug, Clone, Default)]
pub struct ProtocRuntime;

impl ProtocRuntime {
    /// Create a new protoc runtime
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for ProtocRuntime {
    fn name(&self) -> &str {
        "protoc"
    }

    fn description(&self) -> &str {
        "Protocol Buffers Compiler - Google's data interchange format"
    }

    fn aliases(&self) -> &[&str] {
        &["protobuf"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://protobuf.dev/".to_string());
        meta.insert(
            "documentation".to_string(),
            "https://protobuf.dev/programming-guides/proto3/".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/protocolbuffers/protobuf".to_string(),
        );
        meta.insert("category".to_string(), "serialization".to_string());
        meta.insert("license".to_string(), "BSD-3-Clause".to_string());
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // protoc extracts to bin/ directory
        let exe_name = ProtocUrlBuilder::get_executable_name(platform);
        format!("bin/{}", exe_name)
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // protoc uses 'v' prefix in tags, strip it for version display
        // Filter out releases that don't have protoc binaries (e.g., language-specific releases)
        ctx.fetch_github_releases(
            "protoc",
            "protocolbuffers",
            "protobuf",
            GitHubReleaseOptions::new()
                .strip_v_prefix(true)
                .skip_prereleases(true),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(ProtocUrlBuilder::download_url(version, platform))
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
                    "protoc executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling the runtime".to_string()],
            )
        }
    }
}
