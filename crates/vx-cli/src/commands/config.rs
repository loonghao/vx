//! Configuration management commands

use crate::ui::UI;
use anyhow::Result;
use clap::{Args, Subcommand};
use vx_config::{get_current_platform, load_default_config, ConfigManager};

/// Configuration management commands
#[derive(Debug, Args)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub action: ConfigAction,
}

/// Configuration actions
#[derive(Debug, Clone, Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show {
        /// Show only specific tool configuration
        #[arg(short, long)]
        tool: Option<String>,
        /// Show raw configuration (JSON format)
        #[arg(long)]
        raw: bool,
    },
    /// Set configuration value
    Set {
        /// Configuration key (e.g., defaults.auto_install)
        key: String,
        /// Configuration value
        value: String,
    },
    /// Get configuration value
    Get {
        /// Configuration key (e.g., defaults.auto_install)
        key: String,
    },
    /// Reset configuration to defaults
    Reset {
        /// Reset only specific tool configuration
        #[arg(short, long)]
        tool: Option<String>,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
    /// Show configuration status and diagnostics
    Status,
    /// Show default configuration
    Defaults,
    /// Show platform information
    Platform,
    /// Initialize project configuration
    Init {
        /// Tools to include in configuration
        tools: Vec<String>,
        /// Template to use
        #[arg(short, long)]
        template: Option<String>,
    },
}

impl ConfigCommand {
    pub async fn execute(&self) -> Result<()> {
        match &self.action {
            ConfigAction::Show { tool, raw } => show_config(tool.as_deref(), *raw).await,
            ConfigAction::Set { key, value } => handle_set(key, value).await,
            ConfigAction::Get { key } => handle_get(key).await,
            ConfigAction::Reset { tool, yes } => reset_config(tool.as_deref(), *yes).await,
            ConfigAction::Status => show_config_status().await,
            ConfigAction::Defaults => show_default_config().await,
            ConfigAction::Platform => show_platform_info().await,
            ConfigAction::Init { tools, template } => {
                handle_init(tools.clone(), template.clone()).await
            }
        }
    }
}

// Legacy function for backward compatibility
pub async fn handle() -> Result<()> {
    show_config_status().await
}

pub async fn handle_init(tools: Vec<String>, template: Option<String>) -> Result<()> {
    let spinner = UI::new_spinner("Initializing configuration...");

    let config_content = if let Some(template) = template {
        generate_template_config(&template, &tools)?
    } else {
        generate_default_config(&tools)?
    };

    std::fs::write(".vx.toml", config_content)?;
    spinner.finish_and_clear();

    UI::success("Initialized .vx.toml in current directory");
    Ok(())
}

pub async fn handle_set(key: &str, value: &str) -> Result<()> {
    UI::warning("Config set command not yet implemented in new architecture");
    UI::hint(&format!("Would set {} = {}", key, value));
    Ok(())
}

pub async fn handle_get(key: &str) -> Result<()> {
    UI::warning("Config get command not yet implemented in new architecture");
    UI::hint(&format!("Would get {}", key));
    Ok(())
}

pub async fn handle_reset(key: Option<String>) -> Result<()> {
    UI::warning("Config reset command not yet implemented in new architecture");
    if let Some(key) = key {
        UI::hint(&format!("Would reset {}", key));
    } else {
        UI::hint("Would reset all configuration");
    }
    Ok(())
}

pub async fn handle_edit() -> Result<()> {
    UI::warning("Config edit command not yet implemented in new architecture");
    UI::hint("Manually edit .vx.toml files for now");
    Ok(())
}

fn generate_default_config(tools: &[String]) -> Result<String> {
    let mut config = String::from("# vx configuration file\n");
    config.push_str("# This file configures tool versions for this project\n\n");

    if tools.is_empty() {
        config.push_str("[tools.uv]\nversion = \"latest\"\n\n");
        config.push_str("[tools.node]\nversion = \"lts\"\n");
    } else {
        for tool in tools {
            config.push_str(&format!("[tools.{tool}]\nversion = \"latest\"\n\n"));
        }
    }

    Ok(config)
}

// Removed complex methods - simplified implementation

