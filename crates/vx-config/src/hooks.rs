//! Hook execution engine
//!
//! This module provides the hook execution functionality for lifecycle hooks.
//!
//! # Supported Hooks
//!
//! - `pre_setup` / `post_setup` - Run before/after `vx setup`
//! - `pre_commit` - Run before git commit (integrates with git hooks)
//! - `enter` - Run when entering a directory (shell integration)

use crate::types::HookCommand;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};

/// Hook execution result
#[derive(Debug, Clone)]
pub struct HookResult {
    /// Hook name
    pub name: String,
    /// Whether the hook succeeded
    pub success: bool,
    /// Exit code (if available)
    pub exit_code: Option<i32>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Output (if captured)
    pub output: Option<String>,
}

/// Hook executor
pub struct HookExecutor {
    /// Working directory for hook execution
    working_dir: std::path::PathBuf,
    /// Whether to show output
    verbose: bool,
    /// Shell to use
    shell: String,
    /// Environment variables to set
    env_vars: std::collections::HashMap<String, String>,
}

impl HookExecutor {
    /// Create a new hook executor
    pub fn new(working_dir: impl AsRef<Path>) -> Self {
        let shell = if cfg!(windows) {
            "powershell".to_string()
        } else {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
        };

        Self {
            working_dir: working_dir.as_ref().to_path_buf(),
            verbose: false,
            shell,
            env_vars: std::collections::HashMap::new(),
        }
    }

    /// Set verbose mode
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set shell
    pub fn shell(mut self, shell: impl Into<String>) -> Self {
        self.shell = shell.into();
        self
    }

    /// Add environment variable
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    /// Add multiple environment variables
    pub fn envs(mut self, vars: std::collections::HashMap<String, String>) -> Self {
        self.env_vars.extend(vars);
        self
    }

    /// Execute a hook command
    pub fn execute(&self, name: &str, hook: &HookCommand) -> Result<HookResult> {
        let commands = match hook {
            HookCommand::Single(cmd) => vec![cmd.clone()],
            HookCommand::Multiple(cmds) => cmds.clone(),
        };

        let mut combined_output = String::new();

        for cmd in &commands {
            if cmd.trim().is_empty() {
                continue;
            }

            let result = self.run_command(name, cmd)?;
            if let Some(output) = &result.output {
                combined_output.push_str(output);
                combined_output.push('\n');
            }
            if !result.success {
                return Ok(HookResult {
                    output: Some(combined_output),
                    ..result
                });
            }
        }

        Ok(HookResult {
            name: name.to_string(),
            success: true,
            exit_code: Some(0),
            error: None,
            output: if combined_output.is_empty() {
                None
            } else {
                Some(combined_output)
            },
        })
    }

    /// Run a single command
    fn run_command(&self, name: &str, cmd: &str) -> Result<HookResult> {
        let (shell_cmd, shell_arg) = if cfg!(windows) {
            if self.shell.contains("powershell") || self.shell.contains("pwsh") {
                (&self.shell as &str, "-Command")
            } else {
                ("cmd", "/C")
            }
        } else {
            (&self.shell as &str, "-c")
        };

        let mut command = Command::new(shell_cmd);
        command.arg(shell_arg).arg(cmd);
        command.current_dir(&self.working_dir);

        // Set environment variables
        for (key, value) in &self.env_vars {
            command.env(key, value);
        }

        if self.verbose {
            command.stdout(Stdio::inherit());
            command.stderr(Stdio::inherit());
        } else {
            command.stdout(Stdio::piped());
            command.stderr(Stdio::piped());
        }

        let output = command
            .output()
            .with_context(|| format!("Failed to execute hook '{}': {}", name, cmd))?;

        let exit_code = output.status.code();
        let success = output.status.success();

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let error = if !success {
            if stderr.is_empty() {
                Some(format!(
                    "Hook '{}' failed with exit code {:?}",
                    name, exit_code
                ))
            } else {
                Some(stderr.clone())
            }
        } else {
            None
        };

        Ok(HookResult {
            name: name.to_string(),
            success,
            exit_code,
            error,
            output: if stdout.is_empty() && stderr.is_empty() {
                None
            } else {
                Some(format!("{}{}", stdout, stderr))
            },
        })
    }

