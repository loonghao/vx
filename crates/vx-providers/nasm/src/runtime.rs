//! NASM runtime implementation
//!
//! NASM is a portable 80x86 and x86-64 assembler.
//! https://www.nasm.us/

use crate::config::NasmUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, Os, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo,
};

/// NASM runtime implementation
#[derive(Debug, Clone, Default)]
pub struct NasmRuntime;

impl NasmRuntime {
    /// Create a new NASM runtime
    pub fn new() -> Self {
        Self
    }

    /// Parse versions from the NASM release page HTML
    fn parse_versions_from_html(html: &str) -> Vec<String> {
        let version_regex = regex::Regex::new(r#"href="(\d+\.\d+(?:\.\d+)?)/""#).unwrap();
        let mut versions: Vec<String> = version_regex
            .captures_iter(html)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .filter(|v| !v.contains("rc")) // Skip release candidates
            .collect();

        // Sort versions in descending order (newest first)
        versions.sort_by(|a, b| {
            let parse_version =
                |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse().ok()).collect() };
            parse_version(b).cmp(&parse_version(a))
        });
        versions.dedup();
        versions
    }
}

#[async_trait]
impl Runtime for NasmRuntime {
    fn name(&self) -> &str {
        "nasm"
    }

    fn description(&self) -> &str {
        "NASM - Netwide Assembler for x86 and x86-64"
    }

    fn aliases(&self) -> &[&str] {
        &["ndisasm"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://www.nasm.us/".to_string());
        meta.insert(
            "documentation".to_string(),
            "https://www.nasm.us/doc/".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/netwide-assembler/nasm".to_string(),
        );
        meta.insert("category".to_string(), "assembler".to_string());
        meta.insert("license".to_string(), "BSD-2-Clause".to_string());
        meta
    }

    fn executable_relative_path(&self, version: &str, platform: &Platform) -> String {
        let dir_name = NasmUrlBuilder::get_archive_dir_name(version, platform)
            .unwrap_or_else(|| "nasm".to_string());
        let exe_name = NasmUrlBuilder::get_executable_name(platform);
        format!("{}/{}", dir_name, exe_name)
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch the release page and parse versions
        let url = "https://www.nasm.us/pub/nasm/releasebuilds/";
        if let Ok(html) = ctx.http.get(url).await {
            let versions = Self::parse_versions_from_html(&html);
            if !versions.is_empty() {
                return Ok(versions.into_iter().map(VersionInfo::new).collect());
            }
        }

        // Fallback: provide known stable versions
        Ok(vec![
            VersionInfo::new("2.16.03"),
            VersionInfo::new("2.16.02"),
            VersionInfo::new("2.16.01"),
            VersionInfo::new("2.15.05"),
            VersionInfo::new("2.15.04"),
        ])
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(NasmUrlBuilder::download_url(version, platform))
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
                    "NASM executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling the runtime".to_string()],
            )
        }
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        use vx_runtime::Arch;
        vec![
            Platform::new(Os::Windows, Arch::X86_64),
            Platform::new(Os::Windows, Arch::X86),
            Platform::new(Os::MacOS, Arch::X86_64),
            Platform::new(Os::MacOS, Arch::Aarch64),
        ]
    }
}
