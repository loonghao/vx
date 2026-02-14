//! Hook extension support
//!
//! This module implements lifecycle hooks that execute at specific events.

use crate::error::ExtensionResult;
use crate::{Extension, ExtensionDiscovery, ExtensionType};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use tracing::{debug, info, warn};

/// Hook event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookEvent {
    /// Before installing a runtime
    PreInstall,
    /// After installing a runtime
    PostInstall,
    /// Before uninstalling a runtime
    PreUninstall,
    /// After uninstalling a runtime
    PostUninstall,
    /// Before running a command
    PreRun,
    /// After running a command
    PostRun,
    /// When entering a project directory
    EnterProject,
    /// When leaving a project directory
    LeaveProject,
}

impl HookEvent {
    /// Get the config key for this event
    pub fn config_key(&self) -> &'static str {
        match self {
            HookEvent::PreInstall => "pre-install",
            HookEvent::PostInstall => "post-install",
            HookEvent::PreUninstall => "pre-uninstall",
            HookEvent::PostUninstall => "post-uninstall",
            HookEvent::PreRun => "pre-run",
            HookEvent::PostRun => "post-run",
            HookEvent::EnterProject => "enter-project",
            HookEvent::LeaveProject => "leave-project",
        }
    }

    /// Parse from config key
    pub fn from_config_key(key: &str) -> Option<Self> {
        match key {
            "pre-install" => Some(HookEvent::PreInstall),
            "post-install" => Some(HookEvent::PostInstall),
            "pre-uninstall" => Some(HookEvent::PreUninstall),
            "post-uninstall" => Some(HookEvent::PostUninstall),
            "pre-run" => Some(HookEvent::PreRun),
            "post-run" => Some(HookEvent::PostRun),
            "enter-project" => Some(HookEvent::EnterProject),
            "leave-project" => Some(HookEvent::LeaveProject),
            _ => None,
        }
    }
}

impl std::fmt::Display for HookEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.config_key())
    }
}

/// Context passed to hook scripts
#[derive(Debug, Clone, Default)]
pub struct HookContext {
    /// Runtime name (for install/uninstall hooks)
    pub runtime: Option<String>,
    /// Runtime version (for install/uninstall hooks)
    pub version: Option<String>,
    /// Command being run (for pre/post-run hooks)
    pub command: Option<String>,
    /// Command arguments
    pub args: Vec<String>,
    /// Project directory
    pub project_dir: Option<String>,
    /// Additional environment variables
    pub env: HashMap<String, String>,
}

impl HookContext {
    /// Create a new hook context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the runtime name
    pub fn with_runtime(mut self, runtime: impl Into<String>) -> Self {
        self.runtime = Some(runtime.into());
        self
    }

    /// Set the runtime version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set the command
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Set the command arguments
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Set the project directory
    pub fn with_project_dir(mut self, dir: impl Into<String>) -> Self {
        self.project_dir = Some(dir.into());
        self
    }

    /// Add an environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Convert to environment variables
    pub fn to_env_vars(&self) -> HashMap<String, String> {
        let mut env = self.env.clone();

        if let Some(ref runtime) = self.runtime {
            env.insert("VX_HOOK_RUNTIME".to_string(), runtime.clone());
        }
        if let Some(ref version) = self.version {
            env.insert("VX_HOOK_VERSION".to_string(), version.clone());
        }
        if let Some(ref command) = self.command {
            env.insert("VX_HOOK_COMMAND".to_string(), command.clone());
        }
        if !self.args.is_empty() {
            env.insert("VX_HOOK_ARGS".to_string(), self.args.join(" "));
        }
        if let Some(ref project_dir) = self.project_dir {
            env.insert("VX_HOOK_PROJECT_DIR".to_string(), project_dir.clone());
        }

        env
    }
}

/// Hook executor
pub struct HookExecutor {
    discovery: ExtensionDiscovery,
}

impl HookExecutor {
    /// Create a new hook executor
    pub fn new() -> ExtensionResult<Self> {
        Ok(Self {
            discovery: ExtensionDiscovery::new()?,
        })
    }

    /// Create a hook executor with a project directory
    pub fn with_project_dir(project_dir: std::path::PathBuf) -> ExtensionResult<Self> {
        Ok(Self {
            discovery: ExtensionDiscovery::new()?.with_project_dir(project_dir),
        })
    }

    /// Execute all hooks for a given event
    pub async fn execute_hooks(
        &self,
        event: HookEvent,
        context: &HookContext,
    ) -> ExtensionResult<Vec<HookResult>> {
        let extensions = self.discovery.discover_all().await?;
        let mut results = Vec::new();

        // Filter to hook extensions that have this event defined
        let hook_extensions: Vec<_> = extensions
            .into_iter()
            .filter(|ext| {
                ext.config.extension.extension_type == ExtensionType::Hook
                    && ext.config.hooks.contains_key(event.config_key())
            })
            .collect();

        if hook_extensions.is_empty() {
            debug!("No hooks registered for event: {}", event);
            return Ok(results);
        }

        info!(
            "Executing {} hook(s) for event: {}",
            hook_extensions.len(),
            event
        );

        for ext in hook_extensions {
            let result = self.execute_hook(&ext, event, context).await;
            results.push(result);
        }

        Ok(results)
    }