    /// Execute pre-setup hooks
    pub fn execute_pre_setup(&self, hook: &HookCommand) -> Result<HookResult> {
        self.execute("pre_setup", hook)
    }

    /// Execute post-setup hooks
    pub fn execute_post_setup(&self, hook: &HookCommand) -> Result<HookResult> {
        self.execute("post_setup", hook)
    }

    /// Execute pre-commit hooks
    pub fn execute_pre_commit(&self, hook: &HookCommand) -> Result<HookResult> {
        self.execute("pre_commit", hook)
    }

    /// Execute enter hooks
    pub fn execute_enter(&self, hook: &HookCommand) -> Result<HookResult> {
        self.execute("enter", hook)
    }

    /// Execute a custom hook by name
    pub fn execute_custom(&self, name: &str, hook: &HookCommand) -> Result<HookResult> {
        self.execute(name, hook)
    }
}

/// Git hook installer for pre-commit integration
pub struct GitHookInstaller {
    /// Git repository root
    repo_root: std::path::PathBuf,
}

impl GitHookInstaller {
    /// Create a new git hook installer
    pub fn new(repo_root: impl AsRef<Path>) -> Self {
        Self {
            repo_root: repo_root.as_ref().to_path_buf(),
        }
    }

    /// Find git repository root from a path
    pub fn find_repo_root(start: impl AsRef<Path>) -> Option<std::path::PathBuf> {
        let mut current = start.as_ref().to_path_buf();
        loop {
            if current.join(".git").exists() {
                return Some(current);
            }
            if !current.pop() {
                return None;
            }
        }
    }

    /// Get the hooks directory
    pub fn hooks_dir(&self) -> std::path::PathBuf {
        self.repo_root.join(".git").join("hooks")
    }

    /// Install the pre-commit hook
    pub fn install_pre_commit(&self) -> Result<()> {
        let hooks_dir = self.hooks_dir();
        std::fs::create_dir_all(&hooks_dir)?;

        let hook_path = hooks_dir.join("pre-commit");

        let hook_content = self.generate_pre_commit_script();

        // Check if hook already exists
        if hook_path.exists() {
            let existing = std::fs::read_to_string(&hook_path)?;
            if existing.contains("# vx-managed") {
                // Update existing vx hook
                std::fs::write(&hook_path, hook_content)?;
            } else {
                // Backup existing hook and create wrapper
                let backup_path = hooks_dir.join("pre-commit.backup");
                std::fs::rename(&hook_path, &backup_path)?;
                std::fs::write(&hook_path, self.generate_wrapper_script(&backup_path))?;
            }
        } else {
            std::fs::write(&hook_path, hook_content)?;
        }

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&hook_path, perms)?;
        }

        Ok(())
    }

    /// Uninstall the pre-commit hook
    pub fn uninstall_pre_commit(&self) -> Result<()> {
        let hook_path = self.hooks_dir().join("pre-commit");
        let backup_path = self.hooks_dir().join("pre-commit.backup");

        if hook_path.exists() {
            let content = std::fs::read_to_string(&hook_path)?;
            if content.contains("# vx-managed") {
                std::fs::remove_file(&hook_path)?;

                // Restore backup if exists
                if backup_path.exists() {
                    std::fs::rename(&backup_path, &hook_path)?;
                }
            }
        }

        Ok(())
    }

    /// Generate the pre-commit hook script
    fn generate_pre_commit_script(&self) -> String {
        // Same script works for both Windows (via Git Bash) and Unix
        r#"#!/bin/sh
# vx-managed pre-commit hook
# This hook is managed by vx. Do not edit manually.

# Run vx pre-commit hook
if command -v vx >/dev/null 2>&1; then
    vx hook pre-commit
    exit $?
fi

# Fallback: try to find vx in common locations
for vx_path in "$HOME/.vx/bin/vx" "$HOME/.local/bin/vx" "/usr/local/bin/vx"; do
    if [ -x "$vx_path" ]; then
        "$vx_path" hook pre-commit
        exit $?
    fi
done

echo "Warning: vx not found, skipping pre-commit hook"
exit 0
"#
        .to_string()
    }

    /// Generate a wrapper script that calls both the backup and vx hook
    fn generate_wrapper_script(&self, backup_path: &Path) -> String {
        let backup = backup_path.display();
        format!(
            r#"#!/bin/sh
# vx-managed pre-commit hook wrapper
# This hook wraps an existing pre-commit hook.

# Run original hook first
if [ -x "{backup}" ]; then
    "{backup}"
    ORIGINAL_EXIT=$?
    if [ $ORIGINAL_EXIT -ne 0 ]; then
        exit $ORIGINAL_EXIT
    fi
fi

# Run vx pre-commit hook
if command -v vx >/dev/null 2>&1; then
    vx hook pre-commit
    exit $?
fi

exit 0
"#
        )
    }

    /// Check if pre-commit hook is installed
    pub fn is_installed(&self) -> bool {
        let hook_path = self.hooks_dir().join("pre-commit");
        if hook_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&hook_path) {
                return content.contains("# vx-managed");
            }
        }
        false
    }
}

