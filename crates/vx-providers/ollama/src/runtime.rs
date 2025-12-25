//! Ollama runtime implementation
//!
//! Ollama is a tool for running large language models locally.
//! It supports models like Llama, Mistral, Gemma, and many others.
//!
//! Homepage: https://ollama.com
//! Repository: https://github.com/ollama/ollama

use crate::config::OllamaUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VerificationResult,
    VersionInfo,
};

/// Ollama runtime implementation
#[derive(Debug, Clone, Default)]
pub struct OllamaRuntime;

impl OllamaRuntime {
    /// Create a new Ollama runtime
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for OllamaRuntime {
    fn name(&self) -> &str {
        "ollama"
    }

    fn description(&self) -> &str {
        "Ollama - Run large language models locally"
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        // Ollama is a system tool for AI/ML
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://ollama.com".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/ollama/ollama".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://github.com/ollama/ollama/blob/main/README.md".to_string(),
        );
        meta.insert("category".to_string(), "ai".to_string());
        meta.insert("license".to_string(), "MIT".to_string());
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // Ollama archives extract with bin/ subdirectory on Linux/macOS
        // Windows extracts directly
        match platform.os {
            vx_runtime::Os::Windows => OllamaUrlBuilder::get_executable_name(platform).to_string(),
            _ => format!("bin/{}", OllamaUrlBuilder::get_executable_name(platform)),
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        ctx.fetch_github_releases(
            "ollama",
            "ollama",
            "ollama",
            GitHubReleaseOptions::new()
                .strip_v_prefix(true) // Ollama uses 'v' prefix in tags
                .skip_prereleases(true),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(OllamaUrlBuilder::download_url(version, platform))
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
                    "Ollama executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec![
                    "Try reinstalling the runtime with: vx install ollama".to_string(),
                    "Check if the download completed successfully".to_string(),
                ],
            )
        }
    }
}
