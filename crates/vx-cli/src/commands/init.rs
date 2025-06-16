// Init command implementation

use crate::ui::UI;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};

use vx_core::{Result, VxError};

pub async fn handle(
    interactive: bool,
    template: Option<String>,
    tools: Option<String>,
    force: bool,
    dry_run: bool,
    list_templates: bool,
) -> Result<()> {
    if list_templates {
        return list_available_templates();
    }

    let config_path = std::env::current_dir()
        .map_err(|e| VxError::Other {
            message: format!("Failed to get current directory: {}", e),
        })?
        .join(".vx.toml");

    // Check if config already exists
    if config_path.exists() && !force {
        UI::warn("Configuration file .vx.toml already exists");
        UI::info("Use --force to overwrite or edit the existing file");
        return Ok(());
    }

    let config_content = if interactive {
        generate_interactive_config().await?
    } else if let Some(template_name) = template {
        generate_template_config(&template_name)?
    } else if let Some(tools_str) = tools {
        generate_tools_config(&tools_str)?
    } else {
        generate_auto_detected_config().await?
    };

    if dry_run {
        UI::info("Preview of .vx.toml configuration:");
        println!();
        println!("{}", config_content);
        return Ok(());
    }

    // Write configuration file
    fs::write(&config_path, config_content).map_err(|e| VxError::Other {
        message: format!("Failed to write .vx.toml: {}", e),
    })?;

    UI::success("✅ Created .vx.toml configuration file");

    // Show next steps
    println!();
    println!("Next steps:");
    println!("  1. Review the configuration: cat .vx.toml");
    println!("  2. Install tools: vx sync");
    println!("  3. Start using tools: vx <tool> --version");
    println!();
    println!("Optional:");
    println!("  - Add to version control: git add .vx.toml");
    println!("  - Customize configuration: vx config edit --local");

    Ok(())
}

fn list_available_templates() -> Result<()> {
    UI::info("Available templates:");
    println!();
    println!("  node        - Node.js project with npm");
    println!("  python      - Python project with uv");
    println!("  rust        - Rust project with cargo");
    println!("  go          - Go project");
    println!("  fullstack   - Full-stack project (Node.js + Python)");
    println!("  minimal     - Minimal configuration");
    println!();
    println!("Usage: vx init --template <template>");
    Ok(())
}

async fn generate_interactive_config() -> Result<String> {
    UI::header("🚀 VX Project Initialization");

    // Get project name
    print!("Project name (optional): ");
    io::stdout().flush().unwrap();
    let mut project_name = String::new();
    io::stdin().read_line(&mut project_name).unwrap();
    let project_name = project_name.trim();

    // Get description
    print!("Description (optional): ");
    io::stdout().flush().unwrap();
    let mut description = String::new();
    io::stdin().read_line(&mut description).unwrap();
    let description = description.trim();

    // Select tools
    println!();
    println!("Select tools to include:");
    let available_tools = vec![
        ("node", "18.17.0", "Node.js JavaScript runtime"),
        ("npm", "latest", "Node.js package manager"),
        ("python", "3.11", "Python interpreter"),
        ("uv", "latest", "Fast Python package manager"),
        ("go", "latest", "Go programming language"),
        ("cargo", "latest", "Rust package manager"),
    ];

    let mut selected_tools = HashMap::new();
    for (tool, default_version, desc) in &available_tools {
        print!("Include {} ({})? (y/N): ", tool, desc);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if input.trim().to_lowercase().starts_with('y') {
            selected_tools.insert(tool.to_string(), default_version.to_string());
        }
    }

    if selected_tools.is_empty() {
        selected_tools.insert("node".to_string(), "18.17.0".to_string());
        UI::info("No tools selected, adding Node.js as default");
    }

    generate_config_content(project_name, description, &selected_tools, true)
}

fn generate_template_config(template_name: &str) -> Result<String> {
    let tools = match template_name {
        "node" => {
            let mut tools = HashMap::new();
            tools.insert("node".to_string(), "18.17.0".to_string());
            tools.insert("npm".to_string(), "latest".to_string());
            tools
        }
        "python" => {
            let mut tools = HashMap::new();
            tools.insert("python".to_string(), "3.11".to_string());
            tools.insert("uv".to_string(), "latest".to_string());
            tools
        }
        "rust" => {
            let mut tools = HashMap::new();
            tools.insert("cargo".to_string(), "latest".to_string());
            tools
        }
        "go" => {
            let mut tools = HashMap::new();
            tools.insert("go".to_string(), "latest".to_string());
            tools
        }
        "fullstack" => {
            let mut tools = HashMap::new();
            tools.insert("node".to_string(), "18.17.0".to_string());
            tools.insert("python".to_string(), "3.11".to_string());
            tools.insert("uv".to_string(), "latest".to_string());
            tools
        }
        "minimal" => HashMap::new(),
        _ => {
            return Err(VxError::Other {
                message: format!(
                    "Unknown template: {}. Use --list-templates to see available templates.",
                    template_name
                ),
            });
        }
    };

    generate_config_content("", "", &tools, false)
}

