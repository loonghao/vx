//! Tauri framework detector
//!
//! Detects Tauri desktop applications by checking for:
//! - `tauri.conf.json` or `Tauri.toml` configuration files
//! - `@tauri-apps/cli` or `tauri-cli` dependencies
//! - `src-tauri/` directory structure

use super::{FrameworkDetector, FrameworkInfo, ProjectFramework};
use crate::dependency::{Dependency, InstallMethod};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use crate::types::{RequiredTool, Script, ScriptSource};
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;
use tracing::debug;

/// Tauri framework detector
pub struct TauriDetector;

impl TauriDetector {
    /// Create a new Tauri detector
    pub fn new() -> Self {
        Self
    }

    /// Get the src-tauri directory path
    fn get_tauri_dir(root: &Path) -> Option<std::path::PathBuf> {
        let src_tauri = root.join("src-tauri");
        if src_tauri.exists() && src_tauri.is_dir() {
            return Some(src_tauri);
        }

        // Some projects might have tauri config in root
        if root.join("tauri.conf.json").exists() || root.join("Tauri.toml").exists() {
            return Some(root.to_path_buf());
        }

        None
    }

    /// Check package.json for Tauri dependencies
    fn has_tauri_dependency(package_json: &Value) -> bool {
        let check_deps = |deps: Option<&Value>| -> bool {
            deps.and_then(|d| d.as_object()).is_some_and(|obj| {
                obj.contains_key("@tauri-apps/cli")
                    || obj.contains_key("@tauri-apps/api")
                    || obj.contains_key("tauri")
            })
        };

        check_deps(package_json.get("dependencies"))
            || check_deps(package_json.get("devDependencies"))
    }

