//! Configuration parsing tests
//!
//! Tests for parsing `vx.toml` configuration files.

use rstest::rstest;
use vx_config::{parse_config, parse_config_str};

// ============================================
// Basic Parsing Tests
// ============================================

#[test]
fn test_parse_empty_config() {
    let content = "";
    let config = parse_config_str(content).unwrap();
    assert!(config.tools.is_empty());
    assert!(config.scripts.is_empty());
}

#[test]
fn test_parse_minimal_tools_config() {
    let content = r#"
[tools]
node = "20"
"#;
    let config = parse_config_str(content).unwrap();
    assert_eq!(config.get_tool_version("node"), Some("20".to_string()));
}

#[test]
fn test_parse_multiple_tools() {
    let content = r#"
[tools]
node = "20"
uv = "0.5"
go = "1.22"
"#;
    let config = parse_config_str(content).unwrap();
    assert_eq!(config.get_tool_version("node"), Some("20".to_string()));
    assert_eq!(config.get_tool_version("uv"), Some("0.5".to_string()));
    assert_eq!(config.get_tool_version("go"), Some("1.22".to_string()));
}

#[rstest]
#[case("latest", "latest")]
#[case("20", "20")]
#[case("20.0", "20.0")]
#[case("20.0.0", "20.0.0")]
#[case("^20.0.0", "^20.0.0")]
#[case(">=18.0.0", ">=18.0.0")]
fn test_parse_version_formats(#[case] input: &str, #[case] expected: &str) {
    let content = format!(
        r#"
[tools]
node = "{}"
"#,
        input
    );
    let config = parse_config_str(&content).unwrap();
    assert_eq!(config.get_tool_version("node"), Some(expected.to_string()));
}

// ============================================
// Detailed Tool Configuration Tests
// ============================================

#[test]
fn test_parse_detailed_tool_config() {
    let content = r#"
[tools.node]
version = "20"
postinstall = "corepack enable"
"#;
    let config = parse_config_str(content).unwrap();
    assert_eq!(config.get_tool_version("node"), Some("20".to_string()));
}

#[test]
fn test_parse_tool_with_os_restriction() {
    let content = r#"
[tools.node]
version = "20"
os = ["linux", "darwin"]
"#;
    let config = parse_config_str(content).unwrap();
    assert_eq!(config.get_tool_version("node"), Some("20".to_string()));

    // Check OS restriction
    if let Some(vx_config::ToolVersion::Detailed(tool)) = config.tools.get("node") {
        assert_eq!(
            tool.os,
            Some(vec!["linux".to_string(), "darwin".to_string()])
        );
    } else {
        panic!("Expected detailed tool config");
    }
}

#[test]
fn test_parse_tool_with_install_env() {
    let content = r#"
[tools.node]
version = "20"
install_env = { NODE_OPTIONS = "--max-old-space-size=4096" }
"#;
    let config = parse_config_str(content).unwrap();

    if let Some(vx_config::ToolVersion::Detailed(tool)) = config.tools.get("node") {
        let env = tool.install_env.as_ref().unwrap();
        assert_eq!(
            env.get("NODE_OPTIONS"),
            Some(&"--max-old-space-size=4096".to_string())
        );
    } else {
        panic!("Expected detailed tool config");
    }
}

// ============================================
// Script Configuration Tests
// ============================================

#[test]
fn test_parse_simple_scripts() {
    let content = r#"
[scripts]
dev = "npm run dev"
test = "npm test"
build = "npm run build"
"#;
    let config = parse_config_str(content).unwrap();
    assert_eq!(
        config.get_script_command("dev"),
        Some("npm run dev".to_string())
    );
    assert_eq!(
        config.get_script_command("test"),
        Some("npm test".to_string())
    );
    assert_eq!(
        config.get_script_command("build"),
        Some("npm run build".to_string())
    );
}

#[test]
fn test_parse_detailed_script_config() {
    let content = r#"
[scripts.start]
command = "python main.py"
description = "Start the server"
args = ["--host", "0.0.0.0"]
cwd = "src"
"#;
    let config = parse_config_str(content).unwrap();
    assert_eq!(
        config.get_script_command("start"),
        Some("python main.py".to_string())
    );

    if let Some(vx_config::ScriptConfig::Detailed(script)) = config.scripts.get("start") {
        assert_eq!(script.description, Some("Start the server".to_string()));
        assert_eq!(
            script.args,
            vec!["--host".to_string(), "0.0.0.0".to_string()]
        );
        assert_eq!(script.cwd, Some("src".to_string()));
    } else {
        panic!("Expected detailed script config");
    }
}

