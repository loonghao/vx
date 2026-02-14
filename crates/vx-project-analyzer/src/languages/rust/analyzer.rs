//! Rust project analyzer implementation

use super::dependencies::parse_cargo_dependencies;
use super::rules::RUST_RULES;
use super::scripts::parse_cargo_scripts;
use crate::dependency::{Dependency, InstallMethod};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use crate::languages::LanguageAnalyzer;
use crate::languages::rules::{apply_rules, merge_scripts};
use crate::script_parser::ScriptParser;
use crate::types::{RequiredTool, Script};
use async_trait::async_trait;
use std::path::Path;
use tracing::debug;

/// Rust project analyzer
pub struct RustAnalyzer {
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
        let cargo_toml_path = root.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(Vec::new());
        }

        debug!("Analyzing Cargo.toml");
        let content = tokio::fs::read_to_string(&cargo_toml_path).await?;
        let mut deps = parse_cargo_dependencies(&content, &cargo_toml_path)?;

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
        let mut explicit_scripts = Vec::new();

        // 1. Parse scripts from Cargo.toml
        let cargo_toml_path = root.join("Cargo.toml");
        if cargo_toml_path.exists() {
            let content = tokio::fs::read_to_string(&cargo_toml_path).await?;
            explicit_scripts.extend(parse_cargo_scripts(&content, &self.script_parser)?);
        }

        // Note: justfile parsing is now handled by the common JustfileAnalyzer
        // to avoid duplicate parsing in multi-language projects

        // 2. Apply detection rules for common scripts
        let detected_scripts = apply_rules(root, RUST_RULES, &self.script_parser);

        // 3. Merge: explicit scripts take priority over detected ones
        Ok(merge_scripts(explicit_scripts, detected_scripts))
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

        // Note: 'just' tool detection is now handled by the common JustfileAnalyzer
        // to correctly identify it as a cross-language tool

        // Check for cargo-nextest
        for script in scripts {
            if script.command.contains("nextest") {
                tools.push(RequiredTool::new(
                    "cargo-nextest",
                    Ecosystem::Rust,
                    "Test runner",
                    InstallMethod::Cargo {
                        command: "cargo install cargo-nextest".to_string(),
                    },
                ));
                break;
            }
        }

        // Check for cargo-make
        for script in scripts {
            if script.command.contains("cargo make") {
                tools.push(RequiredTool::new(
                    "cargo-make",
                    Ecosystem::Rust,
                    "Task runner",
                    InstallMethod::Cargo {
                        command: "cargo install cargo-make".to_string(),
                    },
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
