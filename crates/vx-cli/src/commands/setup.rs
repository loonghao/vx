//! Setup command - One-click development environment setup
//!
//! This command reads the vx.toml configuration and installs all required
//! tools, making the project ready for development.
//!
//! `vx setup` internally calls `vx sync` for tool installation, then performs
//! additional setup tasks like showing next steps.
//!
//! ## Lifecycle Hooks
//!
//! The setup command supports lifecycle hooks:
//! - `pre_setup`: Runs before tool installation
//! - `post_setup`: Runs after tool installation
//!
//! Use `--no-hooks` to skip hook execution.

use crate::commands::sync;
use crate::ui::UI;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use vx_config::{parse_config, HookExecutor, VxConfig as VxConfigV2};
use vx_paths::{find_config_file, find_vx_config as find_vx_config_path};
use vx_runtime::ProviderRegistry;
use vx_setup::ci::{CiProvider, PathExporter};

// Re-export the new config type for backward compatibility
pub use vx_config::VxConfig as VxConfigNew;

/// Legacy configuration type for backward compatibility
/// This wraps the new VxConfigV2 and provides the old interface
#[derive(Debug, Default, Clone)]
pub struct VxConfig {
    pub tools: HashMap<String, String>,
    pub settings: HashMap<String, String>,
    pub env: HashMap<String, String>,
    pub scripts: HashMap<String, String>,
}

impl From<VxConfigV2> for VxConfig {
    fn from(config: VxConfigV2) -> Self {
        VxConfig {
            tools: config.tools_as_hashmap(),
            settings: config.settings_as_hashmap(),
            env: config.env_as_hashmap(),
            scripts: config.scripts_as_hashmap(),
        }
    }
}

/// Handle the setup command
///
/// This command delegates to `vx sync` for tool installation, then shows
/// additional setup guidance like next steps and available scripts.
///
/// ## Arguments
///
/// - `registry`: Provider registry for tool installation
/// - `force`: Force reinstall even if already installed
/// - `dry_run`: Show what would be done without making changes
/// - `verbose`: Show detailed output
/// - `no_parallel`: Disable parallel installation
/// - `no_hooks`: Skip lifecycle hooks (pre_setup, post_setup)
/// - `ci`: CI mode - output tool paths for CI environment (GitHub Actions, etc.)
pub async fn handle(
    registry: &ProviderRegistry,
    force: bool,
    dry_run: bool,
    verbose: bool,
    no_parallel: bool,
    no_hooks: bool,
    ci: bool,
) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    // Find and parse .vx.toml
    let config_path = find_vx_config(&current_dir)?;
    let config_v2 = parse_vx_config_v2(&config_path)?;
    let config = VxConfig::from(config_v2.clone());

    UI::header("🚀 VX Development Environment Setup");
    println!();

    // Execute pre_setup hook
    if !no_hooks && !dry_run {
        if let Some(hooks) = &config_v2.hooks {
            if let Some(pre_setup) = &hooks.pre_setup {
                UI::info("Running pre_setup hook...");
                let executor = HookExecutor::new(&current_dir).verbose(verbose);
                let result = executor.execute_pre_setup(pre_setup)?;
                if !result.success {
                    if let Some(err) = result.error {
                        return Err(anyhow::anyhow!("pre_setup hook failed: {}", err));
                    }
                    return Err(anyhow::anyhow!("pre_setup hook failed"));
                }
                UI::success("pre_setup hook completed");
                println!();
            }
        }
    }

    // Delegate to sync command for tool installation
    // sync handles: checking status, installing missing tools, showing progress
    sync::handle(
        registry,
        false,       // check: false - we want to install, not just check
        force,       // force: pass through
        dry_run,     // dry_run: pass through
        verbose,     // verbose: pass through (sync will show status when verbose)
        no_parallel, // no_parallel: pass through
        false,       // no_auto_install: false - we want auto install
    )
    .await?;

    // Execute post_setup hook
    if !no_hooks && !dry_run {
        if let Some(hooks) = &config_v2.hooks {
            if let Some(post_setup) = &hooks.post_setup {
                println!();
                UI::info("Running post_setup hook...");
                let executor = HookExecutor::new(&current_dir).verbose(verbose);
                let result = executor.execute_post_setup(post_setup)?;
                if !result.success {
                    if let Some(err) = result.error {
                        UI::warn(&format!("post_setup hook failed: {}", err));
                    } else {
                        UI::warn("post_setup hook failed");
                    }
                } else {
                    UI::success("post_setup hook completed");
                }
            }
        }
    }

    // Show next steps after successful sync (setup-specific feature)
    if !dry_run {
        if ci {
            // CI mode: output tool paths for GitHub Actions
            output_ci_paths(&config)?;
        } else {
            show_next_steps(&config);
        }
    }

    Ok(())
}