#[test]
fn test_parse_script_with_env() {
    let content = r#"
[scripts.dev]
command = "npm run dev"
env = { NODE_ENV = "development", DEBUG = "true" }
"#;
    let config = parse_config_str(content).unwrap();

    if let Some(vx_config::ScriptConfig::Detailed(script)) = config.scripts.get("dev") {
        let env = &script.env;
        assert_eq!(env.get("NODE_ENV"), Some(&"development".to_string()));
        assert_eq!(env.get("DEBUG"), Some(&"true".to_string()));
    } else {
        panic!("Expected detailed script config");
    }
}

// ============================================
// Environment Variables Tests
// ============================================

#[test]
fn test_parse_env_config() {
    let content = r#"
[env]
NODE_ENV = "development"
DEBUG = "true"
API_URL = "http://localhost:3000"
"#;
    let config = parse_config_str(content).unwrap();
    let env = config.env.as_ref().unwrap();
    assert_eq!(env.vars.get("NODE_ENV"), Some(&"development".to_string()));
    assert_eq!(env.vars.get("DEBUG"), Some(&"true".to_string()));
}

// ============================================
// Settings Configuration Tests
// ============================================

#[test]
fn test_parse_settings() {
    let content = r#"
[settings]
auto_install = true
cache_duration = "7d"
"#;
    let config = parse_config_str(content).unwrap();
    let settings = config.settings.as_ref().unwrap();
    assert_eq!(settings.auto_install, Some(true));
    assert_eq!(settings.cache_duration, Some("7d".to_string()));
}

// ============================================
// Hooks Configuration Tests
// ============================================

#[test]
fn test_parse_hooks_single_command() {
    let content = r#"
[hooks]
pre_setup = "echo 'Starting setup...'"
post_setup = "echo 'Setup complete!'"
"#;
    let config = parse_config_str(content).unwrap();
    let hooks = config.hooks.as_ref().unwrap();
    assert!(hooks.pre_setup.is_some());
    assert!(hooks.post_setup.is_some());
}

#[test]
fn test_parse_hooks_multiple_commands() {
    let content = r#"
[hooks]
post_setup = ["vx run migrate", "vx run seed"]
"#;
    let config = parse_config_str(content).unwrap();
    let hooks = config.hooks.as_ref().unwrap();

    if let Some(vx_config::HookCommand::Multiple(cmds)) = &hooks.post_setup {
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0], "vx run migrate");
        assert_eq!(cmds[1], "vx run seed");
    } else {
        panic!("Expected multiple hook commands");
    }
}

// ============================================
// Services Configuration Tests
// ============================================

#[test]
fn test_parse_services() {
    let content = r#"
[services.database]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_PASSWORD = "secret" }

[services.redis]
image = "redis:7"
ports = ["6379:6379"]
"#;
    let config = parse_config_str(content).unwrap();
    assert_eq!(config.services.len(), 2);
    assert!(config.services.contains_key("database"));
    assert!(config.services.contains_key("redis"));

    let db = config.services.get("database").unwrap();
    assert_eq!(db.image, Some("postgres:16".to_string()));
    assert_eq!(db.ports, vec!["5432:5432".to_string()]);
}

#[test]
fn test_parse_service_with_volumes() {
    let content = r#"
[services.database]
image = "postgres:16"
volumes = ["./data:/var/lib/postgresql/data"]
"#;
    let config = parse_config_str(content).unwrap();
    let db = config.services.get("database").unwrap();
    assert_eq!(
        db.volumes,
        vec!["./data:/var/lib/postgresql/data".to_string()]
    );
}

// ============================================
// Project Metadata Tests
// ============================================

#[test]
fn test_parse_project_metadata() {
    let content = r#"
[project]
name = "my-app"
description = "My awesome application"
version = "1.0.0"
license = "MIT"
repository = "https://github.com/user/my-app"
"#;
    let config = parse_config_str(content).unwrap();
    let project = config.project.as_ref().unwrap();
    assert_eq!(project.name, Some("my-app".to_string()));
    assert_eq!(
        project.description,
        Some("My awesome application".to_string())
    );
    assert_eq!(project.version, Some("1.0.0".to_string()));
    assert_eq!(project.license, Some("MIT".to_string()));
}

// ============================================
// Full Configuration Tests
// ============================================

#[test]
fn test_parse_full_v1_config() {
    let content = r#"
[tools]
node = "20"
uv = "latest"

[env]
NODE_ENV = "development"

[scripts]
dev = "npm run dev"
test = "npm test"

[settings]
auto_install = true
cache_duration = "7d"
"#;
    let config = parse_config_str(content).unwrap();

    // Verify tools
    assert_eq!(config.get_tool_version("node"), Some("20".to_string()));
    assert_eq!(config.get_tool_version("uv"), Some("latest".to_string()));

    // Verify scripts
    assert_eq!(
        config.get_script_command("dev"),
        Some("npm run dev".to_string())
    );

    // Verify settings
    let settings = config.settings.as_ref().unwrap();
    assert_eq!(settings.auto_install, Some(true));
}

