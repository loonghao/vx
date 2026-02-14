//! Capacitor framework detector
//! Detects Capacitor hybrid apps via dependencies and config files.

use super::{FrameworkDetector, FrameworkInfo, ProjectFramework};
use crate::dependency::{Dependency, InstallMethod};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use crate::types::{RequiredTool, Script, ScriptSource};
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;
use tracing::debug;

pub struct CapacitorDetector;

impl CapacitorDetector {
    pub fn new() -> Self {
        Self
    }

    fn read_package_json(root: &Path) -> Option<Value> {
        let path = root.join("package.json");
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str::<Value>(&content).ok()
    }

    fn has_capacitor_dependency(pkg: &Value) -> bool {
        let check = |key: &str| -> bool {
            pkg.get(key)
                .and_then(|v| v.as_object())
                .is_some_and(|deps| deps.contains_key("@capacitor/core"))
        };
        check("dependencies") || check("devDependencies")
    }

    fn capacitor_cli_version(pkg: &Value) -> Option<String> {
        let get_version = |deps: Option<&Value>| -> Option<String> {
            deps.and_then(|v| v.get("@capacitor/cli"))
                .and_then(|v| v.as_str())
                .map(|s| s.trim_start_matches(['^', '~']).to_string())
        };
        get_version(pkg.get("devDependencies")).or_else(|| get_version(pkg.get("dependencies")))
    }

    fn config_path(root: &Path) -> Option<std::path::PathBuf> {
        for name in [
            "capacitor.config.ts",
            "capacitor.config.js",
            "capacitor.config.json",
        ] {
            let p = root.join(name);
            if p.exists() {
                return Some(p);
            }
        }
        None
    }

    fn detect_platform_dirs(root: &Path, info: &mut FrameworkInfo) {
        for (dir, name) in [("android", "android"), ("ios", "ios"), ("web", "web")] {
            if root.join(dir).is_dir() {
                info.target_platforms.push(name.to_string());
            }
        }
    }
}

impl Default for CapacitorDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FrameworkDetector for CapacitorDetector {
    fn detect(&self, root: &Path) -> bool {
        // Package dependency or config
        if let Some(pkg) = Self::read_package_json(root)
            && Self::has_capacitor_dependency(&pkg)
        {
            debug!("Detected Capacitor via package.json dependency");
            return true;
        }

        if Self::config_path(root).is_some() {
            debug!("Detected Capacitor via config file");
            return true;
        }

        false
    }

    fn framework(&self) -> ProjectFramework {
        ProjectFramework::Capacitor
    }

    async fn get_info(&self, root: &Path) -> AnalyzerResult<FrameworkInfo> {
        let mut info = FrameworkInfo::new(ProjectFramework::Capacitor);

        if let Some(pkg) = Self::read_package_json(root)
            && let Some(v) = Self::capacitor_cli_version(&pkg)
        {
            info = info.with_version(v);
        }

        if let Some(config) = Self::config_path(root) {
            info = info.with_config_path(config);
        }

        Self::detect_platform_dirs(root, &mut info);
        info = info.with_build_tool("@capacitor/cli");

        Ok(info)
    }

    fn required_tools(&self, _deps: &[Dependency], scripts: &[Script]) -> Vec<RequiredTool> {
        let mut tools = vec![RequiredTool::new(
            "node",
            Ecosystem::NodeJs,
            "Node.js runtime for Capacitor",
            InstallMethod::vx("node"),
        )];

        let uses_cli = scripts.iter().any(|s| s.command.contains("capacitor"));
        if uses_cli {
            tools.push(RequiredTool::new(
                "@capacitor/cli",
                Ecosystem::NodeJs,
                "Capacitor CLI",
                InstallMethod::npm_dev("@capacitor/cli"),
            ));
        }

        tools
    }

    async fn additional_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>> {
        let mut scripts = Vec::new();
        if let Some(pkg) = Self::read_package_json(root)
            && let Some(pkg_scripts) = pkg.get("scripts").and_then(|s| s.as_object())
        {
            let cap_scripts = [
                ("cap", "Run Capacitor CLI"),
                ("cap:sync", "Sync Capacitor platforms"),
                ("cap:android", "Run Capacitor Android"),
                ("cap:ios", "Run Capacitor iOS"),
            ];
            for (name, description) in cap_scripts {
                if let Some(cmd) = pkg_scripts.get(name).and_then(|v| v.as_str()) {
                    let mut script = Script::new(name, cmd, ScriptSource::PackageJson);
                    script.description = Some(description.to_string());
                    scripts.push(script);
                }
            }
        }
        Ok(scripts)
    }
}