    /// Get Tauri CLI version from package.json
    fn get_tauri_cli_version(package_json: &Value) -> Option<String> {
        let get_version = |deps: Option<&Value>| -> Option<String> {
            deps.and_then(|d| d.get("@tauri-apps/cli"))
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

    /// Parse tauri.conf.json for configuration details
    async fn parse_tauri_config(config_path: &Path) -> Option<Value> {
        if !config_path.exists() {
            return None;
        }

        let content = tokio::fs::read_to_string(config_path).await.ok()?;

        // Handle JSON5 format (tauri.conf.json5)
        if config_path.extension().is_some_and(|ext| ext == "json5") {
            // For now, try standard JSON parsing (most JSON5 is valid JSON)
            serde_json::from_str(&content).ok()
        } else {
            serde_json::from_str(&content).ok()
        }
    }

    /// Detect Tauri version (v1 vs v2)
    fn detect_tauri_version(tauri_dir: &Path, package_json: Option<&Value>) -> Option<String> {
        // Check for Tauri v2 indicators
        // v2 uses tauri.conf.json with different structure or Tauri.toml
        if tauri_dir.join("Tauri.toml").exists() {
            return Some("2.x".to_string());
        }

        // Check package.json for @tauri-apps/cli version
        if let Some(pkg) = package_json {
            if let Some(version) = Self::get_tauri_cli_version(pkg) {
                // Parse major version
                if version.starts_with("2") {
                    return Some("2.x".to_string());
                } else if version.starts_with("1") {
                    return Some("1.x".to_string());
                }
            }
        }

        // Check Cargo.toml in src-tauri for tauri dependency version
        let cargo_toml = tauri_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                if let Ok(toml) = content.parse::<toml::Value>() {
                    if let Some(deps) = toml.get("dependencies") {
                        if let Some(tauri_dep) = deps.get("tauri") {
                            let version_str = match tauri_dep {
                                toml::Value::String(v) => Some(v.as_str()),
                                toml::Value::Table(t) => t.get("version").and_then(|v| v.as_str()),
                                _ => None,
                            };

                            if let Some(v) = version_str {
                                if v.starts_with("2") {
                                    return Some("2.x".to_string());
                                } else if v.starts_with("1") {
                                    return Some("1.x".to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }
}

impl Default for TauriDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FrameworkDetector for TauriDetector {
    fn detect(&self, root: &Path) -> bool {
        // Check for src-tauri directory
        let src_tauri = root.join("src-tauri");
        if src_tauri.exists() && src_tauri.is_dir() {
            // Verify it's a Tauri project by checking for config or Cargo.toml
            if src_tauri.join("tauri.conf.json").exists()
                || src_tauri.join("tauri.conf.json5").exists()
                || src_tauri.join("Tauri.toml").exists()
                || src_tauri.join("Cargo.toml").exists()
            {
                debug!("Detected Tauri project via src-tauri directory");
                return true;
            }
        }

        // Check for tauri config in root (less common)
        if root.join("tauri.conf.json").exists() || root.join("Tauri.toml").exists() {
            debug!("Detected Tauri project via root config file");
            return true;
        }

        // Check package.json for Tauri dependencies
        let package_json_path = root.join("package.json");
        if package_json_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&package_json_path) {
                if let Ok(package_json) = serde_json::from_str::<Value>(&content) {
                    if Self::has_tauri_dependency(&package_json) {
                        debug!("Detected Tauri project via package.json dependency");
                        return true;
                    }
                }
            }
        }

        false
    }

    fn framework(&self) -> ProjectFramework {
        ProjectFramework::Tauri
    }

    async fn get_info(&self, root: &Path) -> AnalyzerResult<FrameworkInfo> {
        let mut info = FrameworkInfo::new(ProjectFramework::Tauri);

        let tauri_dir = Self::get_tauri_dir(root);

        // Read package.json for CLI version
        let package_json_path = root.join("package.json");
        let package_json = if package_json_path.exists() {
            let content = tokio::fs::read_to_string(&package_json_path).await?;
            serde_json::from_str::<Value>(&content).ok()
        } else {
            None
        };

        // Detect Tauri version
        if let Some(ref tauri_dir) = tauri_dir {
            if let Some(version) = Self::detect_tauri_version(tauri_dir, package_json.as_ref()) {
                info = info.with_version(version);
            }
        }

        // Find and parse tauri config
        let config_paths = [
            root.join("src-tauri/tauri.conf.json"),
            root.join("src-tauri/tauri.conf.json5"),
            root.join("src-tauri/Tauri.toml"),
            root.join("tauri.conf.json"),
            root.join("Tauri.toml"),
        ];

        for config_path in &config_paths {
            if config_path.exists() {
                info = info.with_config_path(config_path.clone());

                // Parse JSON config for additional info
                if config_path
                    .extension()
                    .is_some_and(|ext| ext == "json" || ext == "json5")
                {
                    if let Some(config) = Self::parse_tauri_config(config_path).await {
                        // Get product name
                        if let Some(name) = config
                            .get("productName")
                            .or_else(|| config.get("package").and_then(|p| p.get("productName")))
                            .and_then(|v| v.as_str())
                        {
                            info = info.with_metadata("productName", name);
                        }

                        // Get identifier
                        if let Some(identifier) = config
                            .get("identifier")
                            .or_else(|| {
                                config
                                    .get("tauri")
                                    .and_then(|t| t.get("bundle"))
                                    .and_then(|b| b.get("identifier"))
                            })
                            .and_then(|v| v.as_str())
                        {
                            info = info.with_metadata("identifier", identifier);
                        }

                        // Get target platforms from bundle config
                        if let Some(targets) = config
                            .get("tauri")
                            .and_then(|t| t.get("bundle"))
                            .and_then(|b| b.get("targets"))
                            .and_then(|t| t.as_array())
                        {
                            for target in targets {
                                if let Some(t) = target.as_str() {
                                    info = info.with_platform(t);
                                }
                            }
                        }
                    }
                }

                break;
            }
        }

        // Set build tool
        info = info.with_build_tool("tauri-cli");

        Ok(info)
    }

    fn required_tools(&self, _deps: &[Dependency], scripts: &[Script]) -> Vec<RequiredTool> {
        let mut tools = Vec::new();

        // Tauri projects need Rust
        tools.push(RequiredTool::new(
            "rust",
            Ecosystem::Rust,
            "Rust toolchain for Tauri backend",
            InstallMethod::vx("rust"),
        ));

        // Tauri projects also need Node.js for the frontend
        tools.push(RequiredTool::new(
            "node",
            Ecosystem::NodeJs,
            "Node.js runtime for Tauri frontend",
            InstallMethod::vx("node"),
        ));

        // Check for tauri-cli usage in scripts
        let needs_cli = scripts
            .iter()
            .any(|s| s.command.contains("tauri") || s.command.contains("@tauri-apps/cli"));

        if needs_cli {
            tools.push(RequiredTool::new(
                "tauri-cli",
                Ecosystem::NodeJs,
                "Tauri CLI for building and development",
                InstallMethod::npm_dev("@tauri-apps/cli"),
            ));
        }

        // Platform-specific requirements
        #[cfg(target_os = "linux")]
        {
            tools.push(RequiredTool::new(
                "webkit2gtk",
                Ecosystem::Unknown,
                "WebKit2GTK for Tauri on Linux",
                InstallMethod::System {
                    instructions: "Install via system package manager: apt install libwebkit2gtk-4.1-dev (Ubuntu/Debian)".to_string(),
                },
            ));
        }

        tools
    }

    async fn additional_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>> {
        let mut scripts = Vec::new();

        // Check package.json for Tauri-specific scripts
        let package_json_path = root.join("package.json");
        if package_json_path.exists() {
            let content = tokio::fs::read_to_string(&package_json_path).await?;
            if let Ok(package_json) = serde_json::from_str::<Value>(&content) {
                if let Some(pkg_scripts) = package_json.get("scripts").and_then(|s| s.as_object()) {
                    // Common Tauri script patterns
                    let tauri_patterns = [
                        ("tauri", "Run Tauri CLI"),
                        ("tauri:dev", "Start Tauri in development mode"),
                        ("tauri:build", "Build Tauri application"),
                        ("tauri:debug", "Build Tauri in debug mode"),
                    ];

                    for (name, description) in tauri_patterns {
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