    /// Execute a single hook
    async fn execute_hook(
        &self,
        extension: &Extension,
        event: HookEvent,
        context: &HookContext,
    ) -> HookResult {
        let script = match extension.config.hooks.get(event.config_key()) {
            Some(s) => s,
            None => {
                return HookResult {
                    extension: extension.name.clone(),
                    event,
                    success: false,
                    exit_code: None,
                    error: Some("Hook script not found".to_string()),
                };
            }
        };

        let script_path = extension.path.join(script);

        if !script_path.exists() {
            return HookResult {
                extension: extension.name.clone(),
                event,
                success: false,
                exit_code: None,
                error: Some(format!("Script not found: {}", script_path.display())),
            };
        }

        debug!(
            "Executing hook '{}' for extension '{}'",
            event, extension.name
        );

        // Determine the interpreter based on file extension
        let interpreter = Self::get_interpreter(&script_path);

        // Build the command
        let mut cmd = if let Some(interp) = interpreter {
            let mut c = Command::new(interp);
            c.arg(&script_path);
            c
        } else {
            Command::new(&script_path)
        };

        // Set working directory to extension directory
        cmd.current_dir(&extension.path);

        // Set environment variables
        let env_vars = context.to_env_vars();
        for (key, value) in &env_vars {
            cmd.env(key, value);
        }

        // Add standard vx environment variables
        cmd.env("VX_EXTENSION_NAME", &extension.name);
        cmd.env(
            "VX_EXTENSION_DIR",
            extension.path.to_string_lossy().as_ref(),
        );
        cmd.env("VX_HOOK_EVENT", event.config_key());

        // Execute
        match cmd.output() {
            Ok(output) => {
                let success = output.status.success();
                let exit_code = output.status.code();

                if !success {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    warn!(
                        "Hook '{}' for extension '{}' failed: {}",
                        event, extension.name, stderr
                    );
                }

                HookResult {
                    extension: extension.name.clone(),
                    event,
                    success,
                    exit_code,
                    error: if success {
                        None
                    } else {
                        Some(String::from_utf8_lossy(&output.stderr).to_string())
                    },
                }
            }
            Err(e) => HookResult {
                extension: extension.name.clone(),
                event,
                success: false,
                exit_code: None,
                error: Some(format!("Failed to execute hook: {}", e)),
            },
        }
    }

    /// Get the interpreter for a script based on file extension
    fn get_interpreter(script_path: &Path) -> Option<&'static str> {
        let ext = script_path.extension()?.to_str()?;
        match ext {
            "py" => Some("python"),
            "js" => Some("node"),
            "ts" => Some("npx ts-node"),
            "rb" => Some("ruby"),
            "pl" => Some("perl"),
            "sh" => Some("sh"),
            "bash" => Some("bash"),
            "ps1" => Some("powershell"),
            _ => None,
        }
    }
}

impl Default for HookExecutor {
    fn default() -> Self {
        Self::new().expect("Failed to create hook executor")
    }
}

/// Result of executing a hook
#[derive(Debug, Clone)]
pub struct HookResult {
    /// Extension that provided the hook
    pub extension: String,
    /// Event that was triggered
    pub event: HookEvent,
    /// Whether the hook succeeded
    pub success: bool,
    /// Exit code if available
    pub exit_code: Option<i32>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Convenience function to execute hooks for an event
pub async fn execute_hooks(
    event: HookEvent,
    context: &HookContext,
) -> ExtensionResult<Vec<HookResult>> {
    let executor = HookExecutor::new()?;
    executor.execute_hooks(event, context).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_event_config_key() {
        assert_eq!(HookEvent::PreInstall.config_key(), "pre-install");
        assert_eq!(HookEvent::PostInstall.config_key(), "post-install");
        assert_eq!(HookEvent::PreRun.config_key(), "pre-run");
        assert_eq!(HookEvent::PostRun.config_key(), "post-run");
    }

    #[test]
    fn test_hook_event_from_config_key() {
        assert_eq!(
            HookEvent::from_config_key("pre-install"),
            Some(HookEvent::PreInstall)
        );
        assert_eq!(
            HookEvent::from_config_key("post-install"),
            Some(HookEvent::PostInstall)
        );
        assert_eq!(HookEvent::from_config_key("invalid"), None);
    }

    #[test]
    fn test_hook_context_to_env_vars() {
        let context = HookContext::new()
            .with_runtime("node")
            .with_version("18.0.0")
            .with_command("install")
            .with_args(vec!["express".to_string()]);

        let env = context.to_env_vars();
        assert_eq!(env.get("VX_HOOK_RUNTIME"), Some(&"node".to_string()));
        assert_eq!(env.get("VX_HOOK_VERSION"), Some(&"18.0.0".to_string()));
        assert_eq!(env.get("VX_HOOK_COMMAND"), Some(&"install".to_string()));
        assert_eq!(env.get("VX_HOOK_ARGS"), Some(&"express".to_string()));
    }

    #[test]
    fn test_hook_context_builder() {
        let context = HookContext::new()
            .with_runtime("python")
            .with_version("3.11")
            .with_project_dir("/path/to/project")
            .with_env("CUSTOM_VAR", "value");

        assert_eq!(context.runtime, Some("python".to_string()));
        assert_eq!(context.version, Some("3.11".to_string()));
        assert_eq!(context.project_dir, Some("/path/to/project".to_string()));
        assert_eq!(context.env.get("CUSTOM_VAR"), Some(&"value".to_string()));
    }
}