/// Find vx.toml or .vx.toml in current directory or parent directories
///
/// This is a wrapper around `vx_paths::find_vx_config` that converts the error
/// to `anyhow::Result` for consistency with other CLI commands.
pub fn find_vx_config(start_dir: &Path) -> Result<PathBuf> {
    find_vx_config_path(start_dir).map_err(|e| anyhow::anyhow!("{}", e))
}

/// Find config file in current directory only (for add/remove/update operations)
fn find_config_in_current_dir(current_dir: &Path) -> Result<PathBuf> {
    find_config_file(current_dir)
        .ok_or_else(|| anyhow::anyhow!("No vx.toml found. Run 'vx init' first."))
}

/// Parse .vx.toml configuration using the new serde-based parser
pub fn parse_vx_config(path: &Path) -> Result<VxConfig> {
    let config_v2 = parse_config(path)
        .with_context(|| format!("Failed to parse configuration file: {}", path.display()))?;

    Ok(VxConfig::from(config_v2))
}

/// Parse .vx.toml configuration and return the new typed config
pub fn parse_vx_config_v2(path: &Path) -> Result<VxConfigV2> {
    parse_config(path)
        .with_context(|| format!("Failed to parse configuration file: {}", path.display()))
}

/// Show next steps after setup
fn show_next_steps(config: &VxConfig) {
    println!();
    UI::info("Next steps:");
    println!("  1. Enter dev environment: vx dev");
    println!("  2. Or run tools directly: vx <tool> [args]");

    if !config.scripts.is_empty() {
        println!();
        println!("Available scripts:");
        for (name, cmd) in &config.scripts {
            println!("  vx run {} -> {}", name, cmd);
        }
    }
}

/// Output tool paths for CI environment (GitHub Actions, etc.)
///
/// This function uses vx-setup's PathExporter for CI path export.
fn output_ci_paths(config: &VxConfig) -> Result<()> {
    use vx_paths::VxPaths;

    let paths = VxPaths::new()?;
    let store_dir = &paths.store_dir;

    // Detect CI provider
    let ci_provider = CiProvider::detect();

    if ci_provider.is_ci() {
        UI::info(&format!("CI mode: {} detected", ci_provider));
    } else {
        UI::info("CI mode: Outputting tool paths");
    }

    let mut exported_paths = Vec::new();

    // Iterate through configured tools and find their paths
    for tool_name in config.tools.keys() {
        let tool_dir = store_dir.join(tool_name);

        if !tool_dir.exists() {
            UI::warn(&format!("Tool '{}' not found in store", tool_name));
            continue;
        }

        // Find the latest version directory
        let versions: Vec<_> = fs::read_dir(&tool_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();

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
            exported_paths.push(bin_dir.clone());
            println!("  {} -> {}", tool_name, bin_dir.display());
        }

        // Check for tool-specific subdirectories (e.g., uv-x86_64-unknown-linux-gnu)
        if let Ok(entries) = fs::read_dir(&version_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let entry_name = entry.file_name().to_string_lossy().to_string();
                if entry_name.starts_with(&format!("{}-", tool_name))
                    && entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
                {
                    let subdir = entry.path();
                    exported_paths.push(subdir.clone());
                    println!("  {} -> {}", tool_name, subdir.display());
                }
            }
        }

        // Also check if executable exists directly in version directory
        let exe_name = if cfg!(windows) {
            format!("{}.exe", tool_name)
        } else {
            tool_name.to_string()
        };
        if version_dir.join(&exe_name).exists() {
            exported_paths.push(version_dir.clone());
            println!("  {} -> {}", tool_name, version_dir.display());
        }
    }

    // Also add vx bin directory
    let vx_bin_dir = paths.bin_dir.clone();
    if vx_bin_dir.exists() {
        exported_paths.push(vx_bin_dir.clone());
        println!("  vx bin -> {}", vx_bin_dir.display());
    }

    // Use PathExporter from vx-setup
    let exporter = PathExporter::new(ci_provider);
    let result = exporter.export(&exported_paths)?;

    if result.target_file.is_some() {
        UI::success(&result.message);
    } else if let Some(commands) = &result.shell_commands {
        println!();
        println!("{}", commands);
    }

    Ok(())
}

