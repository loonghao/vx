//! Node.js project analyzer implementation

use super::dependencies::parse_package_json_dependencies;
use super::package_manager::PackageManager;
use super::rules::NODEJS_RULES;
use super::scripts::parse_package_json_scripts;
use crate::dependency::{Dependency, InstallMethod};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use crate::languages::rules::{apply_rules, merge_scripts};
use crate::languages::LanguageAnalyzer;
use crate::script_parser::ScriptParser;
use crate::types::{RequiredTool, Script};
use async_trait::async_trait;
use std::path::Path;
use tracing::debug;

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
}

impl Default for NodeJsAnalyzer {
    fn default() -> Self {
        Self::new()
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
        let package_json_path = root.join("package.json");
        if !package_json_path.exists() {
            return Ok(Vec::new());
        }

        debug!("Analyzing package.json");
        let content = tokio::fs::read_to_string(&package_json_path).await?;
        let mut deps = parse_package_json_dependencies(&content, &package_json_path)?;

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
        let package_json_path = root.join("package.json");
        let pm = PackageManager::detect(root);

        // 1. Parse explicit scripts from package.json (highest priority)
        let mut explicit_scripts = Vec::new();
        if package_json_path.exists() {
            let content = tokio::fs::read_to_string(&package_json_path).await?;
            explicit_scripts = parse_package_json_scripts(&content, pm, &self.script_parser)?;
        }

        // 2. Apply detection rules for common scripts
        let detected_scripts = apply_rules(root, NODEJS_RULES, &self.script_parser);

        // 3. Merge: explicit scripts take priority over detected ones
        Ok(merge_scripts(explicit_scripts, detected_scripts))
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
