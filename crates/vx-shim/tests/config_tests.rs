//! Shim configuration tests

use anyhow::Result;
use pretty_assertions::assert_eq;
use rstest::*;
use vx_shim::ShimConfig;

mod common;
use common::sample_configs;

/// Test TOML shim configuration parsing
#[rstest]
fn test_toml_config_parsing() -> Result<()> {
    let config = ShimConfig::parse(sample_configs::TOML_SHIM_CONFIG)?;

    assert_eq!(config.path, "/usr/bin/node");
    assert_eq!(config.resolved_args(), vec!["--version".to_string()]);

    // Check environment variables
    let env = config.env.unwrap();
    assert_eq!(env.get("NODE_ENV"), Some(&"development".to_string()));
    assert_eq!(env.get("PATH_EXTRA"), Some(&"/extra/path".to_string()));

    Ok(())
}

/// Test environment variable expansion
#[rstest]
fn test_env_var_expansion() -> Result<()> {
    // Set test environment variables
    std::env::set_var("NODE_HOME", "/opt/node");
    std::env::set_var("NODE_ARGS", "--inspect");
    std::env::set_var("DEBUG", "true");

    let config = ShimConfig::parse(sample_configs::SHIM_WITH_ENV_VARS)?;

    // Test resolved values (with environment variable expansion)
    assert_eq!(config.resolved_path(), "/opt/node/bin/node");
    assert_eq!(config.resolved_args(), vec!["--inspect".to_string()]);

    // Check environment variables with expansion
    let resolved_env = config.resolved_env();
    assert_eq!(resolved_env.get("DEBUG"), Some(&"true".to_string()));

    // Clean up
    std::env::remove_var("NODE_HOME");
    std::env::remove_var("NODE_ARGS");
    std::env::remove_var("DEBUG");

    Ok(())
}

/// Test invalid configuration handling
#[rstest]
fn test_invalid_config() {
    let invalid_config = "invalid toml content [[[";
    let result = ShimConfig::parse(invalid_config);
    assert!(result.is_err());
}

/// Test minimal configuration
#[rstest]
fn test_minimal_config() -> Result<()> {
    let minimal_config = r#"path = "/bin/echo""#;
    let config = ShimConfig::parse(minimal_config)?;

    assert_eq!(config.path, "/bin/echo");
    assert_eq!(config.args, None);
    assert_eq!(config.env, None);

    Ok(())
}
