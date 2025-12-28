//! Python project analyzer

use crate::dependency::{Dependency, DependencySource, InstallMethod};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use crate::script_parser::ScriptParser;
use crate::types::{RequiredTool, Script, ScriptSource};
use async_trait::async_trait;
use std::path::Path;
use tracing::debug;

use super::LanguageAnalyzer;

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
            // Mark all dependencies as potentially installed
            // A more thorough check would parse the lock file
            for dep in &mut deps {
                dep.is_installed = true;
            }
        }

        Ok(deps)
    }

    async fn analyze_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>> {
        let mut scripts = Vec::new();

        // Analyze pyproject.toml scripts
        let pyproject_path = root.join("pyproject.toml");
        if pyproject_path.exists() {
            let content = tokio::fs::read_to_string(&pyproject_path).await?;
            scripts.extend(parse_pyproject_scripts(&content, &self.script_parser)?);
        }

        // Detect noxfile.py
        if root.join("noxfile.py").exists() {
            let mut script = Script::new(
                "nox",
                "uv run nox",
                ScriptSource::Detected {
                    reason: "noxfile.py exists".to_string(),
                },
            );
            script.tools = self.script_parser.parse("uv run nox");
            script.description = Some("Run nox sessions".to_string());
            scripts.push(script);
        }

        // Detect pytest
        if root.join("pytest.ini").exists()
            || root.join("pyproject.toml").exists()
            || root.join("tests").exists()
        {
            let mut script = Script::new(
                "test",
                "uv run pytest",
                ScriptSource::Detected {
                    reason: "pytest configuration detected".to_string(),
                },
            );
            script.tools = self.script_parser.parse("uv run pytest");
            script.description = Some("Run tests with pytest".to_string());
            scripts.push(script);
        }

        Ok(scripts)
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

/// Parse dependencies from pyproject.toml
fn parse_pyproject_dependencies(content: &str, path: &Path) -> AnalyzerResult<Vec<Dependency>> {
    let mut deps = Vec::new();

    // Parse as TOML
    let doc: toml::Value = toml::from_str(content)?;

    // Parse [project.dependencies]
    if let Some(project) = doc.get("project") {
        if let Some(dependencies) = project.get("dependencies") {
            if let Some(deps_array) = dependencies.as_array() {
                for dep_str in deps_array {
                    if let Some(s) = dep_str.as_str() {
                        if let Some(dep) = parse_dependency_string(s, path, "project.dependencies")
                        {
                            deps.push(dep);
                        }
                    }
                }
            }
        }

        // Parse [project.optional-dependencies]
        if let Some(optional) = project.get("optional-dependencies") {
            if let Some(optional_table) = optional.as_table() {
                for (group, group_deps) in optional_table {
                    if let Some(deps_array) = group_deps.as_array() {
                        for dep_str in deps_array {
                            if let Some(s) = dep_str.as_str() {
                                let section = format!("project.optional-dependencies.{}", group);
                                if let Some(mut dep) = parse_dependency_string(s, path, &section) {
                                    dep.is_dev = group == "dev" || group == "test";
                                    deps.push(dep);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Parse [dependency-groups] (PEP 735)
    if let Some(groups) = doc.get("dependency-groups") {
        if let Some(groups_table) = groups.as_table() {
            for (group, group_deps) in groups_table {
                if let Some(deps_array) = group_deps.as_array() {
                    for dep_str in deps_array {
                        if let Some(s) = dep_str.as_str() {
                            let section = format!("dependency-groups.{}", group);
                            if let Some(mut dep) = parse_dependency_string(s, path, &section) {
                                dep.is_dev = group == "dev" || group == "test";
                                deps.push(dep);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(deps)
}

/// Parse a single dependency string (e.g., "requests>=2.0")
fn parse_dependency_string(s: &str, path: &Path, section: &str) -> Option<Dependency> {
    // Simple parsing - extract name and version
    let s = s.trim();
    if s.is_empty() || s.starts_with('#') {
        return None;
    }

    // Handle extras: package[extra1,extra2]
    let name_end = s.find(['[', '>', '<', '=', '!', ';']).unwrap_or(s.len());

    let name = s[..name_end].trim().to_string();
    if name.is_empty() {
        return None;
    }

    // Extract version if present
    let version = if name_end < s.len() {
        let rest = &s[name_end..];
        // Find version specifier
        if let Some(idx) = rest.find(|c: char| c.is_ascii_digit()) {
            let version_start = idx;
            let version_end = rest[version_start..]
                .find([',', ';', '['])
                .map(|i| version_start + i)
                .unwrap_or(rest.len());
            Some(rest[version_start..version_end].trim().to_string())
        } else {
            None
        }
    } else {
        None
    };

    let mut dep = Dependency::new(
        name,
        Ecosystem::Python,
        DependencySource::ConfigFile {
            path: path.to_path_buf(),
            section: section.to_string(),
        },
    );

    if let Some(v) = version {
        dep = dep.with_version(v);
    }

    Some(dep)
}

/// Parse requirements.txt
fn parse_requirements_txt(content: &str, path: &Path) -> AnalyzerResult<Vec<Dependency>> {
    let mut deps = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') || line.starts_with('-') {
            continue;
        }

        if let Some(dep) = parse_dependency_string(line, path, "requirements.txt") {
            deps.push(dep);
        }
    }

    Ok(deps)
}

/// Parse scripts from pyproject.toml
fn parse_pyproject_scripts(content: &str, parser: &ScriptParser) -> AnalyzerResult<Vec<Script>> {
    let mut scripts = Vec::new();

    let doc: toml::Value = toml::from_str(content)?;

    // Parse [tool.uv.scripts] (uv-specific scripts)
    if let Some(tool) = doc.get("tool") {
        if let Some(uv) = tool.get("uv") {
            if let Some(uv_scripts) = uv.get("scripts") {
                if let Some(scripts_table) = uv_scripts.as_table() {
                    for (name, cmd) in scripts_table {
                        if let Some(cmd_str) = cmd.as_str() {
                            let mut script = Script::new(
                                name.clone(),
                                cmd_str.to_string(),
                                ScriptSource::PyprojectToml {
                                    section: "tool.uv.scripts".to_string(),
                                },
                            );
                            script.tools = parser.parse(cmd_str);
                            scripts.push(script);
                        }
                    }
                }
            }
        }
    }

    // Parse [project.scripts] (entry points)
    if let Some(project) = doc.get("project") {
        if let Some(project_scripts) = project.get("scripts") {
            if let Some(scripts_table) = project_scripts.as_table() {
                for (name, entry_point) in scripts_table {
                    if let Some(ep_str) = entry_point.as_str() {
                        // Entry points are module:function format
                        let cmd =
                            format!("python -m {}", ep_str.split(':').next().unwrap_or(ep_str));
                        let mut script = Script::new(
                            name.clone(),
                            cmd.clone(),
                            ScriptSource::PyprojectToml {
                                section: "project.scripts".to_string(),
                            },
                        );
                        script.tools = parser.parse(&cmd);
                        scripts.push(script);
                    }
                }
            }
        }
    }

    Ok(scripts)
}