/// Add a tool to the project configuration
pub async fn add_tool(tool: &str, version: Option<&str>) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let config_path = find_config_in_current_dir(&current_dir)?;

    let version = version.unwrap_or("latest");
    let mut config = parse_vx_config(&config_path)?;

    if config.tools.contains_key(tool) {
        UI::warn(&format!("Tool '{}' already configured", tool));
        UI::info(&format!(
            "Current version: {}",
            config.tools.get(tool).unwrap()
        ));
        return Ok(());
    }

    // Add tool to config
    config.tools.insert(tool.to_string(), version.to_string());

    // Rewrite config file
    write_vx_config(&config_path, &config)?;

    UI::success(&format!(
        "Added {}@{} to {}",
        tool,
        version,
        config_path.file_name().unwrap().to_string_lossy()
    ));
    UI::hint("Run 'vx setup' to install the tool");

    Ok(())
}

/// Remove a tool from the project configuration
pub async fn remove_tool(tool: &str) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let config_path = find_config_in_current_dir(&current_dir)?;

    let mut config = parse_vx_config(&config_path)?;

    if !config.tools.contains_key(tool) {
        UI::warn(&format!("Tool '{}' not found in configuration", tool));
        return Ok(());
    }

    config.tools.remove(tool);
    write_vx_config(&config_path, &config)?;

    UI::success(&format!(
        "Removed '{}' from {}",
        tool,
        config_path.file_name().unwrap().to_string_lossy()
    ));

    Ok(())
}

/// Update a tool version in the project configuration
pub async fn update_tool(tool: &str, version: &str) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let config_path = find_config_in_current_dir(&current_dir)?;

    let mut config = parse_vx_config(&config_path)?;

    let old_version = config.tools.get(tool).cloned();
    config.tools.insert(tool.to_string(), version.to_string());
    write_vx_config(&config_path, &config)?;

    if let Some(old) = old_version {
        UI::success(&format!("Updated {} from {} to {}", tool, old, version));
    } else {
        UI::success(&format!("Added {}@{} to vx.toml", tool, version));
    }

    UI::hint("Run 'vx setup' to install the updated tool");

    Ok(())
}

/// Write configuration back to .vx.toml
fn write_vx_config(path: &Path, config: &VxConfig) -> Result<()> {
    let mut content = String::new();

    content.push_str("# VX Project Configuration\n");
    content.push_str("# Run 'vx setup' to install all required tools.\n");
    content.push_str("# Run 'vx dev' to enter the development environment.\n\n");

    // Tools section
    content.push_str("[tools]\n");
    let mut tools: Vec<_> = config.tools.iter().collect();
    tools.sort_by_key(|(k, _)| *k);
    for (name, version) in tools {
        content.push_str(&format!("{} = \"{}\"\n", name, version));
    }
    content.push('\n');

    // Settings section
    if !config.settings.is_empty() {
        content.push_str("[settings]\n");
        let mut settings: Vec<_> = config.settings.iter().collect();
        settings.sort_by_key(|(k, _)| *k);
        for (key, value) in settings {
            content.push_str(&format!("{} = {}\n", key, format_toml_value(value)));
        }
        content.push('\n');
    }

    // Env section
    if !config.env.is_empty() {
        content.push_str("[env]\n");
        let mut env: Vec<_> = config.env.iter().collect();
        env.sort_by_key(|(k, _)| *k);
        for (key, value) in env {
            content.push_str(&format!("{} = \"{}\"\n", key, value));
        }
        content.push('\n');
    }

    // Scripts section
    if !config.scripts.is_empty() {
        content.push_str("[scripts]\n");
        let mut scripts: Vec<_> = config.scripts.iter().collect();
        scripts.sort_by_key(|(k, _)| *k);
        for (name, cmd) in scripts {
            content.push_str(&format!("{} = \"{}\"\n", name, cmd));
        }
    }

    fs::write(path, content)?;
    Ok(())
}

/// Format a value for TOML output, detecting booleans and numbers
fn format_toml_value(value: &str) -> String {
    // Check if it's a boolean
    if value == "true" || value == "false" {
        return value.to_string();
    }

    // Check if it's an integer
    if value.parse::<i64>().is_ok() {
        return value.to_string();
    }

    // Check if it's a float
    if value.parse::<f64>().is_ok() {
        return value.to_string();
    }

    // Otherwise, quote it as a string
    format!("\"{}\"", value)
}
