//! Setup pipeline execution engine
//!
//! This module provides the setup pipeline functionality for `vx setup`.
//! The setup process is a pipeline of hooks that can be customized.
//!
//! # Default Pipeline
//!
//! 1. `pre_setup` - User-defined hook (from [hooks] section)
//! 2. `install_tools` - Built-in: Install all configured tools
//! 3. `export_paths` - Built-in: Export tool paths for CI environments
//! 4. `post_setup` - User-defined hook (from [hooks] section)
//!
//! # CI Environment Support
//!
//! The pipeline automatically detects CI environments and adjusts behavior:
//! - GitHub Actions: Exports paths to `$GITHUB_PATH`
//! - GitLab CI: Outputs paths for manual export
//! - Azure Pipelines: Uses `$GITHUB_PATH` (compatible)
//! - CircleCI: Exports to `$BASH_ENV`
//! - Generic CI: Outputs paths for manual export

use crate::HookExecutor;
use crate::types::{CiProvider, HookCommand, HooksConfig, SetupConfig, VxConfig};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Setup pipeline result
#[derive(Debug, Clone)]
pub struct SetupPipelineResult {
    /// Whether the pipeline succeeded
    pub success: bool,
    /// Results for each hook
    pub hook_results: Vec<SetupHookResult>,
    /// Exported paths (if any)
    pub exported_paths: Vec<PathBuf>,
    /// CI provider detected
    pub ci_provider: CiProvider,
}

/// Result of a single setup hook
#[derive(Debug, Clone)]
pub struct SetupHookResult {
    /// Hook name
    pub name: String,
    /// Whether the hook succeeded
    pub success: bool,
    /// Whether the hook was skipped
    pub skipped: bool,
    /// Skip reason (if skipped)
    pub skip_reason: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Output (if any)
    pub output: Option<String>,
}

/// Setup pipeline executor
pub struct SetupPipeline {
    /// Working directory
    working_dir: PathBuf,
    /// VX store directory (where tools are installed)
    store_dir: PathBuf,
    /// VX bin directory
    bin_dir: PathBuf,
    /// Setup configuration
    setup_config: Option<SetupConfig>,
    /// Hooks configuration
    hooks_config: Option<HooksConfig>,
    /// Tool versions from config
    tools: HashMap<String, String>,
    /// Whether to show verbose output
    verbose: bool,
    /// CI provider
    ci_provider: CiProvider,
    /// Force CI mode
    force_ci: bool,
}

impl SetupPipeline {
    /// Create a new setup pipeline
    pub fn new(
        working_dir: impl AsRef<Path>,
        store_dir: impl AsRef<Path>,
        bin_dir: impl AsRef<Path>,
    ) -> Self {
        Self {
            working_dir: working_dir.as_ref().to_path_buf(),
            store_dir: store_dir.as_ref().to_path_buf(),
            bin_dir: bin_dir.as_ref().to_path_buf(),
            setup_config: None,
            hooks_config: None,
            tools: HashMap::new(),
            verbose: false,
            ci_provider: CiProvider::detect(),
            force_ci: false,
        }
    }

    /// Set verbose mode
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Force CI mode
    pub fn force_ci(mut self, force: bool) -> Self {
        self.force_ci = force;
        self
    }

    /// Set configuration from VxConfig
    pub fn with_config(mut self, config: &VxConfig) -> Self {
        self.setup_config = config.setup.clone();
        self.hooks_config = config.hooks.clone();
        self.tools = config.tools_as_hashmap();
        self
    }

    /// Check if running in CI mode
    pub fn is_ci(&self) -> bool {
        if self.force_ci {
            return true;
        }
        if let Some(ci_config) = self.setup_config.as_ref().and_then(|s| s.ci.as_ref())
            && let Some(enabled) = ci_config.enabled
        {
            return enabled;
        }
        self.ci_provider.is_ci()
    }

