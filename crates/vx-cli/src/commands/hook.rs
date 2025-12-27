//! Hook command implementation
//!
//! This module handles lifecycle hook management including:
//! - pre-commit hook execution and git integration
//! - enter hook execution for directory changes
//! - hook installation and status

use crate::ui::UI;
use anyhow::Result;
use std::env;
use vx_config::{EnterHookManager, GitHookInstaller, HookExecutor};

/// Handle pre-commit hook execution
pub async fn handle_pre_commit() -> Result<()> {
    let current_dir = env::current_dir()?;
    let config_path = current_dir.join(".vx.toml");

    if !config_path.exists() {
        // No config, skip silently (allow commit)
        return Ok(());
    }

    let config = vx_config::parse_config(&config_path)?;

    if let Some(hooks) = &config.hooks {
        if let Some(pre_commit) = &hooks.pre_commit {
            UI::info("Running pre-commit hook...");

            let executor = HookExecutor::new(&current_dir).verbose(true);
            let result = executor.execute_pre_commit(pre_commit)?;

            if !result.success {
                UI::error(&format!(
                    "Pre-commit hook failed: {}",
                    result.error.unwrap_or_default()
                ));
                std::process::exit(1);
            }

            UI::success("Pre-commit hook passed");
        }
    }

    Ok(())
}

/// Handle enter hook execution
pub async fn handle_enter() -> Result<()> {
    let current_dir = env::current_dir()?;
    let config_path = current_dir.join(".vx.toml");

    if !config_path.exists() {
        return Ok(());
    }

    // Check if we should trigger the hook
    let vx_paths = vx_paths::VxPaths::new()?;
    let manager = EnterHookManager::new(&vx_paths.cache_dir);

    if !manager.should_trigger(&current_dir) {
        return Ok(());
    }

    let config = vx_config::parse_config(&config_path)?;

    if let Some(hooks) = &config.hooks {
        if let Some(enter) = &hooks.enter {
            let executor = HookExecutor::new(&current_dir);
            let result = executor.execute_enter(enter)?;

            if !result.success {
                UI::warn(&format!(
                    "Enter hook failed: {}",
                    result.error.unwrap_or_default()
                ));
            }
        }
    }

    // Update cache
    manager.set_current_directory(&current_dir)?;

    Ok(())
}

/// Handle hook installation
pub async fn handle_install(force: bool) -> Result<()> {
    let current_dir = env::current_dir()?;

    // Find git repository root
    let repo_root = GitHookInstaller::find_repo_root(&current_dir).ok_or_else(|| {
        anyhow::anyhow!("Not in a git repository. Run this command from within a git repo.")
    })?;

    let installer = GitHookInstaller::new(&repo_root);

    if installer.is_installed() && !force {
        UI::info("Git hooks are already installed. Use --force to reinstall.");
        return Ok(());
    }

    UI::info("Installing git hooks...");
    installer.install_pre_commit()?;
    UI::success("Git hooks installed successfully");

    // Show hook file location
    let hooks_dir = installer.hooks_dir();
    UI::hint(&format!("Hook installed at: {}", hooks_dir.display()));

    Ok(())
}

/// Handle hook uninstallation
pub async fn handle_uninstall() -> Result<()> {
    let current_dir = env::current_dir()?;

    let repo_root = GitHookInstaller::find_repo_root(&current_dir).ok_or_else(|| {
        anyhow::anyhow!("Not in a git repository. Run this command from within a git repo.")
    })?;

    let installer = GitHookInstaller::new(&repo_root);

    if !installer.is_installed() {
        UI::info("Git hooks are not installed.");
        return Ok(());
    }

    UI::info("Uninstalling git hooks...");
    installer.uninstall_pre_commit()?;
    UI::success("Git hooks uninstalled successfully");

    Ok(())
}

