//! Bun framework detector
//!
//! Detects Bun applications by checking for:
//! - `bun.lockb` lock file (Bun's binary lock file)
//! - `bunfig.toml` configuration file
//! - TypeScript/JavaScript files with Bun imports
//!
//! Bun is an all-in-one JavaScript runtime with:
//! - Built-in bundler, test runner, and package manager
//! - Node.js-compatible API with better performance
//! - Native TypeScript support

use super::{FrameworkDetector, FrameworkInfo, ProjectFramework};
use crate::dependency::Dependency;
use crate::error::AnalyzerResult;
use crate::types::{RequiredTool, Script, ScriptSource};
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;
use tracing::debug;

/// Bun framework detector
#[allow(clippy::vec_init_then_push)]
#[allow(clippy::collapsible_if)]
pub struct BunDetector;

impl BunDetector {
    /// Create a new Bun detector
    pub fn new() -> Self {
        Self
    }

    /// Check for Bun lock file
    fn has_bun_lockb(root: &Path) -> bool {
        root.join("bun.lockb").exists() || root.join("bun.lock").exists()
    }

    /// Check for Bun configuration files
    fn has_bun_config(root: &Path) -> bool {
        root.join("bunfig.toml").exists()
            || root.join("bunfig.toml5").exists()
    }

    /// Check for Bun-specific files
    fn has_bun_files(root: &Path) -> bool {
        let bun_files = [
            "bunfig.toml",
            "bunfig.toml5",
            "bun.lockb",
            "bun.lock",
        ];

        bun_files.iter().any(|f| root.join(f).exists())
    }

    /// Detect Bun-specific imports in TypeScript/JavaScript files
    fn has_bun_imports(root: &Path) -> bool {
        // Check common entry point files
        let check_files = [
            "index.ts",
            "index.js",
            "main.ts",
            "main.js",
            "app.ts",
            "app.js",
            "server.ts",
            "server.js",
        ];

        for file in &check_files {
            let path = root.join(file);
            if !path.exists() {
                continue;
            }

            // Read first few lines to check for Bun imports
            if let Ok(content) = std::fs::read_to_string(&path) {
                // Bun-specific import patterns
                if content.contains("bun:") || content.contains("from 'bun:") {
                    return true;
                }
            }
        }

        false
    }
}

impl Default for BunDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FrameworkDetector for BunDetector {
    fn detect(&self, root: &Path) -> bool {
        // Check for Bun lock file
        if Self::has_bun_lockb(root) {
            debug!("Detected Bun project via lock file");
            return true;
        }

        // Check for Bun-specific files
        if Self::has_bun_files(root) {
            debug!("Detected Bun project via special files");
            return true;
        }

        // Check for Bun configuration
        if Self::has_bun_config(root) {
            debug!("Detected Bun project via configuration file");
            return true;
        }

        // Check for Bun-specific imports
        if Self::has_bun_imports(root) {
            debug!("Detected Bun project via Bun imports");
            return true;
        }

        false
    }

    fn framework(&self) -> ProjectFramework {
        ProjectFramework::Bun
    }

    async fn get_info(&self, root: &Path) -> AnalyzerResult<FrameworkInfo> {
        let mut info = FrameworkInfo::new(ProjectFramework::Bun);

        // Check for bunfig.toml
        let bunfig_path = root.join("bunfig.toml");
        if bunfig_path.exists() {
            info = info.with_config_path(bunfig_path);
        }

        // Check for bun.lockb (binary lock file)
        if root.join("bun.lockb").exists() {
            info = info.with_metadata("lock_file", "bun.lockb (binary)");
        } else if root.join("bun.lock").exists() {
            info = info.with_metadata("lock_file", "bun.lock (text)");
        }

        // Check for TypeScript configuration
        if root.join("tsconfig.json").exists() {
            info = info.with_metadata("typescript", "true");
        }

        Ok(info)
    }

    fn required_tools(&self, _deps: &[Dependency], _scripts: &[Script]) -> Vec<RequiredTool> {
        vec![
            RequiredTool::new(
                "bun",
                crate::ecosystem::Ecosystem::NodeJs,
                "Bun runtime for JavaScript/TypeScript",
                crate::dependency::InstallMethod::Vx { tool: "bun".to_string(), version: None },
            ),
        ]
    }

    async fn additional_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>> {
        let mut scripts = Vec::new();

        // Check package.json for Bun-specific scripts
        let package_json_path = root.join("package.json");
        if package_json_path.exists() {
            let content = tokio::fs::read_to_string(&package_json_path).await?;
            if let Ok(package_json) = serde_json::from_str::<Value>(&content)
                && let Some(scripts_obj) = package_json
                    .get("scripts")
                    .and_then(|s| s.as_object())
            {
                    // Common Bun script patterns
                    let bun_patterns = [
                        ("bun:dev", "Start Bun in development mode"),
                        ("bun:start", "Start Bun production server"),
                        ("bun:test", "Run tests with Bun"),
                        ("bun:run", "Run script with Bun"),
                    ];

                    for (name, description) in bun_patterns {
                        if let Some(command) = scripts_obj.get(name).and_then(|v| v.as_str()) {
                            let mut script = Script::new(name, command, ScriptSource::PackageJson);
                            script.description = Some(description.to_string());
                            scripts.push(script);
                        }
                    }
                }
            }
        }

        Ok(scripts)
    }
}
