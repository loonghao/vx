//! Nix framework detector
//!
//! Detects Nix projects by checking for:
//! - `flake.nix` (Nix Flake configuration)
//! - `default.nix` (Traditional Nix configuration)
//! - `shell.nix` (Development shell configuration)
//! - `.nix-version` (Nix version file)
//!
//! Nix is a package manager and build system:
//! - Reproducible builds and deployments
//! - Functional package management
//! - Cross-platform (Linux, macOS, WSL)

use super::{FrameworkDetector, FrameworkInfo, ProjectFramework};
use crate::dependency::Dependency;
use crate::error::AnalyzerResult;
use crate::types::{RequiredTool, Script, ScriptSource};
use async_trait::async_trait;
use std::path::Path;
use tracing::debug;

/// Nix framework detector
pub struct NixDetector;

impl NixDetector {
    /// Create a new Nix detector
    pub fn new() -> Self {
        Self
    }

    /// Check for Nix configuration files
    fn has_nix_config(root: &Path) -> bool {
        root.join("flake.nix").exists()
            || root.join("default.nix").exists()
            || root.join("shell.nix").exists()
    }

    /// Check for Nix version file
    fn has_nix_version(root: &Path) -> bool {
        root.join(".nix-version").exists()
    }

    /// Detect Nix version from .nix-version
    fn detect_nix_version(root: &Path) -> Option<String> {
        let version_file = root.join(".nix-version");
        if version_file.exists()
            && let Ok(content) = std::fs::read_to_string(&version_file)
        {
            let version = content.trim().to_string();
            if !version.is_empty() {
                return Some(version);
            }
        }
        None
    }

    /// Check for Nix-specific shell files
    fn has_nix_shell_files(root: &Path) -> bool {
        let nix_files = ["shell.nix", "dev-shell.nix", "flake.nix"];

        nix_files.iter().any(|f| root.join(f).exists())
    }
}

impl Default for NixDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FrameworkDetector for NixDetector {
    fn detect(&self, root: &Path) -> bool {
        // Check for Nix configuration files
        if Self::has_nix_config(root) {
            debug!("Detected Nix project via configuration file");
            return true;
        }

        // Check for Nix version file
        if Self::has_nix_version(root) {
            debug!("Detected Nix project via version file");
            return true;
        }

        // Check for Nix shell files
        if Self::has_nix_shell_files(root) {
            debug!("Detected Nix project via shell files");
            return true;
        }

        false
    }

    fn framework(&self) -> ProjectFramework {
        ProjectFramework::Nix
    }

    async fn get_info(&self, root: &Path) -> AnalyzerResult<FrameworkInfo> {
        let mut info = FrameworkInfo::new(ProjectFramework::Nix);

        // Detect version
        if let Some(version) = Self::detect_nix_version(root) {
            info = info.with_version(version);
        }

        // Check for flake.nix
        if root.join("flake.nix").exists() {
            info = info.with_config_path(root.join("flake.nix"));
            info = info.with_metadata("flake", "true");
        }

        // Check for shell.nix
        if root.join("shell.nix").exists() {
            info = info.with_metadata("shell", "true");
        }

        // Check for default.nix
        if root.join("default.nix").exists() {
            info = info.with_metadata("traditional", "true");
        }

        Ok(info)
    }

    fn required_tools(&self, _deps: &[Dependency], _scripts: &[Script]) -> Vec<RequiredTool> {
        vec![RequiredTool::new(
            "nix",
            crate::ecosystem::Ecosystem::Nix,
            "Nix package manager and build system",
            crate::dependency::InstallMethod::Vx {
                tool: "nix".to_string(),
                version: None,
            },
        )]
    }

    async fn additional_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>> {
        let mut scripts = Vec::new();

        // Check for common Nix commands in flake.nix
        let flake_path = root.join("flake.nix");
        if flake_path.exists() {
            let content = std::fs::read_to_string(&flake_path)?;

            // Common Nix flake outputs
            let nix_patterns = [
                ("build", "Build Nix flake"),
                ("check", "Run Nix checks"),
                ("devShell", "Enter development shell"),
                ("package", "Build package"),
                ("run", "Run application"),
            ];

            for (name, _description) in nix_patterns.iter() {
                if content.contains(name) {
                    let script = Script::new(
                        format!("nix:{}", name),
                        format!("nix build .#{}", name),
                        ScriptSource::BuildNix,
                    );
                    scripts.push(script);
                }
            }
        }

        Ok(scripts)
    }
}
