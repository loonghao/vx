//! Zig framework detector
//!
//! Detects Zig projects by checking for:
//! - `build.zig` (Zig build file)
//! - `.zig-version` (Zig version file)
//! - `zig-out/` (Zig build output directory)
//!
//! Zig is a general-purpose programming language:
//! - Performance comparable to C/C++
//! - Memory safety without garbage collection
//! - First-class cross-compilation support

use super::{FrameworkDetector, FrameworkInfo, ProjectFramework};
use crate::dependency::Dependency;
use crate::error::AnalyzerResult;
use crate::types::{RequiredTool, Script, ScriptSource};
use async_trait::async_trait;
use std::path::Path;
use tracing::debug;

/// Zig framework detector
#[allow(clippy::needless_borrow)]
pub struct ZigDetector;

impl ZigDetector {
    /// Create a new Zig detector
    pub fn new() -> Self {
        Self
    }

    /// Check for Zig build files
    fn has_zig_build(root: &Path) -> bool {
        root.join("build.zig").exists()
    }

    /// Check for Zig version file
    fn has_zig_version(root: &Path) -> bool {
        root.join(".zig-version").exists()
    }

    /// Check for Zig build output
    fn has_zig_output(root: &Path) -> bool {
        root.join("zig-out").exists() || root.join("zig-cache").exists()
    }

    /// Detect Zig version from .zig-version
    fn detect_zig_version(root: &Path) -> Option<String> {
        let version_file = root.join(".zig-version");
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
}

impl Default for ZigDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FrameworkDetector for ZigDetector {
    fn detect(&self, root: &Path) -> bool {
        // Check for Zig build file
        if Self::has_zig_build(root) {
            debug!("Detected Zig project via build.zig");
            return true;
        }

        // Check for Zig version file
        if Self::has_zig_version(root) {
            debug!("Detected Zig project via .zig-version");
            return true;
        }

        // Check for Zig build output
        if Self::has_zig_output(root) {
            debug!("Detected Zig project via build output");
            return true;
        }

        false
    }

    fn framework(&self) -> ProjectFramework {
        ProjectFramework::Zig
    }

    async fn get_info(&self, root: &Path) -> AnalyzerResult<FrameworkInfo> {
        let mut info = FrameworkInfo::new(ProjectFramework::Zig);

        // Detect version
        if let Some(version) = Self::detect_zig_version(root) {
            info = info.with_version(version);
        }

        // Check for build.zig
        if Self::has_zig_build(root) {
            info = info.with_config_path(root.join("build.zig"));
        }

        // Check for .zig-version
        if Self::has_zig_version(root) {
            info = info.with_metadata("version_file", ".zig-version");
        }

        // Check for cross-compilation targets
        let zig_targets = [
            "x86_64-linux",
            "aarch64-linux",
            "x86_64-windows",
            "aarch64-windows",
            "wasm32-freestanding",
        ];
        for target in &zig_targets {
            let target_file = root.join(format!("zig-cache{}", target));
            if target_file.exists() {
                info = info.with_platform(target.to_string());
            }
        }

        Ok(info)
    }

    fn required_tools(&self, _deps: &[Dependency], _scripts: &[Script]) -> Vec<RequiredTool> {
        vec![
            RequiredTool::new(
                "zig",
                crate::ecosystem::Ecosystem::Zig,
                "Zig compiler for general-purpose programming",
                crate::dependency::InstallMethod::Vx {
                    tool: "zig".to_string(),
                    version: None,
                },
            ),
        ]
    }

    async fn additional_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>> {
        let mut scripts = Vec::new();

        // Check build.zig for common build/test/run commands
        let build_zig_path = root.join("build.zig");
        if build_zig_path.exists() {
            let content = std::fs::read_to_string(&build_zig_path)?;

            // Common Zig build steps
            let zig_steps = [
                ("build", "Build Zig project"),
                ("test", "Run Zig tests"),
                ("run", "Run Zig application"),
                ("install", "Install Zig project"),
            ];

            for (step, _description) in zig_steps.iter() {
                if content.contains(step) {
                    let script = Script::new(
                        format!("zig:{}", step),
                        format!("zig build {}", step),
                        ScriptSource::BuildZig,
                    );
                    scripts.push(script);
                }
            }
        }

        Ok(scripts)
    }
}