    /// Get the pipeline to execute
    pub fn get_pipeline(&self) -> Vec<String> {
        self.setup_config
            .as_ref()
            .map(|s| s.get_pipeline())
            .unwrap_or_else(SetupConfig::default_pipeline)
    }

    /// Execute the setup pipeline
    ///
    /// This is the main entry point for running the setup pipeline.
    /// It executes hooks in order and collects results.
    ///
    /// # Arguments
    /// * `install_fn` - Async function to install tools (called for `install_tools` hook)
    pub async fn execute<F, Fut>(&self, install_fn: F) -> Result<SetupPipelineResult>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        let pipeline = self.get_pipeline();
        let mut hook_results = Vec::new();
        let mut exported_paths = Vec::new();
        let mut overall_success = true;

        // Track if install_fn has been called
        let mut install_fn = Some(install_fn);

        for hook_name in &pipeline {
            let result = match hook_name.as_str() {
                "pre_setup" => self.execute_user_hook("pre_setup").await,
                "post_setup" => self.execute_user_hook("post_setup").await,
                "install_tools" => {
                    // Call the provided install function (only once)
                    if let Some(f) = install_fn.take() {
                        match f().await {
                            Ok(()) => SetupHookResult {
                                name: "install_tools".to_string(),
                                success: true,
                                skipped: false,
                                skip_reason: None,
                                error: None,
                                output: None,
                            },
                            Err(e) => SetupHookResult {
                                name: "install_tools".to_string(),
                                success: false,
                                skipped: false,
                                skip_reason: None,
                                error: Some(e.to_string()),
                                output: None,
                            },
                        }
                    } else {
                        SetupHookResult {
                            name: "install_tools".to_string(),
                            success: true,
                            skipped: true,
                            skip_reason: Some("Already executed".to_string()),
                            error: None,
                            output: None,
                        }
                    }
                }
                "export_paths" => {
                    let (result, paths) = self.execute_export_paths().await;
                    exported_paths = paths;
                    result
                }
                _ => {
                    // Custom hook
                    self.execute_custom_hook(hook_name).await
                }
            };

            if !result.success && !result.skipped {
                overall_success = false;
            }

            hook_results.push(result);

            // Stop on failure unless continue_on_failure is set
            if !overall_success {
                break;
            }
        }

