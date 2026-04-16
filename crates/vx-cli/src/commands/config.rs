// Config command implementation

use crate::cli::OutputFormat;
use crate::output::{CommandOutput, OutputRenderer};
use crate::ui::UI;
use anyhow::Result;
use serde::Serialize;
use std::env;
use std::path::PathBuf;
use vx_paths::{CONFIG_FILE_NAME, find_config_file};

#[derive(Serialize)]
struct ConfigShowOutput {
    found: bool,
    file: Option<String>,
    project: Option<String>,
    tools: usize,
    scripts: usize,
    services: usize,
    message: Option<String>,
    hint: Option<String>,
}

impl CommandOutput for ConfigShowOutput {
    fn render_text(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        if !self.found {
            writeln!(
                writer,
                "No vx.toml found in current directory or parent directories"
            )?;
            if let Some(hint) = &self.hint {
                writeln!(writer, "{}", hint)?;
            }
            return Ok(());
        }

        writeln!(writer, "Project Configuration")?;
        writeln!(writer)?;

        if let Some(file) = &self.file {
            writeln!(writer, "File: {}", file)?;
        }
        if let Some(project) = &self.project {
            writeln!(writer, "Project: {}", project)?;
        }
        writeln!(writer, "Tools: {}", self.tools)?;
        writeln!(writer, "Scripts: {}", self.scripts)?;
        writeln!(writer, "Services: {}", self.services)?;

        Ok(())
    }

    fn render_compact(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        if !self.found {
            writeln!(writer, "err config missing")?;
            return Ok(());
        }

        let project = self.project.as_deref().unwrap_or("-");
        writeln!(
            writer,
            "config project:{} tools:{} scripts:{} services:{}",
            project, self.tools, self.scripts, self.services
        )?;
        Ok(())
    }
}

pub async fn handle(format: OutputFormat) -> Result<()> {
    use vx_config::parse_config;

    let current_dir = env::current_dir()?;
    let renderer = OutputRenderer::new(format);

    let Some(config_path) = find_config_file(&current_dir) else {
        let output = ConfigShowOutput {
            found: false,
            file: None,
            project: None,
            tools: 0,
            scripts: 0,
            services: 0,
            message: Some(
                "No vx.toml found in current directory or parent directories".to_string(),
            ),
            hint: Some("Run 'vx init' to create one".to_string()),
        };

        if renderer.is_text() {
            UI::warning("No vx.toml found in current directory or parent directories");
            UI::hint("Run 'vx init' to create one");
        } else {
            renderer.render(&output)?;
        }
        return Ok(());
    };

    let config = parse_config(&config_path)?;

    let output = ConfigShowOutput {
        found: true,
        file: Some(config_path.display().to_string()),
        project: config
            .project
            .as_ref()
            .and_then(|project| project.name.clone()),
        tools: config.tools_as_hashmap().len(),
        scripts: config.scripts.len(),
        services: config.services.len(),
        message: None,
        hint: None,
    };

    if renderer.is_text() {
        UI::header("📋 Project Configuration");
        println!();
        if let Some(file) = &output.file {
            UI::info(&format!("File: {}", file));
        }
        if let Some(project) = &output.project {
            UI::info(&format!("Project: {}", project));
        }
        UI::info(&format!("Tools: {}", output.tools));
        UI::info(&format!("Scripts: {}", output.scripts));
        UI::info(&format!("Services: {}", output.services));
    } else {
        renderer.render(&output)?;
    }

    Ok(())
}

pub async fn handle_init(tools: Vec<String>, template: Option<String>) -> Result<()> {
    let spinner = UI::new_spinner("Initializing configuration...");

    let config_content = if let Some(template) = template {
        generate_template_config(&template, &tools)?
    } else {
        generate_default_config(&tools)?
    };

    std::fs::write(CONFIG_FILE_NAME, config_content)?;
    spinner.finish_and_clear();

    UI::success(&format!(
        "Initialized {} in current directory",
        CONFIG_FILE_NAME
    ));
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
    UI::hint("Manually edit vx.toml files for now");
    Ok(())
}

/// Handle config validate command
pub async fn handle_validate(path: Option<String>, verbose: bool) -> Result<()> {
    use vx_config::{ConfigMigrator, parse_config, validate_config};

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
    use vx_config::VxConfig;
    use vx_config::schemars::schema_for;

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

/// Handle config dir command - show configuration directory path
pub async fn handle_dir() -> Result<()> {
    let paths = vx_paths::VxPaths::new()?;
    println!("{}", paths.config_dir.display());
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
        find_config_file(&current_dir).ok_or_else(|| {
            anyhow::anyhow!(
                "No {} found in current directory. Run 'vx init' to create one.",
                CONFIG_FILE_NAME
            )
        })
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