#[test]
fn test_parse_full_v2_config() {
    let content = r#"
min_version = "0.6.0"

[project]
name = "my-fullstack-app"
description = "A full-stack application"

[tools]
node = "20"
uv = "0.5"

[tools.go]
version = "1.22"
postinstall = "go install golang.org/x/tools/gopls@latest"

[env]
NODE_ENV = "development"

[scripts]
dev = "npm run dev"

[scripts.test]
command = "npm test"
description = "Run tests"

[hooks]
pre_setup = "echo 'Starting...'"
post_setup = ["npm install", "npm run build"]

[services.database]
image = "postgres:16"
ports = ["5432:5432"]

[settings]
auto_install = true
"#;
    let config = parse_config_str(content).unwrap();

    // Verify min_version
    assert_eq!(config.min_version, Some("0.6.0".to_string()));

    // Verify project
    let project = config.project.as_ref().unwrap();
    assert_eq!(project.name, Some("my-fullstack-app".to_string()));

    // Verify tools (both simple and detailed)
    assert_eq!(config.get_tool_version("node"), Some("20".to_string()));
    assert_eq!(config.get_tool_version("go"), Some("1.22".to_string()));

    // Verify hooks
    assert!(config.hooks.is_some());

    // Verify services
    assert_eq!(config.services.len(), 1);
}

// ============================================
// Backward Compatibility Tests
// ============================================

#[test]
fn test_backward_compatible_tools_hashmap() {
    let content = r#"
[tools]
python = "3.11"
uv = "latest"
node = "20"
"#;
    let config = parse_config_str(content).unwrap();
    let tools = config.tools_as_hashmap();

    assert_eq!(tools.get("python"), Some(&"3.11".to_string()));
    assert_eq!(tools.get("uv"), Some(&"latest".to_string()));
    assert_eq!(tools.get("node"), Some(&"20".to_string()));
}

#[test]
fn test_backward_compatible_scripts_hashmap() {
    let content = r#"
[scripts]
dev = "npm run dev"
test = "npm test"
"#;
    let config = parse_config_str(content).unwrap();
    let scripts = config.scripts_as_hashmap();

    assert_eq!(scripts.get("dev"), Some(&"npm run dev".to_string()));
    assert_eq!(scripts.get("test"), Some(&"npm test".to_string()));
}

#[test]
fn test_backward_compatible_settings_hashmap() {
    let content = r#"
[settings]
auto_install = true
cache_duration = "7d"
"#;
    let config = parse_config_str(content).unwrap();
    let settings = config.settings_as_hashmap();

    assert_eq!(settings.get("auto_install"), Some(&"true".to_string()));
    assert_eq!(settings.get("cache_duration"), Some(&"7d".to_string()));
}

// ============================================
// Error Handling Tests
// ============================================

#[test]
fn test_parse_invalid_toml() {
    let content = r#"
[tools
node = "20"
"#;
    let result = parse_config_str(content);
    assert!(result.is_err());
}

#[test]
fn test_parse_nonexistent_file() {
    let result = parse_config("/nonexistent/path/vx.toml");
    assert!(result.is_err());
}

// ============================================
// Python Configuration Tests
// ============================================

#[test]
fn test_parse_python_config() {
    let content = r#"
[python]
version = "3.11"
venv = ".venv"
package_manager = "uv"

[python.dependencies]
requirements = ["requirements.txt"]
packages = ["requests", "flask"]
dev = ["pytest", "black"]
"#;
    let config = parse_config_str(content).unwrap();
    let python = config.python.as_ref().unwrap();

    assert_eq!(python.version, Some("3.11".to_string()));
    assert_eq!(python.venv, Some(".venv".to_string()));
    assert_eq!(python.package_manager, Some("uv".to_string()));

    let deps = python.dependencies.as_ref().unwrap();
    assert_eq!(deps.requirements, vec!["requirements.txt".to_string()]);
    assert_eq!(
        deps.packages,
        vec!["requests".to_string(), "flask".to_string()]
    );
}

// ============================================
// Dependencies Configuration Tests
// ============================================

#[test]
fn test_parse_dependencies_config() {
    let content = r#"
[dependencies]
lockfile = true
auto_update = "weekly"
"#;
    let config = parse_config_str(content).unwrap();
    let deps = config.dependencies.as_ref().unwrap();

    assert_eq!(deps.lockfile, Some(true));
    assert_eq!(deps.auto_update, Some("weekly".to_string()));
}
