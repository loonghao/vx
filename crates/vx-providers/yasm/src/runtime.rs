//! YASM runtime implementation
//!
//! YASM is a modular assembler with support for multiple output formats.
//! https://github.com/yasm/yasm

use crate::config::YasmUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use vx_runtime::{
    Arch, Ecosystem, ExecutableLayout, Os, Platform, Runtime, RuntimeContext,
    VerificationResult, VersionInfo,
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

    fn executable_layout(&self) -> Option<ExecutableLayout> {
        // Parse the layout from our embedded provider.toml
        // For now, we'll define it in code (later it will be parsed from manifest)
        use vx_runtime::BinaryLayout;
        
        let mut binary_configs = std::collections::HashMap::new();
        
        binary_configs.insert(
            "windows-x86_64".to_string(),
            BinaryLayout {
                source_name: "yasm-{version}-win64.exe".to_string(),
                target_name: "yasm.exe".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: None,
            },
        );
        
        binary_configs.insert(
            "windows-x86".to_string(),
            BinaryLayout {
                source_name: "yasm-{version}-win32.exe".to_string(),
                target_name: "yasm.exe".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: None,
            },
        );
        
        binary_configs.insert(
            "macos-x86_64".to_string(),
            BinaryLayout {
                source_name: "yasm-{version}-macos".to_string(),
                target_name: "yasm".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: Some("755".to_string()),
            },
        );
        
        binary_configs.insert(
            "linux-x86_64".to_string(),
            BinaryLayout {
                source_name: "yasm-{version}-linux".to_string(),
                target_name: "yasm".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: Some("755".to_string()),
            },
        );
        
        Some(ExecutableLayout {
            download_type: vx_runtime::DownloadType::Binary,
            binary: Some(binary_configs),
            archive: None,
            msi: None,
            windows: None,
            macos: None,
            linux: None,
        })
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

    fn post_extract(&self, version: &str, install_path: &PathBuf) -> Result<()> {
        // Rename downloaded file to standard name
        // Original: bin/yasm-1.3.0-win64.exe
        // Standard: bin/yasm.exe
        use std::fs;
        
        let platform = Platform::current();
        
        let original_name = match (&platform.os, &platform.arch) {
            (Os::Windows, Arch::X86_64) => format!("yasm-{}-win64.exe", version),
            (Os::Windows, Arch::X86) => format!("yasm-{}-win32.exe", version),
            (Os::MacOS, _) => format!("yasm-{}-macos", version),
            (Os::Linux, _) => format!("yasm-{}-linux", version),
            _ => return Ok(()),
        };
        
        let bin_dir = install_path.join("bin");
        fs::create_dir_all(&bin_dir)?;
        
        let original_path = bin_dir.join(&original_name);
        let standard_name = YasmUrlBuilder::get_executable_name(&platform);
        let standard_path = bin_dir.join(&standard_name);
        
        // Rename the file to standard name
        if original_path.exists() {
            if standard_path.exists() {
                fs::remove_file(&standard_path)?;
            }
            fs::rename(&original_path, &standard_path)?;
            
            // Set executable permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&standard_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&standard_path, perms)?;
            }
        }
        
        Ok(())
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        // Check for standard executable name (after post_extract renamed it)
        let exe_name = YasmUrlBuilder::get_executable_name(platform);
        let exe_path = install_path.join("bin").join(&exe_name);
        
        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "Executable not found at any expected path:\n  - {}",
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
