//! CMake runtime implementation
//!
//! CMake is a cross-platform build system generator.
//! https://github.com/Kitware/CMake

use crate::config::CMakeUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, Os, Platform, Runtime, RuntimeContext, VerificationResult,
    VersionInfo,
};

/// CMake runtime implementation
#[derive(Debug, Clone, Default)]
pub struct CMakeRuntime;

impl CMakeRuntime {
    /// Create a new CMake runtime
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for CMakeRuntime {
    fn name(&self) -> &str {
        "cmake"
    }

    fn description(&self) -> &str {
        "CMake - Cross-platform build system generator"
    }

    fn aliases(&self) -> &[&str] {
        &["ctest", "cpack"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://cmake.org/".to_string());
        meta.insert(
            "documentation".to_string(),
            "https://cmake.org/documentation/".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/Kitware/CMake".to_string(),
        );
        meta.insert("category".to_string(), "build-system".to_string());
        meta.insert("license".to_string(), "BSD-3-Clause".to_string());
        meta
    }

    fn executable_relative_path(&self, version: &str, platform: &Platform) -> String {
        let dir_name = CMakeUrlBuilder::get_archive_dir_name(version, platform)
            .unwrap_or_else(|| "cmake".to_string());
        let exe_name = CMakeUrlBuilder::get_executable_name(platform);

        match &platform.os {
            // On macOS, CMake is in CMake.app/Contents/bin/
            Os::MacOS => format!("{}/CMake.app/Contents/bin/{}", dir_name, exe_name),
            // On Windows and Linux, it's in bin/
            _ => format!("{}/bin/{}", dir_name, exe_name),
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // CMake uses 'v' prefix in tags, strip it for version display
        ctx.fetch_github_releases(
            "cmake",
            "Kitware",
            "CMake",
            GitHubReleaseOptions::new()
                .strip_v_prefix(true)
                .skip_prereleases(true),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(CMakeUrlBuilder::download_url(version, platform))
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
                    "CMake executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling the runtime".to_string()],
            )
        }
    }
}