/// Directory enter hook manager
pub struct EnterHookManager {
    /// Cache file for tracking directory state
    cache_file: std::path::PathBuf,
}

impl EnterHookManager {
    /// Create a new enter hook manager
    pub fn new(cache_dir: impl AsRef<Path>) -> Self {
        Self {
            cache_file: cache_dir.as_ref().join("enter_hook_cache.json"),
        }
    }

    /// Get the last directory from cache
    pub fn get_last_directory(&self) -> Option<std::path::PathBuf> {
        if let Ok(content) = std::fs::read_to_string(&self.cache_file) {
            if let Ok(cache) = serde_json::from_str::<EnterHookCache>(&content) {
                return Some(std::path::PathBuf::from(cache.last_directory));
            }
        }
        None
    }

    /// Set the current directory in cache
    pub fn set_current_directory(&self, dir: impl AsRef<Path>) -> Result<()> {
        let cache = EnterHookCache {
            last_directory: dir.as_ref().to_string_lossy().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        if let Some(parent) = self.cache_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string(&cache)?;
        std::fs::write(&self.cache_file, content)?;
        Ok(())
    }

    /// Check if directory changed and should trigger enter hook
    pub fn should_trigger(&self, current_dir: impl AsRef<Path>) -> bool {
        let current = current_dir.as_ref();
        if let Some(last) = self.get_last_directory() {
            // Check if we entered a new directory (not just navigating within)
            !current.starts_with(&last) || current != last
        } else {
            true
        }
    }

    /// Generate shell integration script for enter hook
    pub fn generate_shell_integration(shell: &str) -> String {
        match shell {
            "bash" => r#"
# vx enter hook integration for bash
__vx_enter_hook() {
    if [ -f ".vx.toml" ]; then
        vx hook enter 2>/dev/null
    fi
}

# Add to PROMPT_COMMAND
if [[ ! "$PROMPT_COMMAND" =~ "__vx_enter_hook" ]]; then
    PROMPT_COMMAND="__vx_enter_hook${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
fi
"#
            .to_string(),

            "zsh" => r#"
# vx enter hook integration for zsh
__vx_enter_hook() {
    if [ -f ".vx.toml" ]; then
        vx hook enter 2>/dev/null
    fi
}

# Add to chpwd hook
autoload -U add-zsh-hook
add-zsh-hook chpwd __vx_enter_hook

# Also run on shell start
__vx_enter_hook
"#
            .to_string(),

            "fish" => r#"
# vx enter hook integration for fish
function __vx_enter_hook --on-variable PWD
    if test -f ".vx.toml"
        vx hook enter 2>/dev/null
    end
end

# Also run on shell start
__vx_enter_hook
"#
            .to_string(),

            "pwsh" | "powershell" => r#"
# vx enter hook integration for PowerShell
function __vx_enter_hook {
    if (Test-Path ".vx.toml") {
        vx hook enter 2>$null
    }
}

# Override prompt to include enter hook
$__vx_original_prompt = $function:prompt
function prompt {
    __vx_enter_hook
    & $__vx_original_prompt
}
"#
            .to_string(),

            _ => String::new(),
        }
    }

    /// Get shell init file path
    pub fn get_shell_init_file(shell: &str) -> Option<std::path::PathBuf> {
        let home = dirs::home_dir()?;
        match shell {
            "bash" => {
                // Prefer .bashrc, fallback to .bash_profile
                let bashrc = home.join(".bashrc");
                if bashrc.exists() {
                    Some(bashrc)
                } else {
                    Some(home.join(".bash_profile"))
                }
            }
            "zsh" => Some(home.join(".zshrc")),
            "fish" => Some(home.join(".config/fish/config.fish")),
            "pwsh" | "powershell" => {
                // PowerShell profile
                if cfg!(windows) {
                    Some(home.join("Documents/PowerShell/Microsoft.PowerShell_profile.ps1"))
                } else {
                    Some(home.join(".config/powershell/Microsoft.PowerShell_profile.ps1"))
                }
            }
            _ => None,
        }
    }
}

/// Cache for enter hook state
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct EnterHookCache {
    last_directory: String,
    timestamp: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_hook_executor_single_command() {
        let executor = HookExecutor::new(env::current_dir().unwrap());
        let hook = HookCommand::Single("echo hello".to_string());
        let result = executor.execute("test", &hook).unwrap();
        assert!(result.success);
        assert_eq!(result.exit_code, Some(0));
    }

    #[test]
    fn test_hook_executor_multiple_commands() {
        let executor = HookExecutor::new(env::current_dir().unwrap());
        let hook = HookCommand::Multiple(vec!["echo first".to_string(), "echo second".to_string()]);
        let result = executor.execute("test", &hook).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_hook_executor_empty_command() {
        let executor = HookExecutor::new(env::current_dir().unwrap());
        let hook = HookCommand::Single("".to_string());
        let result = executor.execute("test", &hook).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_hook_executor_failing_command() {
        let executor = HookExecutor::new(env::current_dir().unwrap());
        let hook = if cfg!(windows) {
            HookCommand::Single("exit 1".to_string())
        } else {
            HookCommand::Single("exit 1".to_string())
        };
        let result = executor.execute("test", &hook).unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_hook_executor_stops_on_failure() {
        let executor = HookExecutor::new(env::current_dir().unwrap());
        let hook = HookCommand::Multiple(vec![
            "exit 1".to_string(),
            "echo should not run".to_string(),
        ]);
        let result = executor.execute("test", &hook).unwrap();
        assert!(!result.success);
    }

    #[test]
    fn test_hook_executor_with_env() {
        let executor = HookExecutor::new(env::current_dir().unwrap()).env("TEST_VAR", "test_value");
        let hook = if cfg!(windows) {
            HookCommand::Single("echo $env:TEST_VAR".to_string())
        } else {
            HookCommand::Single("echo $TEST_VAR".to_string())
        };
        let result = executor.execute("test", &hook).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_git_hook_installer_find_repo() {
        // This test depends on being run in a git repo
        if let Some(repo) = GitHookInstaller::find_repo_root(env::current_dir().unwrap()) {
            assert!(repo.join(".git").exists());
        }
    }

    #[test]
    fn test_shell_integration_generation() {
        let bash = EnterHookManager::generate_shell_integration("bash");
        assert!(bash.contains("__vx_enter_hook"));
        assert!(bash.contains("PROMPT_COMMAND"));

        let zsh = EnterHookManager::generate_shell_integration("zsh");
        assert!(zsh.contains("chpwd"));

        let fish = EnterHookManager::generate_shell_integration("fish");
        assert!(fish.contains("--on-variable PWD"));

        let pwsh = EnterHookManager::generate_shell_integration("pwsh");
        assert!(pwsh.contains("function prompt"));
    }
}