async fn show_config(tool: Option<&str>, raw: bool) -> Result<()> {
    let config_manager = ConfigManager::new().await?;
    let config = config_manager.config();

    if raw {
        // Show raw JSON configuration
        let json = serde_json::to_string_pretty(config)?;
        println!("{}", json);
        return Ok(());
    }

    if let Some(tool_name) = tool {
        // Show specific tool configuration
        if let Some(tool_config) = config.tools.get(tool_name) {
            UI::info(&format!("Configuration for tool '{}':", tool_name));
            if let Some(desc) = &tool_config.description {
                println!("  Description: {}", desc);
            }
            if let Some(homepage) = &tool_config.homepage {
                println!("  Homepage: {}", homepage);
            }
            if let Some(repo) = &tool_config.repository {
                println!("  Repository: {}", repo);
            }
            if let Some(version) = &tool_config.version {
                println!("  Version: {}", version);
            }
            if let Some(deps) = &tool_config.depends_on {
                println!("  Dependencies: {}", deps.join(", "));
            }
        } else {
            UI::error(&format!("Tool '{}' not found in configuration", tool_name));
        }
    } else {
        // Show full configuration summary
        UI::info("VX Configuration Summary");
        println!();

        println!("📁 Global Settings:");
        println!("  Home Directory: {}", config.global.home_dir);
        println!("  Tools Directory: {}", config.global.tools_dir);
        println!("  Cache Directory: {}", config.global.cache_dir);
        println!();

        println!("⚙️  Default Settings:");
        println!("  Auto Install: {}", config.defaults.auto_install);
        println!("  Cache Duration: {}", config.defaults.cache_duration);
        println!("  Use System Path: {}", config.defaults.use_system_path);
        println!("  Download Timeout: {}s", config.defaults.download_timeout);
        println!();

        println!("🚀 Turbo CDN Settings:");
        println!("  Enabled: {}", config.turbo_cdn.enabled);
        println!("  Default Region: {}", config.turbo_cdn.default_region);
        println!(
            "  Max Concurrent Chunks: {}",
            config.turbo_cdn.max_concurrent_chunks
        );
        println!("  Cache Enabled: {}", config.turbo_cdn.cache_enabled);
        println!();

        println!("🔧 Configured Tools:");
        let mut tools: Vec<_> = config.tools.keys().collect();
        tools.sort();
        for tool_name in tools {
            if let Some(tool_config) = config.tools.get(tool_name) {
                let version = tool_config.version.as_deref().unwrap_or("latest");
                let desc = tool_config
                    .description
                    .as_deref()
                    .unwrap_or("No description");
                println!("  {} ({}): {}", tool_name, version, desc);
            }
        }
    }

    Ok(())
}

async fn reset_config(tool: Option<&str>, yes: bool) -> Result<()> {
    if !yes {
        let confirmation = if let Some(tool_name) = tool {
            format!("Reset configuration for tool '{}'?", tool_name)
        } else {
            "Reset all configuration to defaults?".to_string()
        };

        if !UI::confirm(&confirmation) {
            UI::info("Configuration reset cancelled");
            return Ok(());
        }
    }

    let config_manager = ConfigManager::new().await?;

    if let Some(tool_name) = tool {
        UI::info(&format!("Resetting configuration for tool '{}'", tool_name));

        // Reset specific tool configuration
        match config_manager.reset_tool_config(tool_name).await {
            Ok(_) => {
                UI::success(&format!(
                    "Configuration for '{}' has been reset to defaults",
                    tool_name
                ));
            }
            Err(e) => {
                UI::error(&format!(
                    "Failed to reset configuration for '{}': {}",
                    tool_name, e
                ));
                return Err(e.into());
            }
        }
    } else {
        UI::info("Resetting all configuration to defaults");

        // Confirm before resetting all
        if !yes {
            UI::warning("This will reset ALL configuration to defaults!");
            UI::warning("This action cannot be undone.");

            if !dialoguer::Confirm::new()
                .with_prompt("Are you sure you want to continue?")
                .default(false)
                .interact()?
            {
                UI::info("Configuration reset cancelled");
                return Ok(());
            }
        }

        // Reset all configuration
        match config_manager.reset_all_config().await {
            Ok(_) => {
                UI::success("All configuration has been reset to defaults");
            }
            Err(e) => {
                UI::error(&format!("Failed to reset configuration: {}", e));
                return Err(e.into());
            }
        }
    }

    Ok(())
}

