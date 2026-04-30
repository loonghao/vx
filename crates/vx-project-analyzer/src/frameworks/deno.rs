//! Deno framework detector
//!
//! Detects Deno applications by checking for:
//! - `deno.json` or `deno.jsonc` configuration files
//! - Deno-specific URL imports (https://deno.land/, https://esm.sh/)
//! - `deps.ts` or `deps.js` files (Deno dependency management)
//!
//! Deno is a secure JavaScript/TypeScript runtime with:
//! - Built-in test runner, formatter, linter
//! - URL-based module system (no package.json required)
//! - Standard library and first-party modules

use super::{FrameworkDetector, FrameworkInfo, ProjectFramework};
use crate::dependency::Dependency;
use crate::error::AnalyzerResult;
use crate::types::{RequiredTool, Script, ScriptSource};
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;
use tracing::debug;

/// Deno framework detector
pub struct DenoDetector;

impl DenoDetector {
    /// Create a new Deno detector
    pub fn new() -> Self {
        Self {}
    }

    /// Check for Deno configuration files
    fn has_deno_config(root: &Path) -> bool {
        root.join("deno.json").exists() || root.join("deno.jsonc").exists()
    }

    /// Check for Deno-specific files
    fn has_deno_files(root: &Path) -> bool {
        // Common Deno entry points
        let deno_files = [
            "mod.ts", "mod.js", "main.ts", "main.js", "deps.ts", "deps.js",
        ];

        deno_files.iter().any(|f| root.join(f).exists())
    }

    /// Check if any TypeScript/JavaScript file contains Deno imports
    fn has_deno_imports(root: &Path) -> bool {
        // Check common entry point files
        let check_files = [
            "mod.ts", "mod.js", "main.ts", "main.js", "index.ts", "index.js",
        ];

        for file in &check_files {
            let path = root.join(file);
            if !path.exists() {
                continue;
            }

            // Read first few lines to check for Deno imports
            if let Ok(content) = std::fs::read_to_string(&path) {
                // Deno-specific import patterns
                if content.contains("https://deno.land/")
                    || content.contains("https://esm.sh/")
                    || content.contains("https://unpkg.com/")
                {
                    return true;
                }
            }
        }

        false
    }

    /// Detect build/test tasks from deno.json
    fn detect_deno_tasks(deno_json: &Value) -> Vec<String> {
        let mut tasks = Vec::new();

        if let Some(tasks_obj) = deno_json.get("tasks").and_then(|t| t.as_object()) {
            for (name, _cmd) in tasks_obj {
                tasks.push(name.clone());
            }
        }

        tasks
    }
}

impl Default for DenoDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FrameworkDetector for DenoDetector {
    fn detect(&self, root: &Path) -> bool {
        // Check for Deno configuration files
        if Self::has_deno_config(root) {
            debug!("Detected Deno project via configuration file");
            return true;
        }

        // Check for Deno-specific files
        if Self::has_deno_files(root) {
            debug!("Detected Deno project via special files");
            return true;
        }

        // Check for Deno-specific imports
        if Self::has_deno_imports(root) {
            debug!("Detected Deno project via URL imports");
            return true;
        }

        false
    }

    fn framework(&self) -> ProjectFramework {
        ProjectFramework::Deno
    }

    async fn get_info(&self, root: &Path) -> AnalyzerResult<FrameworkInfo> {
        let mut info = FrameworkInfo::new(ProjectFramework::Deno);

        // Read deno.json if available
        let deno_json_path = root.join("deno.json");
        if deno_json_path.exists() {
            let content = tokio::fs::read_to_string(&deno_json_path).await?;
            if let Ok(deno_json) = serde_json::from_str::<Value>(&content) {
                // Get Deno version constraint if specified
                if let Some(_version) = deno_json
                    .get("compilerOptions")
                    .and_then(|o| o.as_object())
                    .and(None::<String>)
                // Deno doesn't specify version in config
                {
                    // Deno version is typically managed by `deno upgrade` or .tool-versions
                }

                // Detect tasks
                let tasks = Self::detect_deno_tasks(&deno_json);
                if !tasks.is_empty() {
                    info = info.with_metadata("tasks", tasks.join(","));
                }

                // Check for lint/test configurations
                if deno_json.get("lint").is_some() {
                    info = info.with_metadata("has_lint", "true");
                }
                if deno_json.get("fmt").is_some() {
                    info = info.with_metadata("has_fmt", "true");
                }
            }
        }

        // Check for .tool-versions (asdf/vm-compatible)
        if root.join(".tool-versions").exists() {
            info = info.with_metadata("version_manager", "tool-versions");
        }

        Ok(info)
    }

    fn required_tools(&self, _deps: &[Dependency], _scripts: &[Script]) -> Vec<RequiredTool> {
        vec![RequiredTool::new(
            "deno",
            crate::ecosystem::Ecosystem::NodeJs, // Deno is in Node.js ecosystem
            "Deno runtime for JavaScript/TypeScript",
            crate::dependency::InstallMethod::Vx {
                tool: "deno".to_string(),
                version: None,
            },
        )]
    }

    async fn additional_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>> {
        let mut scripts = Vec::new();

        // Check deno.json for tasks
        let deno_json_path = root.join("deno.json");
        if deno_json_path.exists() {
            let content = tokio::fs::read_to_string(&deno_json_path).await?;
            if let Ok(deno_json) = serde_json::from_str::<Value>(&content)
                && let Some(tasks) = deno_json.get("tasks").and_then(|t| t.as_object())
            {
                for (name, cmd) in tasks {
                    if let Some(_cmd_str) = cmd.as_str() {
                        let script = Script::new(
                            format!("deno:{}", name),
                            format!("deno task {}", name),
                            ScriptSource::BuildDeno,
                        );
                        scripts.push(script);
                    }
                }
            }
        }

        Ok(scripts)
    }
}
