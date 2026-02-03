//! Tests for verifying init command generates valid vx.toml
//!
//! These tests ensure that the output of `vx init` can be correctly parsed.

use std::collections::HashMap;
use vx_config::config_manager::TomlWriter;
use vx_config::parse_config_str;

/// Simulate the config generation from init.rs
fn generate_config_content(
    project_name: &str,
    description: &str,
    detected_tools: &HashMap<String, String>,
    detected_scripts: &HashMap<String, String>,
    include_extras: bool,
) -> String {
    let mut writer = TomlWriter::new();

    // Header comments
    writer = writer
        .comment("VX Project Configuration")
        .comment("This file defines the tools and versions required for this project.")
        .comment("Run 'vx setup' to install all required tools.")
        .comment("Run 'vx dev' to enter the development environment.");

    if !project_name.is_empty() {
        writer = writer.comment(&format!("Project: {}", project_name));
    }
    if !description.is_empty() {
        writer = writer.comment(&format!("Description: {}", description));
    }

    // Tools section
    writer = writer.section("tools");
    if detected_tools.is_empty() {
        writer = writer
            .comment("Add your tools here, for example:")
            .comment("node = \"20\"")
            .comment("python = \"3.12\"")
            .comment("uv = \"latest\"");
    } else {
        writer = writer.kv_map_sorted(detected_tools);
    }

    // Settings section
    writer = writer.section("settings");
    writer = writer
        .comment("Automatically install missing tools when entering dev environment")
        .kv_bool("auto_install", true)
        .comment("Cache duration for version checks")
        .kv("cache_duration", "7d");

    if include_extras {
        writer = writer
            .comment("Install tools in parallel")
            .kv_bool("parallel_install", true);
    }

    // Scripts section
    if !detected_scripts.is_empty() {
        writer = writer.section("scripts").kv_map_sorted(detected_scripts);
    } else if include_extras {
        writer = writer
            .section("scripts")
            .comment("Define custom scripts that can be run with 'vx run <script>'")
            .comment("dev = \"npm run dev\"")
            .comment("test = \"npm test\"")
            .comment("build = \"npm run build\"");
    }

    writer.build()
}

#[test]
fn test_init_generates_valid_toml() {
    let mut tools = HashMap::new();
    tools.insert("node".to_string(), "20".to_string());
    tools.insert("npm".to_string(), "latest".to_string());

    let scripts = HashMap::new();

    let config_content =
        generate_config_content("test-project", "Test description", &tools, &scripts, false);

    println!("Generated TOML:\n{}", config_content);

    // Should be valid TOML
    let parsed = parse_config_str(&config_content);
    assert!(
        parsed.is_ok(),
        "Generated config should be valid TOML: {:?}",
        parsed.err()
    );

    let config = parsed.unwrap();
    assert_eq!(config.get_tool_version("node"), Some("20".to_string()));
    assert_eq!(config.get_tool_version("npm"), Some("latest".to_string()));
}

#[test]
fn test_init_generates_valid_toml_with_extras() {
    let mut tools = HashMap::new();
    tools.insert("node".to_string(), "20".to_string());

    let mut scripts = HashMap::new();
    scripts.insert("build".to_string(), "npm run build".to_string());
    scripts.insert("test".to_string(), "npm test".to_string());

    let config_content = generate_config_content("test-project", "", &tools, &scripts, true);

    println!("Generated TOML with extras:\n{}", config_content);

    // Should be valid TOML
    let parsed = parse_config_str(&config_content);
    assert!(
        parsed.is_ok(),
        "Generated config with extras should be valid TOML: {:?}",
        parsed.err()
    );

    let config = parsed.unwrap();
    assert_eq!(config.get_tool_version("node"), Some("20".to_string()));
    assert_eq!(
        config.get_script_command("build"),
        Some("npm run build".to_string())
    );
    assert_eq!(
        config.get_script_command("test"),
        Some("npm test".to_string())
    );
}

#[test]
fn test_init_empty_tools_generates_valid_toml() {
    let tools = HashMap::new();
    let scripts = HashMap::new();

    let config_content = generate_config_content("", "", &tools, &scripts, false);

    println!("Generated TOML (empty tools):\n{}", config_content);

    // Should be valid TOML even with empty tools
    let parsed = parse_config_str(&config_content);
    assert!(
        parsed.is_ok(),
        "Generated config with empty tools should be valid TOML: {:?}",
        parsed.err()
    );
}

#[test]
fn test_init_with_special_characters_in_scripts() {
    let mut tools = HashMap::new();
    tools.insert("node".to_string(), "20".to_string());

    let mut scripts = HashMap::new();
    scripts.insert("mcp:build".to_string(), "npm run mcp:build".to_string());
    scripts.insert("dev:server".to_string(), "npm run dev:server".to_string());

    let config_content = generate_config_content("", "", &tools, &scripts, false);

    println!("Generated TOML with special keys:\n{}", config_content);

    // Should be valid TOML
    let parsed = parse_config_str(&config_content);
    assert!(
        parsed.is_ok(),
        "Generated config with special keys should be valid TOML: {:?}",
        parsed.err()
    );

    let config = parsed.unwrap();
    assert_eq!(
        config.get_script_command("mcp:build"),
        Some("npm run mcp:build".to_string())
    );
    assert_eq!(
        config.get_script_command("dev:server"),
        Some("npm run dev:server".to_string())
    );
}
