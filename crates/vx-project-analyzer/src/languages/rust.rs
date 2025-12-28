//! Rust project analyzer

use crate::dependency::{Dependency, DependencySource, InstallMethod};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use crate::script_parser::ScriptParser;
use crate::types::{RequiredTool, Script, ScriptSource};
use async_trait::async_trait;
use std::path::Path;
use tracing::debug;

use super::LanguageAnalyzer;

/// Rust project analyzer
pub struct RustAnalyzer {
    #[allow(dead_code)]
    script_parser: ScriptParser,
}

impl RustAnalyzer {
    /// Create a new Rust analyzer
    pub fn new() -> Self {
        Self {
            script_parser: ScriptParser::new(),
        }
    }
}

impl Default for RustAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageAnalyzer for RustAnalyzer {
    fn detect(&self, root: &Path) -> bool {
        root.join("Cargo.toml").exists()
    }

    fn name(&self) -> &'static str {
        "Rust"
    }

    async fn analyze_dependencies(&self, root: &Path) -> AnalyzerResult<Vec<Dependency>> {
        let mut deps = Vec::new();

        let cargo_toml_path = root.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(deps);
        }

        debug!("Analyzing Cargo.toml");
        let content = tokio::fs::read_to_string(&cargo_toml_path).await?;
        let doc: toml::Value = toml::from_str(&content)?;

        // Parse [dependencies]
        if let Some(dependencies) = doc.get("dependencies") {
            if let Some(deps_table) = dependencies.as_table() {
                for (name, value) in deps_table {
                    let version = extract_version(value);
                    let mut dep = Dependency::new(
                        name.clone(),
                        Ecosystem::Rust,
                        DependencySource::ConfigFile {
                            path: cargo_toml_path.clone(),
                            section: "dependencies".to_string(),
                        },
                    );
                    if let Some(v) = version {
                        dep = dep.with_version(v);
                    }
                    deps.push(dep);
                }
            }
        }

        // Parse [dev-dependencies]
        if let Some(dev_deps) = doc.get("dev-dependencies") {
            if let Some(deps_table) = dev_deps.as_table() {
                for (name, value) in deps_table {
                    let version = extract_version(value);
                    let mut dep = Dependency::new(
                        name.clone(),
                        Ecosystem::Rust,
                        DependencySource::ConfigFile {
                            path: cargo_toml_path.clone(),
                            section: "dev-dependencies".to_string(),
                        },
                    );
                    if let Some(v) = version {
                        dep = dep.with_version(v);
                    }
                    dep = dep.as_dev();
                    deps.push(dep);
                }
            }
        }

        // Check if Cargo.lock exists (dependencies resolved)
        let has_lock = root.join("Cargo.lock").exists();
        if has_lock {
            for dep in &mut deps {
                dep.is_installed = true;
            }
        }

        Ok(deps)
    }

    async fn analyze_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>> {
        let mut scripts = Vec::new();

        // Rust projects typically use cargo commands
        // We can detect common patterns

        // Check for justfile
        if root.join("justfile").exists() || root.join("Justfile").exists() {
            scripts.push(Script::new(
                "just",
                "just",
                ScriptSource::Detected {
                    reason: "justfile exists".to_string(),
                },
            ));
        }

        // Standard cargo commands
        scripts.push(Script::new(
            "build",
            "cargo build",
            ScriptSource::Detected {
                reason: "Cargo.toml exists".to_string(),
            },
        ));

        scripts.push(Script::new(
            "test",
            "cargo test",
            ScriptSource::Detected {
                reason: "Cargo.toml exists".to_string(),
            },
        ));

        scripts.push(Script::new(
            "check",
            "cargo check",
            ScriptSource::Detected {
                reason: "Cargo.toml exists".to_string(),
            },
        ));

        Ok(scripts)
    }

    fn required_tools(&self, _deps: &[Dependency], scripts: &[Script]) -> Vec<RequiredTool> {
        let mut tools = Vec::new();

        // Always need rust for Rust projects
        tools.push(RequiredTool::new(
            "rust",
            Ecosystem::Rust,
            "Rust toolchain",
            InstallMethod::vx("rust"),
        ));

        // Check for just
        for script in scripts {
            if script.command.starts_with("just") {
                tools.push(RequiredTool::new(
                    "just",
                    Ecosystem::Rust,
                    "Task runner",
                    InstallMethod::vx("just"),
                ));
                break;
            }
        }

        tools
    }

    fn install_command(&self, dep: &Dependency) -> Option<String> {
        Some(format!("cargo add {}", dep.name))
    }
}

/// Extract version from Cargo.toml dependency value
fn extract_version(value: &toml::Value) -> Option<String> {
    match value {
        toml::Value::String(s) => Some(s.clone()),
        toml::Value::Table(t) => t.get("version").and_then(|v| v.as_str().map(String::from)),
        _ => None,
    }
}