async fn show_config_status() -> Result<()> {
    let config_manager = ConfigManager::new().await?;
    let status = config_manager.get_status();

    UI::info("VX Configuration Status");
    println!();

    println!("📋 Configuration Layers:");
    for layer in &status.layers {
        let status_icon = if layer.available { "✅" } else { "❌" };
        println!(
            "  {} {} (priority: {})",
            status_icon, layer.name, layer.priority
        );
    }
    println!();

    println!("🔧 Available Tools: {}", status.available_tools.len());
    for tool in &status.available_tools {
        println!("  - {}", tool);
    }
    println!();

    println!(
        "⚙️  Fallback to Builtin: {}",
        if status.fallback_enabled {
            "✅ Enabled"
        } else {
            "❌ Disabled"
        }
    );

    if let Some(project_info) = &status.project_info {
        println!("📁 Project Type: {:?}", project_info.project_type);
        println!("📄 Config File: {}", project_info.config_file.display());
        if !project_info.tool_versions.is_empty() {
            println!("🔧 Project Tools:");
            for (tool, version) in &project_info.tool_versions {
                println!("  - {} @ {}", tool, version);
            }
        }
    } else {
        println!("📁 No project detected");
    }

    println!();
    println!(
        "🏥 Health: {}",
        if status.is_healthy() {
            "✅ Healthy"
        } else {
            "❌ Issues detected"
        }
    );

    Ok(())
}

async fn show_default_config() -> Result<()> {
    UI::info("VX Default Configuration");
    println!();

    let default_config = load_default_config()?;

    println!("🔧 Default Tools:");
    let mut tools: Vec<_> = default_config.tools.keys().collect();
    tools.sort();

    for tool_name in tools {
        if let Some(tool_config) = default_config.tools.get(tool_name) {
            println!("  📦 {}", tool_name);
            if let Some(desc) = &tool_config.description {
                println!("     Description: {}", desc);
            }
            if let Some(homepage) = &tool_config.homepage {
                println!("     Homepage: {}", homepage);
            }
            if let Some(deps) = &tool_config.depends_on {
                println!("     Dependencies: {}", deps.join(", "));
            }
            println!();
        }
    }

    Ok(())
}

async fn show_platform_info() -> Result<()> {
    UI::info("Platform Information");
    println!();

    let platform = get_current_platform();
    println!("🖥️  Current Platform: {}", platform);
    println!("🏗️  OS: {}", std::env::consts::OS);
    println!("🏛️  Architecture: {}", std::env::consts::ARCH);

    // Show platform-specific executable extensions
    let exe_suffix = if cfg!(target_os = "windows") {
        ".exe"
    } else {
        ""
    };
    println!(
        "📄 Executable Suffix: {}",
        if exe_suffix.is_empty() {
            "(none)"
        } else {
            exe_suffix
        }
    );

    Ok(())
}

fn generate_template_config(template: &str, additional_tools: &[String]) -> Result<String> {
    let mut config = String::from("# vx configuration file\n");
    config.push_str(&format!("# Generated from {template} template\n\n"));

    match template {
        "node" | "javascript" | "js" => {
            config.push_str("[tools.node]\nversion = \"lts\"\n\n");
            config.push_str("[tools.npm]\nversion = \"latest\"\n\n");
        }
        "python" | "py" => {
            config.push_str("[tools.uv]\nversion = \"latest\"\n\n");
            config.push_str("[tools.python]\nversion = \"3.11\"\n\n");
        }
        "rust" => {
            config.push_str("[tools.rust]\nversion = \"stable\"\n\n");
            config.push_str("[tools.cargo]\nversion = \"latest\"\n\n");
        }
        "go" => {
            config.push_str("[tools.go]\nversion = \"latest\"\n\n");
        }
        "bun" => {
            config.push_str("[tools.bun]\nversion = \"latest\"\n\n");
            config.push_str("[tools.bunx]\nversion = \"latest\"\n\n");
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown template: {}", template));
        }
    }

    for tool in additional_tools {
        config.push_str(&format!("[tools.{tool}]\nversion = \"latest\"\n\n"));
    }

    Ok(config)
}
