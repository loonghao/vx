//! Electron framework detector
//!
//! Detects Electron desktop applications by checking for:
//! - `electron` dependency in package.json
//! - Electron-specific configuration files (electron-builder, electron-forge)
//! - Electron Vite configuration

use super::{FrameworkDetector, FrameworkInfo, ProjectFramework};
use crate::dependency::{Dependency, InstallMethod};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use crate::types::{RequiredTool, Script, ScriptSource};
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;
use tracing::debug;

/// Electron framework detector
pub struct ElectronDetector;

impl ElectronDetector {
    /// Create a new Electron detector
    pub fn new() -> Self {
        Self
    }

    /// Check package.json for Electron dependency
    fn has_electron_dependency(package_json: &Value) -> bool {
        let check_deps = |deps: Option<&Value>| -> bool {
            deps.and_then(|d| d.as_object())
                .is_some_and(|obj| obj.contains_key("electron"))
        };

        check_deps(package_json.get("dependencies"))
            || check_deps(package_json.get("devDependencies"))
    }

    /// Get Electron version from package.json
    fn get_electron_version(package_json: &Value) -> Option<String> {
        let get_version = |deps: Option<&Value>| -> Option<String> {
            deps.and_then(|d| d.get("electron"))
                .and_then(|v| v.as_str())
                .map(|s| {
                    s.trim_start_matches('^')
                        .trim_start_matches('~')
                        .to_string()
                })
        };

        get_version(package_json.get("devDependencies"))
            .or_else(|| get_version(package_json.get("dependencies")))
    }

    /// Detect build tool from project files
    fn detect_build_tool(root: &Path, package_json: &Value) -> Option<String> {
        // Check for electron-builder
        if root.join("electron-builder.json").exists()
            || root.join("electron-builder.yml").exists()
            || root.join("electron-builder.yaml").exists()
            || root.join("builder-debug.config.ts").exists()
        {
            return Some("electron-builder".to_string());
        }

        // Check for electron-forge
        if root.join("forge.config.js").exists() || root.join("forge.config.ts").exists() {
            return Some("electron-forge".to_string());
        }

        // Check devDependencies
        if let Some(dev_deps) = package_json
            .get("devDependencies")
            .and_then(|d| d.as_object())
        {
            if dev_deps.contains_key("electron-builder") {
                return Some("electron-builder".to_string());
            }
            if dev_deps.contains_key("@electron-forge/cli") {
                return Some("electron-forge".to_string());
            }
            if dev_deps.contains_key("electron-vite") {
                return Some("electron-vite".to_string());
            }
        }

        None
    }

    /// Check for electron-vite configuration
    fn has_electron_vite(root: &Path) -> bool {
        root.join("electron.vite.config.js").exists()
            || root.join("electron.vite.config.ts").exists()
            || root.join("electron.vite.config.mjs").exists()
    }

    /// Check for todesktop configuration (Electron distribution service)
    fn has_todesktop(root: &Path) -> bool {
        root.join("todesktop.json").exists() || root.join("todesktop.staging.json").exists()
    }
}

impl Default for ElectronDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FrameworkDetector for ElectronDetector {
    fn detect(&self, root: &Path) -> bool {
        let package_json_path = root.join("package.json");
        if !package_json_path.exists() {
            return false;
        }

        // Read and parse package.json
        let content = match std::fs::read_to_string(&package_json_path) {
            Ok(c) => c,
            Err(_) => return false,
        };

        let package_json: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => return false,
        };

        // Check for electron dependency
        if Self::has_electron_dependency(&package_json) {
            debug!("Detected Electron project via package.json dependency");
            return true;
        }

        // Check for electron-specific config files
        for indicator in ProjectFramework::Electron.indicator_files() {
            if root.join(indicator).exists() {
                debug!("Detected Electron project via config file: {}", indicator);
                return true;
            }
        }

        false
    }

    fn framework(&self) -> ProjectFramework {
        ProjectFramework::Electron
    }

    async fn get_info(&self, root: &Path) -> AnalyzerResult<FrameworkInfo> {
        let mut info = FrameworkInfo::new(ProjectFramework::Electron);

        // Read package.json
        let package_json_path = root.join("package.json");
        if package_json_path.exists() {
            let content = tokio::fs::read_to_string(&package_json_path).await?;
            if let Ok(package_json) = serde_json::from_str::<Value>(&content) {
                // Get Electron version
                if let Some(version) = Self::get_electron_version(&package_json) {
                    info = info.with_version(version);
                }

                // Detect build tool
                if let Some(build_tool) = Self::detect_build_tool(root, &package_json) {
                    info = info.with_build_tool(&build_tool);
                }

                // Get product name if available
                if let Some(name) = package_json.get("productName").and_then(|v| v.as_str()) {
                    info = info.with_metadata("productName", name);
                }
            }
        }

        // Check for electron-vite
        if Self::has_electron_vite(root) {
            info = info.with_metadata("bundler", "electron-vite");
        }

        // Check for todesktop
        if Self::has_todesktop(root) {
            info = info.with_metadata("distribution", "todesktop");
        }

        // Detect config file path
        for config_file in &[
            "electron-builder.json",
            "electron-builder.yml",
            "electron-builder.yaml",
            "builder-debug.config.ts",
            "forge.config.js",
            "forge.config.ts",
        ] {
            let config_path = root.join(config_file);
            if config_path.exists() {
                info = info.with_config_path(config_path);
                break;
            }
        }

        Ok(info)
    }

    fn required_tools(&self, _deps: &[Dependency], scripts: &[Script]) -> Vec<RequiredTool> {
        let mut tools = Vec::new();

        // Electron projects always need Node.js
        tools.push(RequiredTool::new(
            "node",
            Ecosystem::NodeJs,
            "Node.js runtime for Electron",
            InstallMethod::vx("node"),
        ));

        // Check for electron-builder in scripts
        let needs_builder = scripts
            .iter()
            .any(|s| s.command.contains("electron-builder") || s.command.contains("build --"));

        if needs_builder {
            tools.push(RequiredTool::new(
                "electron-builder",
                Ecosystem::NodeJs,
                "Electron application packager",
                InstallMethod::npm_dev("electron-builder"),
            ));
        }

        // Check for electron-forge in scripts
        let needs_forge = scripts.iter().any(|s| {
            s.command.contains("electron-forge")
                || s.command.contains("forge make")
                || s.command.contains("forge package")
        });

        if needs_forge {
            tools.push(RequiredTool::new(
                "electron-forge",
                Ecosystem::NodeJs,
                "Electron application toolkit",
                InstallMethod::npm_dev("@electron-forge/cli"),
            ));
        }

        tools
    }

    async fn additional_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>> {
        let mut scripts = Vec::new();

        // Check package.json for Electron-specific scripts
        let package_json_path = root.join("package.json");
        if package_json_path.exists() {
            let content = tokio::fs::read_to_string(&package_json_path).await?;
            if let Ok(package_json) = serde_json::from_str::<Value>(&content) {
                if let Some(pkg_scripts) = package_json.get("scripts").and_then(|s| s.as_object()) {
                    // Common Electron script patterns
                    let electron_patterns = [
                        ("electron:dev", "Start Electron in development mode"),
                        ("electron:build", "Build Electron application"),
                        ("electron:pack", "Package Electron application"),
                        ("electron:dist", "Distribute Electron application"),
                        ("make", "Build distributable packages"),
                        ("package", "Package the application"),
                        ("publish", "Publish the application"),
                    ];

                    for (name, description) in electron_patterns {
                        if let Some(command) = pkg_scripts.get(name).and_then(|v| v.as_str()) {
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
