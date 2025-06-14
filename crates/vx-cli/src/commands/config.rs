// Config command implementation

use crate::ui::UI;
use vx_core::{Result, VxError};

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
    // TODO: Replace with vx-core tool manager
    // let config_manager = crate::tool_manager::ToolManager::new()
    //     .or_else(|_| crate::tool_manager::ToolManager::minimal())?;
    // let status = config_manager.config().get_status();
    spinner.finish_and_clear();

    UI::header("Current Configuration");
    UI::warning("Config show not yet implemented in new architecture");
    // println!("Status: {}", status.summary());

    // if let Some(project_info) = &status.project_info {
    //     println!("Project type: {:?}", project_info.project_type);
    //     println!("Config file: {}", project_info.config_file.display());
    // }

    Ok(())
}

#[allow(dead_code)]
async fn set_config(_key: &str, _value: &str) -> Result<()> {
    UI::warning("Config set not yet implemented");
    Ok(())
}

#[allow(dead_code)]
async fn get_config(_key: &str) -> Result<()> {
    UI::warning("Config get not yet implemented");
    Ok(())
}

#[allow(dead_code)]
async fn reset_config(_key: Option<String>) -> Result<()> {
    UI::warning("Config reset not yet implemented");
    Ok(())
}

#[allow(dead_code)]
async fn edit_config() -> Result<()> {
    UI::warning("Config edit not yet implemented");
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