fn generate_tools_config(tools_str: &str) -> Result<String> {
    let mut tools = HashMap::new();

    for tool_spec in tools_str.split(',') {
        let tool_spec = tool_spec.trim();
        if tool_spec.contains('@') {
            let parts: Vec<&str> = tool_spec.split('@').collect();
            if parts.len() == 2 {
                tools.insert(parts[0].to_string(), parts[1].to_string());
            }
        } else {
            tools.insert(tool_spec.to_string(), "latest".to_string());
        }
    }

    generate_config_content("", "", &tools, false)
}

async fn generate_auto_detected_config() -> Result<String> {
    let current_dir = std::env::current_dir().map_err(|e| VxError::Other {
        message: format!("Failed to get current directory: {}", e),
    })?;

    let mut tools = HashMap::new();
    let mut detected_types = Vec::new();

    // Check for Node.js project
    if current_dir.join("package.json").exists() {
        tools.insert("node".to_string(), "18.17.0".to_string());
        tools.insert("npm".to_string(), "latest".to_string());
        detected_types.push("Node.js");
        UI::info("🔍 Detected Node.js project (package.json found)");
    }

    // Check for Python project
    if current_dir.join("pyproject.toml").exists() || current_dir.join("requirements.txt").exists()
    {
        tools.insert("python".to_string(), "3.11".to_string());
        tools.insert("uv".to_string(), "latest".to_string());
        detected_types.push("Python");
        UI::info("🔍 Detected Python project");
    }

    // Check for Go project
    if current_dir.join("go.mod").exists() {
        tools.insert("go".to_string(), "latest".to_string());
        detected_types.push("Go");
        UI::info("🔍 Detected Go project (go.mod found)");
    }

    // Check for Rust project
    if current_dir.join("Cargo.toml").exists() {
        tools.insert("cargo".to_string(), "latest".to_string());
        detected_types.push("Rust");
        UI::info("🔍 Detected Rust project (Cargo.toml found)");
    }

    if tools.is_empty() {
        UI::info("No project type detected, creating minimal configuration");
        tools.insert("node".to_string(), "18.17.0".to_string());
    } else if detected_types.len() > 1 {
        UI::info(&format!(
            "🔍 Detected mixed project ({})",
            detected_types.join(" + ")
        ));
    }

    generate_config_content("", "", &tools, false)
}

fn generate_config_content(
    project_name: &str,
    description: &str,
    tools: &HashMap<String, String>,
    include_extras: bool,
) -> Result<String> {
    let mut content = String::new();

    // Header comment
    content.push_str("# VX Project Configuration\n");
    content.push_str("# This file defines the tools and versions required for this project.\n");
    content.push_str("# Run 'vx sync' to install all required tools.\n");

    if !project_name.is_empty() {
        content.push_str(&format!("# Project: {}\n", project_name));
    }
    if !description.is_empty() {
        content.push_str(&format!("# Description: {}\n", description));
    }

    content.push('\n');

    // Tools section
    content.push_str("[tools]\n");
    if tools.is_empty() {
        content.push_str("# Add your tools here, for example:\n");
        content.push_str("# node = \"18.17.0\"\n");
        content.push_str("# python = \"3.11\"\n");
        content.push_str("# uv = \"latest\"\n");
    } else {
        for (tool, version) in tools {
            content.push_str(&format!("{} = \"{}\"\n", tool, version));
        }
    }

    content.push('\n');

    // Settings section
    content.push_str("[settings]\n");
    content.push_str("auto_install = true\n");
    content.push_str("cache_duration = \"7d\"\n");

    if include_extras {
        content.push_str("parallel_install = true\n");
        content.push('\n');

        // Scripts section
        content.push_str("[scripts]\n");
        content.push_str("# Add custom scripts here\n");
        content.push_str("# dev = \"vx node server.js\"\n");
        content.push_str("# test = \"vx uv run pytest\"\n");
        content.push('\n');

        // Environment section
        content.push_str("[env]\n");
        content.push_str("# Add environment variables here\n");
        content.push_str("# NODE_ENV = \"development\"\n");
        content.push_str("# DEBUG = \"true\"\n");
    }

    Ok(content)
}
