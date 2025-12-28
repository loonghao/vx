// Config command implementation

use crate::ui::UI;
use anyhow::Result;
use std::env;
use std::path::PathBuf;

pub async fn handle() -> Result<()> {
    UI::warning("Config command not yet implemented in new architecture");
    UI::hint("Use .vx.toml files for project configuration");
    Ok(())
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

/// Handle config validate command
pub async fn handle_validate(path: Option<String>, verbose: bool) -> Result<()> {
    use vx_config::{parse_config, validate_config, ConfigMigrator};

    let config_path = resolve_config_path(path)?;

    UI::header("🔍 Configuration Validation");
    println!();

    UI::info(&format!("Validating: {}", config_path.display()));
    println!();

    // Parse config
    let config = parse_config(&config_path)?;

    // Detect version
    let content = std::fs::read_to_string(&config_path)?;
    let migrator = ConfigMigrator::new();
    let version = migrator.detect_version(&content);

    if verbose {
        UI::info(&format!("Detected version: {}", version));
        UI::info(&format!("Tools: {}", config.tools.len()));
        UI::info(&format!("Scripts: {}", config.scripts.len()));
        UI::info(&format!("Services: {}", config.services.len()));
        println!();
    }

    // Validate
    let result = validate_config(&config);

    if !result.errors.is_empty() {
        UI::error("Validation errors:");
        for error in &result.errors {
            println!("  ✗ {}", error);
        }
        println!();
    }

    if !result.warnings.is_empty() {
        UI::warn("Validation warnings:");
        for warning in &result.warnings {
            println!("  ⚠ {}", warning);
        }
        println!();
    }

    if result.is_ok() {
        UI::success("Configuration is valid");
        if result.warnings.is_empty() {
            println!("  ✓ No errors or warnings");
        }
    } else {
        return Err(anyhow::anyhow!("Configuration validation failed"));
    }

    Ok(())
}

/// Handle config schema command
pub async fn handle_schema(output: Option<String>) -> Result<()> {
    use vx_config::schemars::schema_for;
    use vx_config::VxConfig;

    UI::header("📋 JSON Schema Generation");
    println!();

    let schema = schema_for!(VxConfig);
    let schema_json = serde_json::to_string_pretty(&schema)?;

    if let Some(output_path) = output {
        std::fs::write(&output_path, &schema_json)?;
        UI::success(&format!("Schema written to: {}", output_path));
    } else {
        println!("{}", schema_json);
    }

    Ok(())
}

/// Resolve config path from option or current directory
fn resolve_config_path(path: Option<String>) -> Result<PathBuf> {
    if let Some(p) = path {
        let path = PathBuf::from(p);
        if !path.exists() {
            return Err(anyhow::anyhow!("Config file not found: {}", path.display()));
        }
        Ok(path)
    } else {
        let current_dir = env::current_dir()?;
        let config_path = current_dir.join(".vx.toml");
        if !config_path.exists() {
            return Err(anyhow::anyhow!(
                "No .vx.toml found in current directory. Run 'vx init' to create one."
            ));
        }
        Ok(config_path)
    }
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
            return Err(anyhow::anyhow!("Unknown template: {}", template));
        }
    }

    for tool in additional_tools {
        config.push_str(&format!("[tools.{tool}]\nversion = \"latest\"\n\n"));
    }

    Ok(config)
}
