//! Setup pipeline executor

use super::hooks::{HookExecutor, HookResult};
use crate::ci::{CiProvider, PathExporter};
use crate::types::{HookCommand, SetupPipelineConfig};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
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

impl From<HookResult> for SetupHookResult {
    fn from(result: HookResult) -> Self {
        Self {
            name: result.name,
            success: result.success,
            skipped: false,
            skip_reason: None,
            error: result.error,
            output: result.output,
        }
    }
}

/// Setup pipeline executor
pub struct SetupPipeline {
    /// Working directory
    working_dir: PathBuf,
    /// VX store directory (where tools are installed)
    store_dir: PathBuf,
    /// VX bin directory
    bin_dir: PathBuf,
    /// Pipeline configuration
    config: SetupPipelineConfig,
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
            config: SetupPipelineConfig::default(),
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

    /// Set configuration
    pub fn with_config(mut self, config: SetupPipelineConfig) -> Self {
        self.config = config;
        self
    }

    /// Set tools
    pub fn with_tools(mut self, tools: HashMap<String, String>) -> Self {
        self.config.tools = tools;
        self
    }

    /// Check if running in CI mode
    pub fn is_ci(&self) -> bool {
        if self.force_ci {
            return true;
        }
        if let Some(ref ci_config) = self.config.ci {
            if let Some(enabled) = ci_config.enabled {
                return enabled;
            }
        }
        self.ci_provider.is_ci()
    }

    /// Get the pipeline to execute
    pub fn get_pipeline(&self) -> Vec<String> {
        self.config.get_pipeline()
    }

    /// Get the CI provider
    pub fn ci_provider(&self) -> CiProvider {
        self.ci_provider
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
                "pre_setup" => self.execute_user_hook("pre_setup", &self.config.pre_setup),
                "post_setup" => self.execute_user_hook("post_setup", &self.config.post_setup),
                "install_tools" => {
                    // Call the provided install function
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
                    let (result, paths) = self.execute_export_paths();
                    exported_paths = paths;
                    result
                }
                _ => {
                    // Custom hook
                    self.execute_custom_hook(hook_name)
                }
            };

            if !result.success && !result.skipped {
                overall_success = false;
            }

            hook_results.push(result);

            // Stop on failure
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
    fn execute_user_hook(
        &self,
        hook_name: &str,
        hook_command: &Option<HookCommand>,
    ) -> SetupHookResult {
        match hook_command {
            Some(cmd) => {
                let executor = HookExecutor::new(&self.working_dir).verbose(self.verbose);
                match executor.execute(hook_name, cmd) {
                    Ok(result) => result.into(),
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

    /// Execute a custom hook
    fn execute_custom_hook(&self, hook_name: &str) -> SetupHookResult {
        let custom_hook = self.config.custom_hooks.get(hook_name);

        match custom_hook {
            Some(hook) => {
                if !hook.enabled {
                    return SetupHookResult {
                        name: hook_name.to_string(),
                        success: true,
                        skipped: true,
                        skip_reason: Some("Hook disabled".to_string()),
                        error: None,
                        output: None,
                    };
                }

                // Check CI/local only conditions
                if hook.ci_only && !self.is_ci() {
                    return SetupHookResult {
                        name: hook_name.to_string(),
                        success: true,
                        skipped: true,
                        skip_reason: Some("CI only hook (not in CI)".to_string()),
                        error: None,
                        output: None,
                    };
                }
                if hook.local_only && self.is_ci() {
                    return SetupHookResult {
                        name: hook_name.to_string(),
                        success: true,
                        skipped: true,
                        skip_reason: Some("Local only hook (in CI)".to_string()),
                        error: None,
                        output: None,
                    };
                }

                let mut executor = HookExecutor::new(&self.working_dir).verbose(self.verbose);
                if !hook.env.is_empty() {
                    executor = executor.envs(hook.env.clone());
                }

                match executor.execute(hook_name, &hook.command) {
                    Ok(result) => result.into(),
                    Err(e) => SetupHookResult {
                        name: hook_name.to_string(),
                        success: hook.continue_on_failure,
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
    fn execute_export_paths(&self) -> (SetupHookResult, Vec<PathBuf>) {
        // Check if we should skip (ci_only mode)
        let export_config = self
            .config
            .export_paths
            .as_ref()
            .cloned()
            .unwrap_or_default();

        if export_config.ci_only && !self.is_ci() {
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

        for tool_name in self.config.tools.keys() {
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
        for path in &export_config.extra_paths {
            let expanded = shellexpand::tilde(path).to_string();
            let path_buf = PathBuf::from(expanded);
            if path_buf.exists() {
                paths.push(path_buf.clone());
                output_lines.push(format!("  extra -> {}", path_buf.display()));
            }
        }

        // Also add vx bin directory
        if self.bin_dir.exists() {
            paths.push(self.bin_dir.clone());
            output_lines.push(format!("  vx bin -> {}", self.bin_dir.display()));
        }

        // Export paths using PathExporter
        let custom_path_file = self
            .config
            .ci
            .as_ref()
            .and_then(|c| c.path_env_file.clone());

        let mut exporter = PathExporter::new(self.ci_provider);
        if let Some(file) = custom_path_file {
            exporter = exporter.with_custom_path_file(file);
        }

        let export_result = exporter.export(&paths);

        let output = if output_lines.is_empty() {
            None
        } else {
            Some(output_lines.join("\n"))
        };

        match export_result {
            Ok(result) => (
                SetupHookResult {
                    name: "export_paths".to_string(),
                    success: true,
                    skipped: false,
                    skip_reason: None,
                    error: None,
                    output: Some(format!(
                        "{}\n{}",
                        output.unwrap_or_default(),
                        result.message
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pipeline() {
        let pipeline = SetupPipelineConfig::default_pipeline();
        assert_eq!(
            pipeline,
            vec!["pre_setup", "install_tools", "export_paths", "post_setup"]
        );
    }

    #[test]
    fn test_setup_pipeline_new() {
        let pipeline = SetupPipeline::new("/tmp", "/tmp/store", "/tmp/bin");
        assert_eq!(
            pipeline.get_pipeline(),
            SetupPipelineConfig::default_pipeline()
        );
    }

    #[test]
    fn test_setup_pipeline_force_ci() {
        let pipeline = SetupPipeline::new("/tmp", "/tmp/store", "/tmp/bin").force_ci(true);
        assert!(pipeline.is_ci());
    }

    #[tokio::test]
    async fn test_setup_pipeline_execute() {
        let pipeline = SetupPipeline::new("/tmp", "/tmp/store", "/tmp/bin");

        let result = pipeline.execute(|| async { Ok(()) }).await.unwrap();

        // Pipeline should succeed (all hooks either succeed or are skipped)
        assert!(result.success);
    }
}
