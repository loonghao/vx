//! Setup command - One-click development environment setup
//!
//! This command reads the .vx.toml configuration and installs all required
//! tools, making the project ready for development.

use crate::ui::{InstallProgress, UI};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Instant;
use vx_paths::PathManager;
use vx_runtime::ProviderRegistry;

/// Configuration from .vx.toml
#[derive(Debug, Default, Clone)]
pub struct VxConfig {
    pub tools: HashMap<String, String>,
    pub settings: HashMap<String, String>,
    pub env: HashMap<String, String>,
    pub scripts: HashMap<String, String>,
}

/// Tool installation status
#[derive(Debug, Clone)]
pub struct ToolStatus {
    pub name: String,
    pub version: String,
    pub installed: bool,
    pub path: Option<PathBuf>,
}

/// Handle the setup command
pub async fn handle(
    registry: &ProviderRegistry,
    force: bool,
    dry_run: bool,
    verbose: bool,
    no_parallel: bool,
) -> Result<()> {
    let start_time = Instant::now();
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    // Find and parse .vx.toml
    let config_path = find_vx_config(&current_dir)?;
    let config = parse_vx_config(&config_path)?;

    UI::header("🚀 VX Development Environment Setup");
    println!();

    if config.tools.is_empty() {
        UI::warn("No tools configured in .vx.toml");
        UI::hint("Add tools to the [tools] section in .vx.toml");
        return Ok(());
    }

    // Check current status
    UI::info("Checking tool status...");
    let tool_statuses = check_tool_status(&config.tools, registry).await?;

    // Show status
    println!();
    println!("Tools:");
    let mut tools_to_install = Vec::new();

    for status in &tool_statuses {
        let status_icon = if status.installed { "✓" } else { "✗" };
        let status_text = if status.installed {
            "installed"
        } else {
            "missing"
        };

        println!(
            "  {} {}@{} ({})",
            status_icon, status.name, status.version, status_text
        );

        if !status.installed || force {
            tools_to_install.push(status.clone());
        }
    }
    println!();

    if tools_to_install.is_empty() {
        UI::success("All tools are already installed!");
        show_next_steps(&config);
        return Ok(());
    }

    if dry_run {
        UI::info(&format!(
            "Would install {} tool(s):",
            tools_to_install.len()
        ));
        for tool in &tools_to_install {
            println!("  - {}@{}", tool.name, tool.version);
        }
        return Ok(());
    }

    // Install missing tools
    UI::info(&format!("Installing {} tool(s)...", tools_to_install.len()));
    println!();

    let install_results = if no_parallel {
        install_tools_sequential(&tools_to_install, verbose).await?
    } else {
        install_tools_parallel(&tools_to_install, verbose).await?
    };

    // Show results
    println!();
    let successful = install_results.iter().filter(|(_, ok)| *ok).count();
    let failed = install_results.len() - successful;

    if failed == 0 {
        UI::success(&format!(
            "Successfully installed {} tool(s) in {:.1}s",
            successful,
            start_time.elapsed().as_secs_f64()
        ));
    } else {
        UI::warn(&format!(
            "Installed {}/{} tools ({} failed)",
            successful,
            install_results.len(),
            failed
        ));
    }

    // Show failed tools
    for (tool, ok) in &install_results {
        if !ok {
            UI::error(&format!("  Failed: {}@{}", tool.name, tool.version));
        }
    }

    show_next_steps(&config);
    Ok(())
}

/// Find .vx.toml in current directory or parent directories
fn find_vx_config(start_dir: &Path) -> Result<PathBuf> {
    let mut current = start_dir.to_path_buf();

    loop {
        let config_path = current.join(".vx.toml");
        if config_path.exists() {
            return Ok(config_path);
        }

        if !current.pop() {
            break;
        }
    }

    Err(anyhow::anyhow!(
        "No .vx.toml found in current directory or parent directories.\n\
         Run 'vx init' to create one."
    ))
}

/// Parse .vx.toml configuration
pub fn parse_vx_config(path: &Path) -> Result<VxConfig> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;

    let mut config = VxConfig::default();
    let mut current_section = String::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Section header
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].to_string();
            continue;
        }

        // Key-value pair
        if let Some((key, value)) = parse_key_value(line) {
            match current_section.as_str() {
                "tools" => {
                    config.tools.insert(key, value);
                }
                "settings" => {
                    config.settings.insert(key, value);
                }
                "env" => {
                    config.env.insert(key, value);
                }
                "scripts" => {
                    config.scripts.insert(key, value);
                }
                _ => {}
            }
        }
    }

    Ok(config)
}

/// Parse a key = "value" line
fn parse_key_value(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() != 2 {
        return None;
    }

    let key = parts[0].trim().to_string();
    let value = parts[1]
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string();

    Some((key, value))
}