/// Handle hook status display
pub async fn handle_status() -> Result<()> {
    let current_dir = env::current_dir()?;
    let config_path = current_dir.join(".vx.toml");

    println!("Hook Status");
    println!("===========\n");

    // Git hooks status
    if let Some(repo_root) = GitHookInstaller::find_repo_root(&current_dir) {
        let installer = GitHookInstaller::new(&repo_root);
        let git_installed = installer.is_installed();

        println!("Git Hooks:");
        println!(
            "  pre-commit: {}",
            if git_installed {
                "✓ installed"
            } else {
                "✗ not installed"
            }
        );
        println!("  Location: {}", installer.hooks_dir().display());
    } else {
        println!("Git Hooks: Not in a git repository");
    }

    println!();

    // Config hooks status
    if config_path.exists() {
        let config = vx_config::parse_config(&config_path)?;

        println!("Configured Hooks (.vx.toml):");
        if let Some(hooks) = &config.hooks {
            if hooks.pre_setup.is_some() {
                println!("  pre_setup: ✓ configured");
            }
            if hooks.post_setup.is_some() {
                println!("  post_setup: ✓ configured");
            }
            if hooks.pre_commit.is_some() {
                println!("  pre_commit: ✓ configured");
            }
            if hooks.enter.is_some() {
                println!("  enter: ✓ configured");
            }
            if !hooks.custom.is_empty() {
                let custom_keys: Vec<&str> = hooks.custom.keys().map(|s| s.as_str()).collect();
                println!("  custom hooks: {}", custom_keys.join(", "));
            }
        } else {
            println!("  No hooks configured");
        }
    } else {
        println!("Config: No .vx.toml found");
    }

    Ok(())
}

/// Handle custom hook execution
pub async fn handle_run(name: &str) -> Result<()> {
    let current_dir = env::current_dir()?;
    let config_path = current_dir.join(".vx.toml");

    if !config_path.exists() {
        return Err(anyhow::anyhow!("No .vx.toml found in current directory"));
    }

    let config = vx_config::parse_config(&config_path)?;

    let hooks = config
        .hooks
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No hooks configured in .vx.toml"))?;

    // Check built-in hooks first
    let hook = match name {
        "pre_setup" => hooks.pre_setup.as_ref(),
        "post_setup" => hooks.post_setup.as_ref(),
        "pre_commit" => hooks.pre_commit.as_ref(),
        "enter" => hooks.enter.as_ref(),
        _ => hooks.custom.get(name),
    };

    let hook = hook.ok_or_else(|| {
        let available: Vec<_> = hooks.custom.keys().collect();
        anyhow::anyhow!(
            "Hook '{}' not found. Available custom hooks: {}",
            name,
            if available.is_empty() {
                "none".to_string()
            } else {
                available
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        )
    })?;

    UI::info(&format!("Running hook '{}'...", name));

    let executor = HookExecutor::new(&current_dir).verbose(true);
    let result = executor.execute(name, hook)?;

    if result.success {
        UI::success(&format!("Hook '{}' completed successfully", name));
    } else {
        UI::error(&format!(
            "Hook '{}' failed: {}",
            name,
            result.error.unwrap_or_default()
        ));
        std::process::exit(1);
    }

    Ok(())
}

/// Handle shell integration script generation
pub async fn handle_shell_init(shell: Option<String>) -> Result<()> {
    let shell = shell.unwrap_or_else(|| detect_shell());

    let script = EnterHookManager::generate_shell_integration(&shell);

    if script.is_empty() {
        return Err(anyhow::anyhow!(
            "Unsupported shell: {}. Supported shells: bash, zsh, fish, pwsh",
            shell
        ));
    }

    println!("{}", script);

    // Print usage hint to stderr so it doesn't interfere with eval
    eprintln!();
    eprintln!("# Add the following to your shell config:");
    match shell.as_str() {
        "bash" => eprintln!("# eval \"$(vx hook shell-init --shell bash)\""),
        "zsh" => eprintln!("# eval \"$(vx hook shell-init --shell zsh)\""),
        "fish" => eprintln!("# vx hook shell-init --shell fish | source"),
        "pwsh" | "powershell" => {
            eprintln!("# Invoke-Expression (vx hook shell-init --shell pwsh)")
        }
        _ => {}
    }

    Ok(())
}

/// Detect current shell
fn detect_shell() -> String {
    if cfg!(windows) {
        // Check for PowerShell
        if env::var("PSModulePath").is_ok() {
            return "pwsh".to_string();
        }
        return "cmd".to_string();
    }

    env::var("SHELL")
        .ok()
        .and_then(|s| s.rsplit('/').next().map(String::from))
        .unwrap_or_else(|| "bash".to_string())
}
