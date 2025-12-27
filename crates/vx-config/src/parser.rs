//! Configuration parsing

use crate::error::{ConfigError, ConfigResult};
use crate::types::VxConfig;
use std::fs;
use std::path::Path;

/// Parse configuration from a file
pub fn parse_config<P: AsRef<Path>>(path: P) -> ConfigResult<VxConfig> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(ConfigError::NotFound {
            path: path.display().to_string(),
        });
    }

    let content = fs::read_to_string(path)?;
    parse_config_str(&content)
}

/// Parse configuration from a string
pub fn parse_config_str(content: &str) -> ConfigResult<VxConfig> {
    let config: VxConfig = toml::from_str(content)?;
    Ok(config)
}

/// Find .vx.toml in current directory or parent directories
pub fn find_config<P: AsRef<Path>>(start_dir: P) -> ConfigResult<std::path::PathBuf> {
    let mut current = start_dir.as_ref().to_path_buf();

    loop {
        let config_path = current.join(".vx.toml");
        if config_path.exists() {
            return Ok(config_path);
        }

        if !current.pop() {
            break;
        }
    }

    Err(ConfigError::NotFound {
        path: ".vx.toml".to_string(),
    })
}

/// Load configuration from current directory or parent directories
pub fn load_config<P: AsRef<Path>>(start_dir: P) -> ConfigResult<VxConfig> {
    let config_path = find_config(start_dir)?;
    parse_config(&config_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_config() {
        let content = r#"
[tools]
node = "20"
"#;
        let config = parse_config_str(content).unwrap();
        assert_eq!(config.get_tool_version("node"), Some("20".to_string()));
    }

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
        assert_eq!(config.get_tool_version("node"), Some("20".to_string()));
        assert_eq!(config.get_tool_version("uv"), Some("latest".to_string()));
        assert_eq!(
            config.get_script_command("dev"),
            Some("npm run dev".to_string())
        );
    }

    #[test]
    fn test_parse_detailed_tool_config() {
        let content = r#"
[tools.node]
version = "20"
postinstall = "corepack enable"
os = ["linux", "darwin"]
"#;
        let config = parse_config_str(content).unwrap();
        assert_eq!(config.get_tool_version("node"), Some("20".to_string()));
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
    }

    #[test]
    fn test_parse_hooks_config() {
        let content = r#"
[hooks]
pre_setup = "echo 'Starting...'"
post_setup = ["vx run migrate", "vx run seed"]
"#;
        let config = parse_config_str(content).unwrap();
        assert!(config.hooks.is_some());
    }

    #[test]
    fn test_parse_services_config() {
        let content = r#"
[services.database]
image = "postgres:16"
ports = ["5432:5432"]

[services.redis]
image = "redis:7"
"#;
        let config = parse_config_str(content).unwrap();
        assert_eq!(config.services.len(), 2);
        assert!(config.services.contains_key("database"));
        assert!(config.services.contains_key("redis"));
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that v1 config format still works
        let content = r#"
[tools]
python = "3.11"
uv = "latest"

[settings]
auto_install = true
cache_duration = "7d"

[scripts]
fmt-check = "cargo fmt --all -- --check"
check = "cargo check --workspace --all-targets"
"#;
        let config = parse_config_str(content).unwrap();

        // Test backward-compatible accessors
        let tools = config.tools_as_hashmap();
        assert_eq!(tools.get("python"), Some(&"3.11".to_string()));

        let scripts = config.scripts_as_hashmap();
        assert_eq!(
            scripts.get("fmt-check"),
            Some(&"cargo fmt --all -- --check".to_string())
        );

        let settings = config.settings_as_hashmap();
        assert_eq!(settings.get("auto_install"), Some(&"true".to_string()));
    }
}
