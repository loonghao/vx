// Config command implementation

use crate::ui::UI;
use std::collections::HashMap;
use vx_config::ConfigManager;
use vx_core::{VxError, VxResult as Result};
// TODO: Need to define ProjectConfig or import from appropriate crate

pub async fn handle() -> Result<()> {
    show_config().await
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

async fn show_config() -> Result<()> {
    let spinner = UI::new_spinner("Loading configuration...");

    let config_manager = ConfigManager::new().await?;
    let status = config_manager.get_status();

    spinner.finish_and_clear();

    UI::header("Current Configuration");

    // Show configuration layers
    UI::info("Configuration Layers:");
    for layer in &status.layers {
        let status_icon = if layer.available { "✓" } else { "✗" };
        UI::item(&format!(
            "{} {} (priority: {})",
            status_icon, layer.name, layer.priority
        ));
    }

    // Show project information if available
    if let Some(project_info) = &status.project_info {
        UI::info(&format!("Project Type: {:?}", project_info.project_type));
        UI::info(&format!(
            "Config File: {}",
            project_info.config_file.display()
        ));

        if !project_info.tool_versions.is_empty() {
            UI::info("Detected Tool Versions:");
            for (tool, version) in &project_info.tool_versions {
                UI::item(&format!("{}: {}", tool, version));
            }
        }
    }

    // Show current configuration
    let config = config_manager.config();
    if !config.tools.is_empty() {
        UI::info("Configured Tools:");
        for (tool_name, tool_config) in &config.tools {
            let version = tool_config.version.as_deref().unwrap_or("not specified");
            UI::item(&format!("{}: {}", tool_name, version));
        }
    }

    Ok(())
}

pub async fn handle_set(key: &str, value: &str) -> Result<()> {
    set_config(key, value).await
}

pub async fn handle_get(key: &str) -> Result<()> {
    get_config(key).await
}

async fn set_config(key: &str, value: &str) -> Result<()> {
    let spinner = UI::new_spinner("Updating configuration...");

    // Parse the key to determine if it's a tool config or global setting
    let parts: Vec<&str> = key.split('.').collect();

    match parts.as_slice() {
        ["tools", tool_name, "version"] => {
            // Set tool version
            set_tool_version(tool_name, value).await?;
            spinner.finish_and_clear();
            UI::success(&format!("Set {} version to {}", tool_name, value));
        }
        ["defaults", setting] => {
            // Set global default setting
            set_global_setting(setting, value).await?;
            spinner.finish_and_clear();
            UI::success(&format!("Set {} to {}", setting, value));
        }
        _ => {
            spinner.finish_and_clear();
            return Err(VxError::Other {
                message: format!(
                    "Invalid config key: {}. Use format 'tools.<tool>.version' or 'defaults.<setting>'",
                    key
                ),
            });
        }
    }

    Ok(())
}

async fn get_config(key: &str) -> Result<()> {
    let spinner = UI::new_spinner("Loading configuration...");

    let config_manager = ConfigManager::new().await?;
    let config = config_manager.config();

    spinner.finish_and_clear();

    // Parse the key to determine what to retrieve
    let parts: Vec<&str> = key.split('.').collect();

    match parts.as_slice() {
        ["tools", tool_name, "version"] => {
            if let Some(tool_config) = config.tools.get(*tool_name) {
                let version = tool_config.version.as_deref().unwrap_or("not specified");
                UI::info(&format!("{}: {}", key, version));
            } else {
                UI::warn(&format!("Tool '{}' not configured", tool_name));
            }
        }
        ["defaults", setting] => match *setting {
            "auto_install" => UI::info(&format!("{}: {}", key, config.defaults.auto_install)),
            "cache_duration" => UI::info(&format!("{}: {}", key, config.defaults.cache_duration)),
            "fallback_to_builtin" => {
                UI::info(&format!("{}: {}", key, config.defaults.fallback_to_builtin))
            }
            "use_system_path" => UI::info(&format!("{}: {}", key, config.defaults.use_system_path)),
            _ => {
                UI::warn(&format!("Unknown setting: {}", setting));
            }
        },
        _ => {
            return Err(VxError::Other {
                message: format!(
                    "Invalid config key: {}. Use format 'tools.<tool>.version' or 'defaults.<setting>'",
                    key
                ),
            });
        }
    }

    Ok(())
}

pub async fn handle_reset(key: Option<String>) -> Result<()> {
    reset_config(key).await
}

pub async fn handle_edit() -> Result<()> {
    edit_config().await
}

async fn reset_config(key: Option<String>) -> Result<()> {
    let spinner = UI::new_spinner("Resetting configuration...");

    match key {
        Some(key) => {
            // Reset specific key
            let parts: Vec<&str> = key.split('.').collect();
            match parts.as_slice() {
                ["tools", tool_name, "version"] => {
                    remove_tool_config(tool_name).await?;
                    spinner.finish_and_clear();
                    UI::success(&format!("Reset {} configuration", tool_name));
                }
                _ => {
                    spinner.finish_and_clear();
                    return Err(VxError::Other {
                        message: format!("Cannot reset key: {}", key),
                    });
                }
            }
        }
        None => {
            // Reset entire project config
            let config_path = std::env::current_dir()
                .map_err(|e| VxError::Other {
                    message: format!("Failed to get current directory: {}", e),
                })?
                .join(".vx.toml");

            if config_path.exists() {
                std::fs::remove_file(&config_path)?;
                spinner.finish_and_clear();
                UI::success("Reset project configuration (.vx.toml removed)");
            } else {
                spinner.finish_and_clear();
                UI::info("No project configuration to reset");
            }
        }
    }

    Ok(())
}

async fn edit_config() -> Result<()> {
    let config_path = std::env::current_dir()
        .map_err(|e| VxError::Other {
            message: format!("Failed to get current directory: {}", e),
        })?
        .join(".vx.toml");

    // Create config if it doesn't exist
    if !config_path.exists() {
        UI::info("Creating .vx.toml...");
        generate_default_config(&[]).and_then(|content| {
            std::fs::write(&config_path, content).map_err(|e| VxError::Other {
                message: format!("Failed to create .vx.toml: {}", e),
            })
        })?;
    }

    // Try to open with system editor
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(windows) {
                "notepad".to_string()
            } else {
                "nano".to_string()
            }
        });

    UI::info(&format!(
        "Opening {} with {}...",
        config_path.display(),
        editor
    ));

    let status = std::process::Command::new(&editor)
        .arg(&config_path)
        .status()
        .map_err(|e| VxError::Other {
            message: format!("Failed to open editor '{}': {}", editor, e),
        })?;

    if status.success() {
        UI::success("Configuration edited successfully");
    } else {
        UI::warn("Editor exited with non-zero status");
    }

    Ok(())
}

