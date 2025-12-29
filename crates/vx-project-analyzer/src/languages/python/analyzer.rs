//! Python project analyzer implementation

use super::dependencies::{parse_pyproject_dependencies, parse_requirements_txt};
use super::rules::PYTHON_RULES;
use super::scripts::parse_pyproject_scripts;
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

/// Python project analyzer
pub struct PythonAnalyzer {
    script_parser: ScriptParser,
}

impl PythonAnalyzer {
    /// Create a new Python analyzer
    pub fn new() -> Self {
        Self {
            script_parser: ScriptParser::new(),
        }
    }
}

impl Default for PythonAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageAnalyzer for PythonAnalyzer {
    fn detect(&self, root: &Path) -> bool {
        root.join("pyproject.toml").exists()
            || root.join("setup.py").exists()
            || root.join("requirements.txt").exists()
    }

    fn name(&self) -> &'static str {
        "Python"
    }

    async fn analyze_dependencies(&self, root: &Path) -> AnalyzerResult<Vec<Dependency>> {
        let mut deps = Vec::new();

        // Analyze pyproject.toml
        let pyproject_path = root.join("pyproject.toml");
        if pyproject_path.exists() {
            debug!("Analyzing pyproject.toml");
            let content = tokio::fs::read_to_string(&pyproject_path).await?;
            deps.extend(parse_pyproject_dependencies(&content, &pyproject_path)?);
        }

        // Analyze requirements.txt
        let requirements_path = root.join("requirements.txt");
        if requirements_path.exists() {
            debug!("Analyzing requirements.txt");
            let content = tokio::fs::read_to_string(&requirements_path).await?;
            deps.extend(parse_requirements_txt(&content, &requirements_path)?);
        }

        // Check if dependencies are installed (via uv.lock or .venv)
        let has_lock = root.join("uv.lock").exists();
        let has_venv = root.join(".venv").exists();

        if has_lock || has_venv {
            for dep in &mut deps {
                dep.is_installed = true;
            }
        }

        Ok(deps)
    }

    async fn analyze_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>> {
        // 1. Parse explicit scripts from pyproject.toml (highest priority)
        let mut explicit_scripts = Vec::new();
        let pyproject_path = root.join("pyproject.toml");
        if pyproject_path.exists() {
            let content = tokio::fs::read_to_string(&pyproject_path).await?;
            explicit_scripts = parse_pyproject_scripts(&content, &self.script_parser)?;
        }

        // 2. Apply detection rules for common scripts
        let detected_scripts = apply_rules(root, PYTHON_RULES, &self.script_parser);

        // 3. Merge: explicit scripts take priority over detected ones
        Ok(merge_scripts(explicit_scripts, detected_scripts))
    }

    fn required_tools(&self, _deps: &[Dependency], scripts: &[Script]) -> Vec<RequiredTool> {
        let mut tools = Vec::new();

        // Always need uv for Python projects
        tools.push(RequiredTool::new(
            "uv",
            Ecosystem::Python,
            "Python package manager",
            InstallMethod::vx("uv"),
        ));

        // Check scripts for tool requirements
        for script in scripts {
            for tool in &script.tools {
                if !tool.is_available {
                    let install_method = InstallMethod::uv_dev(&tool.name);
                    tools.push(RequiredTool::new(
                        &tool.name,
                        Ecosystem::Python,
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
        if dep.is_dev {
            Some(format!("uv add --group dev {}", dep.name))
        } else {
            Some(format!("uv add {}", dep.name))
        }
    }
}