/// Check the installation status of all tools
async fn check_tool_status(
    tools: &HashMap<String, String>,
    _registry: &ProviderRegistry,
) -> Result<Vec<ToolStatus>> {
    let path_manager = PathManager::new()?;
    let mut statuses = Vec::new();

    for (name, version) in tools {
        let (installed, path) = if version == "latest" {
            // For latest, check if any version is installed
            let versions = path_manager.list_store_versions(name)?;
            if let Some(latest) = versions.last() {
                let store_path = path_manager.version_store_dir(name, latest);
                (true, Some(store_path))
            } else {
                (false, None)
            }
        } else {
            let store_path = path_manager.version_store_dir(name, version);
            (store_path.exists(), Some(store_path))
        };

        statuses.push(ToolStatus {
            name: name.clone(),
            version: version.clone(),
            installed,
            path: if installed { path } else { None },
        });
    }

    // Sort by name
    statuses.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(statuses)
}

/// Install tools sequentially with modern progress display
async fn install_tools_sequential(
    tools: &[ToolStatus],
    _verbose: bool,
) -> Result<Vec<(ToolStatus, bool)>> {
    let mut results = Vec::new();
    let mut progress = InstallProgress::new(tools.len(), "Installing tools");

    for tool in tools {
        progress.start_tool(&tool.name, &tool.version);

        let success = install_single_tool(&tool.name, &tool.version, false).await;
        progress.complete_tool(success, &tool.name, &tool.version);
        results.push((tool.clone(), success));
    }

    let successful = results.iter().filter(|(_, ok)| *ok).count();
    progress.finish(&format!("✓ {} tools installed", successful));

    Ok(results)
}

/// Install tools in parallel with modern progress display
async fn install_tools_parallel(
    tools: &[ToolStatus],
    _verbose: bool,
) -> Result<Vec<(ToolStatus, bool)>> {
    use tokio::task::JoinSet;

    let mut join_set = JoinSet::new();
    let tools = Arc::new(tools.to_vec());

    for tool in tools.iter() {
        let tool = tool.clone();

        join_set.spawn(async move {
            let success = install_single_tool(&tool.name, &tool.version, false).await;
            (tool, success)
        });
    }

    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        if let Ok((tool, success)) = result {
            let icon = if success { "✓" } else { "✗" };
            println!("  {} {}@{}", icon, tool.name, tool.version);
            results.push((tool, success));
        }
    }

    Ok(results)
}

/// Install a single tool
async fn install_single_tool(name: &str, version: &str, _verbose: bool) -> bool {
    let exe = match env::current_exe() {
        Ok(e) => e,
        Err(_) => return false,
    };

    let mut cmd = Command::new(exe);
    cmd.args(["install", name, version]);

    // Suppress output for clean progress display
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());

    match cmd.status() {
        Ok(status) => status.success(),
        Err(_) => false,
    }
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

/// Add a tool to the project configuration
pub async fn add_tool(tool: &str, version: Option<&str>) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let config_path = current_dir.join(".vx.toml");

    if !config_path.exists() {
        return Err(anyhow::anyhow!("No .vx.toml found. Run 'vx init' first."));
    }

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

    UI::success(&format!("Added {}@{} to .vx.toml", tool, version));
    UI::hint("Run 'vx setup' to install the tool");

    Ok(())
}

/// Remove a tool from the project configuration
pub async fn remove_tool(tool: &str) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let config_path = current_dir.join(".vx.toml");

    if !config_path.exists() {
        return Err(anyhow::anyhow!("No .vx.toml found."));
    }

    let mut config = parse_vx_config(&config_path)?;

    if !config.tools.contains_key(tool) {
        UI::warn(&format!("Tool '{}' not found in configuration", tool));
        return Ok(());
    }

    config.tools.remove(tool);
    write_vx_config(&config_path, &config)?;

    UI::success(&format!("Removed '{}' from .vx.toml", tool));

    Ok(())
}

/// Update a tool version in the project configuration
pub async fn update_tool(tool: &str, version: &str) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let config_path = current_dir.join(".vx.toml");

    if !config_path.exists() {
        return Err(anyhow::anyhow!("No .vx.toml found."));
    }

    let mut config = parse_vx_config(&config_path)?;

    let old_version = config.tools.get(tool).cloned();
    config.tools.insert(tool.to_string(), version.to_string());
    write_vx_config(&config_path, &config)?;

    if let Some(old) = old_version {
        UI::success(&format!("Updated {} from {} to {}", tool, old, version));
    } else {
        UI::success(&format!("Added {}@{} to .vx.toml", tool, version));
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
            content.push_str(&format!("{} = \"{}\"\n", key, value));
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