async fn remove_tool_config(tool_name: &str) -> Result<()> {
    let config_path = std::env::current_dir()
        .map_err(|e| VxError::Other {
            message: format!("Failed to get current directory: {}", e),
        })?
        .join(".vx.toml");

    if !config_path.exists() {
        return Ok(()); // Nothing to remove
    }

    let content = std::fs::read_to_string(&config_path)?;
    let mut project_config =
        toml::from_str::<vx_core::venv::ProjectConfig>(&content).map_err(|e| VxError::Other {
            message: format!("Failed to parse .vx.toml: {}", e),
        })?;

    project_config.tools.remove(tool_name);

    let toml_content = toml::to_string_pretty(&project_config).map_err(|e| VxError::Other {
        message: format!("Failed to serialize configuration: {}", e),
    })?;

    let header = "# VX Project Configuration\n# This file defines the tools and versions required for this project.\n\n";
    let full_content = format!("{}{}", header, toml_content);

    std::fs::write(&config_path, full_content).map_err(|e| VxError::Other {
        message: format!("Failed to write .vx.toml: {}", e),
    })?;

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

async fn set_tool_version(tool_name: &str, version: &str) -> Result<()> {
    // Load or create project config
    let config_path = std::env::current_dir()
        .map_err(|e| VxError::Other {
            message: format!("Failed to get current directory: {}", e),
        })?
        .join(".vx.toml");

    let mut tools = HashMap::new();

    // If config exists, load existing tools
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        if let Ok(project_config) = toml::from_str::<vx_core::venv::ProjectConfig>(&content) {
            tools = project_config.tools;
        }
    }

    // Update the tool version
    tools.insert(tool_name.to_string(), version.to_string());

    // Create updated project config
    let project_config = vx_core::venv::ProjectConfig {
        tools,
        ..Default::default()
    };

    // Write back to file
    let toml_content = toml::to_string_pretty(&project_config).map_err(|e| VxError::Other {
        message: format!("Failed to serialize configuration: {}", e),
    })?;

    let header = "# VX Project Configuration\n# This file defines the tools and versions required for this project.\n\n";
    let full_content = format!("{}{}", header, toml_content);

    std::fs::write(&config_path, full_content).map_err(|e| VxError::Other {
        message: format!("Failed to write .vx.toml: {}", e),
    })?;

    Ok(())
}

async fn set_global_setting(setting: &str, value: &str) -> Result<()> {
    // For now, just show a message that global settings aren't implemented
    // In a full implementation, this would update the global config file
    UI::warning(&format!(
        "Global setting '{}' = '{}' - Global config modification not yet implemented",
        setting, value
    ));
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
        _ => {
            return Err(VxError::Other {
                message: format!("Unknown template: {template}"),
            });
        }
    }

    for tool in additional_tools {
        config.push_str(&format!("[tools.{tool}]\nversion = \"latest\"\n\n"));
    }

    Ok(config)
}
