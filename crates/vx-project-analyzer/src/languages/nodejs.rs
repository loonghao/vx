//! Node.js project analyzer

use crate::dependency::{Dependency, DependencySource, InstallMethod};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use crate::script_parser::ScriptParser;
use crate::types::{RequiredTool, Script, ScriptSource};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use tracing::debug;

use super::LanguageAnalyzer;

/// Node.js project analyzer
pub struct NodeJsAnalyzer {
    script_parser: ScriptParser,
}

impl NodeJsAnalyzer {
    /// Create a new Node.js analyzer
    pub fn new() -> Self {
        Self {
            script_parser: ScriptParser::new(),
        }
    }

    /// Detect which package manager is used
    fn detect_package_manager(&self, root: &Path) -> PackageManager {
        if root.join("pnpm-lock.yaml").exists() {
            PackageManager::Pnpm
        } else if root.join("yarn.lock").exists() {
            PackageManager::Yarn
        } else if root.join("bun.lockb").exists() {
            PackageManager::Bun
        } else {
            PackageManager::Npm
        }
    }
}

impl Default for NodeJsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
}

impl PackageManager {
    fn name(&self) -> &'static str {
        match self {
            PackageManager::Npm => "npm",
            PackageManager::Yarn => "yarn",
            PackageManager::Pnpm => "pnpm",
            PackageManager::Bun => "bun",
        }
    }

    #[allow(dead_code)]
    fn install_dev_cmd(&self, package: &str) -> String {
        match self {
            PackageManager::Npm => format!("npm install --save-dev {}", package),
            PackageManager::Yarn => format!("yarn add --dev {}", package),
            PackageManager::Pnpm => format!("pnpm add --save-dev {}", package),
            PackageManager::Bun => format!("bun add --dev {}", package),
        }
    }

    #[allow(dead_code)]
    fn install_cmd(&self, package: &str) -> String {
        match self {
            PackageManager::Npm => format!("npm install {}", package),
            PackageManager::Yarn => format!("yarn add {}", package),
            PackageManager::Pnpm => format!("pnpm add {}", package),
            PackageManager::Bun => format!("bun add {}", package),
        }
    }
}

#[async_trait]
impl LanguageAnalyzer for NodeJsAnalyzer {
    fn detect(&self, root: &Path) -> bool {
        root.join("package.json").exists()
    }

    fn name(&self) -> &'static str {
        "Node.js"
    }

    async fn analyze_dependencies(&self, root: &Path) -> AnalyzerResult<Vec<Dependency>> {
        let mut deps = Vec::new();

        let package_json_path = root.join("package.json");
        if !package_json_path.exists() {
            return Ok(deps);
        }

        debug!("Analyzing package.json");
        let content = tokio::fs::read_to_string(&package_json_path).await?;
        let pkg: PackageJson = serde_json::from_str(&content)?;

        // Parse dependencies
        for (name, version) in pkg.dependencies.unwrap_or_default() {
            let mut dep = Dependency::new(
                name,
                Ecosystem::NodeJs,
                DependencySource::ConfigFile {
                    path: package_json_path.clone(),
                    section: "dependencies".to_string(),
                },
            );
            dep = dep.with_version(version);
            deps.push(dep);
        }

        // Parse devDependencies
        for (name, version) in pkg.dev_dependencies.unwrap_or_default() {
            let mut dep = Dependency::new(
                name,
                Ecosystem::NodeJs,
                DependencySource::ConfigFile {
                    path: package_json_path.clone(),
                    section: "devDependencies".to_string(),
                },
            );
            dep = dep.with_version(version).as_dev();
            deps.push(dep);
        }

        // Check if node_modules exists (dependencies installed)
        let has_node_modules = root.join("node_modules").exists();
        if has_node_modules {
            for dep in &mut deps {
                dep.is_installed = true;
            }
        }

        Ok(deps)
    }

    async fn analyze_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>> {
        let mut scripts = Vec::new();

        let package_json_path = root.join("package.json");
        if !package_json_path.exists() {
            return Ok(scripts);
        }

        let content = tokio::fs::read_to_string(&package_json_path).await?;
        let pkg: PackageJson = serde_json::from_str(&content)?;

        let pm = self.detect_package_manager(root);

        for (name, cmd) in pkg.scripts.unwrap_or_default() {
            // Convert npm script to full command
            let full_cmd = format!("{} run {}", pm.name(), name);
            let mut script = Script::new(name, full_cmd, ScriptSource::PackageJson);
            script.tools = self.script_parser.parse(&cmd);
            scripts.push(script);
        }

        Ok(scripts)
    }

    fn required_tools(&self, _deps: &[Dependency], scripts: &[Script]) -> Vec<RequiredTool> {
        let mut tools = Vec::new();

        // Always need node for Node.js projects
        tools.push(RequiredTool::new(
            "node",
            Ecosystem::NodeJs,
            "Node.js runtime",
            InstallMethod::vx("node"),
        ));

        // Check scripts for tool requirements
        for script in scripts {
            for tool in &script.tools {
                if !tool.is_available {
                    let install_method = InstallMethod::npm_dev(&tool.name);
                    tools.push(RequiredTool::new(
                        &tool.name,
                        Ecosystem::NodeJs,
                        format!("Required by script '{}'", script.name),
                        install_method,
                    ));
                }
            }
        }

        // Deduplicate by name
        tools.sort_by(|a, b| a.name.cmp(&b.name));
        tools.dedup_by(|a, b| a.name == b.name);

        tools
    }

    fn install_command(&self, dep: &Dependency) -> Option<String> {
        // Default to npm, but could detect package manager
        if dep.is_dev {
            Some(format!("npm install --save-dev {}", dep.name))
        } else {
            Some(format!("npm install {}", dep.name))
        }
    }
}

/// Minimal package.json structure
#[derive(Debug, Deserialize)]
struct PackageJson {
    #[serde(default)]
    dependencies: Option<HashMap<String, String>>,
    #[serde(default, rename = "devDependencies")]
    dev_dependencies: Option<HashMap<String, String>>,
    #[serde(default)]
    scripts: Option<HashMap<String, String>>,
}
