//! React Native framework detector
//! Detects React Native projects via package.json, app.json, or platform folders.

use super::{FrameworkDetector, FrameworkInfo, ProjectFramework};
use crate::dependency::{Dependency, InstallMethod};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use crate::types::{RequiredTool, Script, ScriptSource};
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;
use tracing::debug;

/// React Native framework detector
pub struct ReactNativeDetector;

impl ReactNativeDetector {
    /// Create a new detector
    pub fn new() -> Self {
        Self
    }

    fn read_package_json(root: &Path) -> Option<Value> {
        let path = root.join("package.json");
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str::<Value>(&content).ok()
    }

    fn has_react_native_dependency(pkg: &Value) -> bool {
        let check = |key: &str| -> bool {
            pkg.get(key)
                .and_then(|v| v.as_object())
                .is_some_and(|deps| deps.contains_key("react-native") || deps.contains_key("expo"))
        };

        check("dependencies") || check("devDependencies")
    }

    fn get_react_native_version(pkg: &Value) -> Option<String> {
        let get_version = |deps: Option<&Value>| -> Option<String> {
            deps.and_then(|v| v.get("react-native"))
                .and_then(|v| v.as_str())
                .map(|s| s.trim_start_matches(['^', '~']).to_string())
        };

        get_version(pkg.get("devDependencies")).or_else(|| get_version(pkg.get("dependencies")))
    }

    fn detect_platform_dirs(root: &Path, info: &mut FrameworkInfo) {
        if root.join("android").is_dir() {
            info.target_platforms.push("android".to_string());
        }
        if root.join("ios").is_dir() {
            info.target_platforms.push("ios".to_string());
        }
        if root.join("macos").is_dir() {
            info.target_platforms.push("macos".to_string());
        }
        if root.join("windows").is_dir() {
            info.target_platforms.push("windows".to_string());
        }
    }
}

impl Default for ReactNativeDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FrameworkDetector for ReactNativeDetector {
    fn detect(&self, root: &Path) -> bool {
        // package.json with react-native or expo
        if let Some(pkg) = Self::read_package_json(root) {
            if Self::has_react_native_dependency(&pkg) {
                debug!("Detected React Native via package.json dependency");
                return true;
            }
        }

        // app.json presence (metro apps) or platform folders
        if root.join("app.json").exists()
            || root.join("metro.config.js").exists()
            || root.join("metro.config.ts").exists()
            || root.join("react-native.config.js").exists()
        {
            debug!("Detected React Native via config file");
            return true;
        }

        if root.join("android").is_dir() || root.join("ios").is_dir() {
            debug!("Detected React Native via platform directories");
            return true;
        }

        false
    }

    fn framework(&self) -> ProjectFramework {
        ProjectFramework::ReactNative
    }

    async fn get_info(&self, root: &Path) -> AnalyzerResult<FrameworkInfo> {
        let mut info = FrameworkInfo::new(ProjectFramework::ReactNative);

        if let Some(pkg) = Self::read_package_json(root) {
            if let Some(version) = Self::get_react_native_version(&pkg) {
                info = info.with_version(version);
            }

            if let Some(scripts) = pkg.get("scripts").and_then(|v| v.as_object()) {
                // capture known RN scripts as metadata
                for key in ["android", "ios", "start", "bundle", "test"] {
                    if scripts.contains_key(key) {
                        info = info.with_metadata("has_script", key);
                    }
                }
            }
        }

        // Determine config path preference
        for config in [
            "app.json",
            "react-native.config.js",
            "metro.config.js",
            "metro.config.ts",
        ] {
            let p = root.join(config);
            if p.exists() {
                info = info.with_config_path(p);
                break;
            }
        }

        Self::detect_platform_dirs(root, &mut info);

        // CLI/build tool
        info = info.with_build_tool("react-native-cli");

        Ok(info)
    }

    fn required_tools(&self, _deps: &[Dependency], scripts: &[Script]) -> Vec<RequiredTool> {
        let mut tools = vec![RequiredTool::new(
            "node",
            Ecosystem::NodeJs,
            "Node.js runtime for React Native",
            InstallMethod::vx("node"),
        )];

        // React Native CLI is often invoked via npx, still recommend installing
        let uses_rn_cli = scripts.iter().any(|s| s.command.contains("react-native"));
        if uses_rn_cli {
            tools.push(RequiredTool::new(
                "react-native",
                Ecosystem::NodeJs,
                "React Native CLI",
                InstallMethod::npm_dev("react-native"),
            ));
        }

        tools
    }

    async fn additional_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>> {
        let mut scripts = Vec::new();
        let pkg = Self::read_package_json(root);

        if let Some(pkg) = pkg {
            if let Some(pkg_scripts) = pkg.get("scripts").and_then(|s| s.as_object()) {
                let rn_scripts = [
                    ("android", "Run Android build"),
                    ("ios", "Run iOS build"),
                    ("start", "Start Metro bundler"),
                    ("bundle", "Bundle React Native assets"),
                ];

                for (name, description) in rn_scripts {
                    if let Some(cmd) = pkg_scripts.get(name).and_then(|v| v.as_str()) {
                        let mut script = Script::new(name, cmd, ScriptSource::PackageJson);
                        script.description = Some(description.to_string());
                        scripts.push(script);
                    }
                }
            }
        }

        Ok(scripts)
    }
}
