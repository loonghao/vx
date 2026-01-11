//! YASM runtime implementation
//!
//! YASM is a modular assembler with support for multiple output formats.
//! https://github.com/yasm/yasm

use crate::config::YasmUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, Os, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo,
};

/// YASM runtime implementation
#[derive(Debug, Clone, Default)]
pub struct YasmRuntime;

impl YasmRuntime {
    /// Create a new YASM runtime
    pub fn new() -> Self {
        Self
    }

    /// GitHub API URL for releases
    const GITHUB_API_URL: &'static str = "https://api.github.com/repos/yasm/yasm/releases";
}

#[async_trait]
impl Runtime for YasmRuntime {
    fn name(&self) -> &str {
        "yasm"
    }

    fn description(&self) -> &str {
        "YASM - Modular Assembler with multiple output formats"
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
            "http://yasm.tortall.net/".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "http://yasm.tortall.net/Guide.html".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/yasm/yasm".to_string(),
        );
        meta.insert("category".to_string(), "assembler".to_string());
        meta.insert("license".to_string(), "BSD-3-Clause".to_string());
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // YASM downloads are standalone executables, not archives
        YasmUrlBuilder::get_executable_name(platform).to_string()
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        let json = ctx.http.get_json_value(Self::GITHUB_API_URL).await?;
        let releases = json
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid JSON response"))?;

        let versions: Vec<VersionInfo> = releases
            .iter()
            .filter_map(|release| {
                let tag = release.get("tag_name")?.as_str()?;
                let version = tag.strip_prefix('v').unwrap_or(tag);
                let prerelease = release.get("prerelease")?.as_bool().unwrap_or(false);
                Some(VersionInfo::new(version).with_prerelease(prerelease))
            })
            .collect();

        Ok(versions)
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(YasmUrlBuilder::download_url(version, platform))
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_name = YasmUrlBuilder::get_executable_name(platform);
        let exe_path = install_path.join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "YASM executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling the runtime".to_string()],
            )
        }
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        use vx_runtime::Arch;
        vec![
            Platform {
                os: Os::Windows,
                arch: Arch::X86_64,
            },
            Platform {
                os: Os::Windows,
                arch: Arch::X86,
            },
        ]
    }
}
