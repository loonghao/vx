//! NW.js framework detector
//! Detects NW.js apps via package.json main entry and nw dependency.

use super::{FrameworkDetector, FrameworkInfo, ProjectFramework};
use crate::dependency::{Dependency, InstallMethod};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use crate::types::{RequiredTool, Script, ScriptSource};
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;
use tracing::debug;

pub struct NwJsDetector;

impl NwJsDetector {
    pub fn new() -> Self {
        Self
    }

    fn read_package_json(root: &Path) -> Option<Value> {
        let path = root.join("package.json");
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str::<Value>(&content).ok()
    }

    fn has_nw_dependency(pkg: &Value) -> bool {
        let check = |key: &str| -> bool {
            pkg.get(key)
                .and_then(|v| v.as_object())
                .is_some_and(|deps| deps.contains_key("nw") || deps.contains_key("nwjs"))
        };
        check("dependencies") || check("devDependencies")
    }

    fn main_is_html(pkg: &Value) -> bool {
        pkg.get("main")
            .and_then(|v| v.as_str())
            .is_some_and(|m| m.ends_with(".html") || m.ends_with(".htm"))
    }
}

impl Default for NwJsDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FrameworkDetector for NwJsDetector {
    fn detect(&self, root: &Path) -> bool {
        if let Some(pkg) = Self::read_package_json(root) {
            if Self::has_nw_dependency(&pkg) {
                debug!("Detected NW.js via dependency");
                return true;
            }
            if Self::main_is_html(&pkg) {
                debug!("Detected NW.js via HTML main entry");
                return true;
            }
        }
        false
    }

    fn framework(&self) -> ProjectFramework {
        ProjectFramework::NwJs
    }

    async fn get_info(&self, root: &Path) -> AnalyzerResult<FrameworkInfo> {
        let mut info = FrameworkInfo::new(ProjectFramework::NwJs);
        if let Some(pkg) = Self::read_package_json(root) {
            if let Some(main) = pkg.get("main").and_then(|v| v.as_str()) {
                info = info.with_metadata("main", main);
            }
            info = info.with_build_tool("nw");
        }
        Ok(info)
    }

    fn required_tools(&self, _deps: &[Dependency], scripts: &[Script]) -> Vec<RequiredTool> {
        let mut tools = vec![RequiredTool::new(
            "node",
            Ecosystem::NodeJs,
            "Node.js runtime for NW.js",
            InstallMethod::vx("node"),
        )];
        let uses_nw = scripts.iter().any(|s| s.command.contains("nw"));
        if uses_nw {
            tools.push(RequiredTool::new(
                "nw",
                Ecosystem::NodeJs,
                "NW.js runtime",
                InstallMethod::npm_dev("nw"),
            ));
        }
        tools
    }

    async fn additional_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>> {
        let mut scripts = Vec::new();
        if let Some(pkg) = Self::read_package_json(root) {
            if let Some(pkg_scripts) = pkg.get("scripts").and_then(|s| s.as_object()) {
                for name in ["start", "dev"] {
                    if let Some(cmd) = pkg_scripts.get(name).and_then(|v| v.as_str()) {
                        if cmd.contains("nw") {
                            let mut script = Script::new(name, cmd, ScriptSource::PackageJson);
                            script.description = Some("Run NW.js app".to_string());
                            scripts.push(script);
                        }
                    }
                }
            }
        }
        Ok(scripts)
    }
}