        Ok(SetupPipelineResult {
            success: overall_success,
            hook_results,
            exported_paths,
            ci_provider: self.ci_provider,
        })
    }

    /// Execute a user-defined hook (pre_setup, post_setup)
    async fn execute_user_hook(&self, hook_name: &str) -> SetupHookResult {
        let hook_command = match hook_name {
            "pre_setup" => self.hooks_config.as_ref().and_then(|h| h.pre_setup.clone()),
            "post_setup" => self
                .hooks_config
                .as_ref()
                .and_then(|h| h.post_setup.clone()),
            _ => None,
        };

        match hook_command {
            Some(cmd) => {
                let executor = HookExecutor::new(&self.working_dir).verbose(self.verbose);
                match executor.execute(hook_name, &cmd) {
                    Ok(result) => SetupHookResult {
                        name: hook_name.to_string(),
                        success: result.success,
                        skipped: false,
                        skip_reason: None,
                        error: result.error,
                        output: result.output,
                    },
                    Err(e) => SetupHookResult {
                        name: hook_name.to_string(),
                        success: false,
                        skipped: false,
                        skip_reason: None,
                        error: Some(e.to_string()),
                        output: None,
                    },
                }
            }
            None => SetupHookResult {
                name: hook_name.to_string(),
                success: true,
                skipped: true,
                skip_reason: Some("No hook defined".to_string()),
                error: None,
                output: None,
            },
        }
    }

    /// Execute a custom hook from setup.hooks.custom
    async fn execute_custom_hook(&self, hook_name: &str) -> SetupHookResult {
        let custom_hook = self
            .setup_config
            .as_ref()
            .and_then(|s| s.hooks.as_ref())
            .and_then(|h| h.custom.get(hook_name));

        match custom_hook {
            Some(hook) => {
                // Convert SetupHookCommand to HookCommand
                let cmd = match hook {
                    crate::types::SetupHookCommand::Simple(s) => HookCommand::Single(s.clone()),
                    crate::types::SetupHookCommand::Multiple(v) => HookCommand::Multiple(v.clone()),
                    crate::types::SetupHookCommand::Detailed(d) => match &d.command {
                        crate::types::SetupHookCommandType::Single(s) => {
                            HookCommand::Single(s.clone())
                        }
                        crate::types::SetupHookCommandType::Multiple(v) => {
                            HookCommand::Multiple(v.clone())
                        }
                    },
                };

                let executor = HookExecutor::new(&self.working_dir).verbose(self.verbose);
                match executor.execute(hook_name, &cmd) {
                    Ok(result) => SetupHookResult {
                        name: hook_name.to_string(),
                        success: result.success,
                        skipped: false,
                        skip_reason: None,
                        error: result.error,
                        output: result.output,
                    },
                    Err(e) => SetupHookResult {
                        name: hook_name.to_string(),
                        success: false,
                        skipped: false,
                        skip_reason: None,
                        error: Some(e.to_string()),
                        output: None,
                    },
                }
            }
            None => SetupHookResult {
                name: hook_name.to_string(),
                success: true,
                skipped: true,
                skip_reason: Some(format!("Custom hook '{}' not defined", hook_name)),
                error: None,
                output: None,
            },
        }
    }

    /// Execute the export_paths hook
    async fn execute_export_paths(&self) -> (SetupHookResult, Vec<PathBuf>) {
        // Check if we should skip (ci_only mode)
        let ci_only = self
            .setup_config
            .as_ref()
            .and_then(|s| s.hooks.as_ref())
            .and_then(|h| h.export_paths.as_ref())
            .map(|e| e.ci_only)
            .unwrap_or(true);

        if ci_only && !self.is_ci() {
            return (
                SetupHookResult {
                    name: "export_paths".to_string(),
                    success: true,
                    skipped: true,
                    skip_reason: Some("Not in CI environment (ci_only=true)".to_string()),
                    error: None,
                    output: None,
                },
                Vec::new(),
            );
        }

        // Collect tool paths
        let mut paths = Vec::new();
        let mut output_lines = Vec::new();

        for tool_name in self.tools.keys() {
            let tool_dir = self.store_dir.join(tool_name);

            if !tool_dir.exists() {
                continue;
            }

            // Find the latest version directory
            let versions: Vec<_> = match fs::read_dir(&tool_dir) {
                Ok(entries) => entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect(),
                Err(_) => continue,
            };

            if versions.is_empty() {
                continue;
            }

            // Sort versions and get the latest
            let mut sorted_versions = versions;
            sorted_versions.sort();
            let latest_version = sorted_versions.last().unwrap();
            let version_dir = tool_dir.join(latest_version);

            // Check for bin subdirectory
            let bin_dir = version_dir.join("bin");
            if bin_dir.exists() {
                paths.push(bin_dir.clone());
                output_lines.push(format!("  {} -> {}", tool_name, bin_dir.display()));
            }

            // Check for tool-specific subdirectories (e.g., uv-x86_64-unknown-linux-gnu)
            if let Ok(entries) = fs::read_dir(&version_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let entry_name = entry.file_name().to_string_lossy().to_string();
                    if entry_name.starts_with(&format!("{}-", tool_name))
                        && entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
                    {
                        let subdir = entry.path();
                        paths.push(subdir.clone());
                        output_lines.push(format!("  {} -> {}", tool_name, subdir.display()));
                    }
                }
            }

            // Also check if executable exists directly in version directory
            let exe_name = if cfg!(windows) {
                format!("{}.exe", tool_name)
            } else {
                tool_name.to_string()
            };
            if version_dir.join(&exe_name).exists() && !paths.contains(&version_dir) {
                paths.push(version_dir.clone());
                output_lines.push(format!("  {} -> {}", tool_name, version_dir.display()));
            }
        }

        // Add extra paths from config
        if let Some(extra) = self
            .setup_config
            .as_ref()
            .and_then(|s| s.hooks.as_ref())
            .and_then(|h| h.export_paths.as_ref())
            .map(|e| &e.extra_paths)
        {
            for path in extra {
                let expanded = shellexpand::tilde(path).to_string();
                let path_buf = PathBuf::from(expanded);
                if path_buf.exists() {
                    paths.push(path_buf.clone());
                    output_lines.push(format!("  extra -> {}", path_buf.display()));
                }
            }
        }

        // Also add vx bin directory
        if self.bin_dir.exists() {
            paths.push(self.bin_dir.clone());
            output_lines.push(format!("  vx bin -> {}", self.bin_dir.display()));
        }

        // Export paths based on CI provider
        let export_result = self.export_paths_to_ci(&paths);

        let output = if output_lines.is_empty() {
            None
        } else {
            Some(output_lines.join("\n"))
        };

        match export_result {
            Ok(msg) => (
                SetupHookResult {
                    name: "export_paths".to_string(),
                    success: true,
                    skipped: false,
                    skip_reason: None,
                    error: None,
                    output: Some(format!(
                        "{}\n{}",
                        output.unwrap_or_default(),
                        msg.unwrap_or_default()
                    )),
                },
                paths,
            ),
            Err(e) => (
                SetupHookResult {
                    name: "export_paths".to_string(),
                    success: false,
                    skipped: false,
                    skip_reason: None,
                    error: Some(e.to_string()),
                    output,
                },
                paths,
            ),
        }
    }

    /// Export paths to CI environment
    fn export_paths_to_ci(&self, paths: &[PathBuf]) -> Result<Option<String>> {
        if paths.is_empty() {
            return Ok(Some("No paths to export".to_string()));
        }

        // Check for custom path file from config
        let custom_path_file = self
            .setup_config
            .as_ref()
            .and_then(|s| s.ci.as_ref())
            .and_then(|c| c.path_env_file.clone());

        let path_file = custom_path_file.or_else(|| self.ci_provider.path_export_file());

        match path_file {
            Some(file) => {
                // Write paths to the CI path file
                let mut content = String::new();
                for path in paths {
                    content.push_str(&path.display().to_string());
                    content.push('\n');
                }

                fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(&file)
                    .with_context(|| format!("Failed to open {}", file))?
                    .write_all(content.as_bytes())
                    .with_context(|| format!("Failed to write to {}", file))?;

                Ok(Some(format!(
                    "Exported {} paths to {} ({})",
                    paths.len(),
                    file,
                    self.ci_provider
                )))
            }
            None => {
                // Output paths for manual export
                let mut output = String::new();
                output.push_str(&format!("\n# {} detected\n", self.ci_provider));
                output.push_str("# Add these paths to your PATH:\n");
                for path in paths {
                    output.push_str(&format!("export PATH=\"{}:$PATH\"\n", path.display()));
                }
                Ok(Some(output))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ci_provider_detection() {
        // This test depends on environment, so we just test the API
        let provider = CiProvider::detect();
        let _ = provider.is_ci();
        let _ = provider.name();
        let _ = provider.path_export_file();
    }

    #[test]
    fn test_default_pipeline() {
        let pipeline = SetupConfig::default_pipeline();
        assert_eq!(
            pipeline,
            vec!["pre_setup", "install_tools", "export_paths", "post_setup"]
        );
    }

    #[test]
    fn test_setup_pipeline_new() {
        let pipeline = SetupPipeline::new("/tmp", "/tmp/store", "/tmp/bin");
        assert_eq!(pipeline.get_pipeline(), SetupConfig::default_pipeline());
    }
}
