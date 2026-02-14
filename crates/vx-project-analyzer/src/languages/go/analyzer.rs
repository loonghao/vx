//! Go project analyzer implementation

use super::dependencies::parse_go_mod_dependencies;
use super::rules::GO_RULES;
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

/// Go project analyzer
pub struct GoAnalyzer {
    script_parser: ScriptParser,
}

impl GoAnalyzer {
    /// Create a new Go analyzer
    pub fn new() -> Self {
        Self {
            script_parser: ScriptParser::new(),
        }
    }
}

impl Default for GoAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageAnalyzer for GoAnalyzer {
    fn detect(&self, root: &Path) -> bool {
        root.join("go.mod").exists()
    }

    fn name(&self) -> &'static str {
        "Go"
    }

    async fn analyze_dependencies(&self, root: &Path) -> AnalyzerResult<Vec<Dependency>> {
        let go_mod_path = root.join("go.mod");
        if !go_mod_path.exists() {
            return Ok(Vec::new());
        }

        debug!("Analyzing go.mod");
        let content = tokio::fs::read_to_string(&go_mod_path).await?;
        let mut deps = parse_go_mod_dependencies(&content, &go_mod_path)?;

        // Check if go.sum exists (dependencies resolved)
        let has_sum = root.join("go.sum").exists();
        if has_sum {
            for dep in &mut deps {
                dep.is_installed = true;
            }
        }

        Ok(deps)
    }

    async fn analyze_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>> {
        // Go projects typically don't have explicit script definitions
        // We rely on detection rules
        let detected_scripts = apply_rules(root, GO_RULES, &self.script_parser);
        Ok(merge_scripts(Vec::new(), detected_scripts))
    }

    fn required_tools(&self, _deps: &[Dependency], scripts: &[Script]) -> Vec<RequiredTool> {
        let mut tools = Vec::new();

        // Always need Go for Go projects
        tools.push(RequiredTool::new(
            "go",
            Ecosystem::Go,
            "Go toolchain",
            InstallMethod::vx("go"),
        ));

        // Check for golangci-lint
        for script in scripts {
            if script.command.contains("golangci-lint") {
                tools.push(RequiredTool::new(
                    "golangci-lint",
                    Ecosystem::Go,
                    "Go linter aggregator",
                    InstallMethod::Go {
                        command:
                            "go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest"
                                .to_string(),
                    },
                ));
                break;
            }
        }

        // Check for goimports
        for script in scripts {
            if script.command.contains("goimports") {
                tools.push(RequiredTool::new(
                    "goimports",
                    Ecosystem::Go,
                    "Import organizer",
                    InstallMethod::Go {
                        command: "go install golang.org/x/tools/cmd/goimports@latest".to_string(),
                    },
                ));
                break;
            }
        }

        tools
    }

    fn install_command(&self, dep: &Dependency) -> Option<String> {
        Some(format!("go get {}", dep.name))
    }
}
