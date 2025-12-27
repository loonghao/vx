//! Configuration validation tests
//!
//! Tests for validating `.vx.toml` configuration files.

use rstest::rstest;
use vx_config::{parse_config_str, validate_config};

// ============================================
// Basic Validation Tests
// ============================================

#[test]
fn test_validate_empty_config() {
    let content = "";
    let config = parse_config_str(content).unwrap();
    let result = validate_config(&config);
    assert!(result.is_ok());
}

#[test]
fn test_validate_minimal_config() {
    let content = r#"
[tools]
node = "20"
"#;
    let config = parse_config_str(content).unwrap();
    let result = validate_config(&config);
    assert!(result.is_ok());
    assert!(result.warnings.is_empty());
}

// ============================================
// Version Validation Tests
// ============================================

#[test]
fn test_validate_valid_min_version() {
    let content = r#"
min_version = "0.6.0"

[tools]
node = "20"
"#;
    let config = parse_config_str(content).unwrap();
    let result = validate_config(&config);
    assert!(result.is_ok());
}

#[rstest]
#[case("0.6")]
#[case("1.0")]
#[case("0.6.0")]
#[case("1.0.0")]
fn test_validate_valid_version_formats(#[case] version: &str) {
    let content = format!(
        r#"
min_version = "{}"
"#,
        version
    );
    let config = parse_config_str(&content).unwrap();
    let result = validate_config(&config);
    assert!(result.is_ok(), "Version '{}' should be valid", version);
}

#[rstest]
#[case("invalid")]
#[case("abc.def")]
#[case("1.2.3.4")]
#[case("")]
fn test_validate_invalid_version_formats(#[case] version: &str) {
    let content = format!(
        r#"
min_version = "{}"
"#,
        version
    );
    let config = parse_config_str(&content).unwrap();
    let result = validate_config(&config);
    assert!(!result.is_ok(), "Version '{}' should be invalid", version);
}

// ============================================
// Tool Name Validation Tests
// ============================================

#[rstest]
#[case("node")]
#[case("go")]
#[case("my-tool")]
#[case("my_tool")]
#[case("tool123")]
fn test_validate_valid_tool_names(#[case] name: &str) {
    let content = format!(
        r#"
[tools]
{} = "latest"
"#,
        name
    );
    let config = parse_config_str(&content).unwrap();
    let result = validate_config(&config);
    assert!(
        result.warnings.is_empty(),
        "Tool name '{}' should not produce warnings",
        name
    );
}

#[test]
fn test_validate_unusual_tool_name_warning() {
    let content = r#"
[tools]
"my.tool" = "latest"
"#;
    let config = parse_config_str(content).unwrap();
    let result = validate_config(&config);
    // Should produce a warning for unusual characters
    assert!(!result.warnings.is_empty());
}

// ============================================
// Script Name Validation Tests
// ============================================

#[rstest]
#[case("dev")]
#[case("test")]
#[case("build-prod")]
#[case("pre_commit")]
#[case("test:unit")]
fn test_validate_valid_script_names(#[case] name: &str) {
    let content = format!(
        r#"
[scripts]
{} = "echo test"
"#,
        name
    );
    let config = parse_config_str(&content).unwrap();
    let result = validate_config(&config);
    assert!(
        result.warnings.is_empty(),
        "Script name '{}' should not produce warnings",
        name
    );
}

// ============================================
// Service Validation Tests
// ============================================

#[test]
fn test_validate_service_with_image() {
    let content = r#"
[services.database]
image = "postgres:16"
"#;
    let config = parse_config_str(content).unwrap();
    let result = validate_config(&config);
    assert!(result.warnings.is_empty());
}

#[test]
fn test_validate_service_with_command() {
    let content = r#"
[services.api]
command = "python app.py"
"#;
    let config = parse_config_str(content).unwrap();
    let result = validate_config(&config);
    assert!(result.warnings.is_empty());
}

#[test]
fn test_validate_service_without_image_or_command() {
    let content = r#"
[services.empty]
ports = ["8080:80"]
"#;
    let config = parse_config_str(content).unwrap();
    let result = validate_config(&config);
    // Should produce a warning
    assert!(!result.warnings.is_empty());
    assert!(result.warnings[0].contains("neither"));
}

// ============================================
// Port Mapping Validation Tests
// ============================================

#[rstest]
#[case("8080")]
#[case("8080:80")]
#[case("3000:3000")]
fn test_validate_valid_port_mappings(#[case] port: &str) {
    let content = format!(
        r#"
[services.app]
image = "nginx"
ports = ["{}"]
"#,
        port
    );
    let config = parse_config_str(&content).unwrap();
    let result = validate_config(&config);
    assert!(
        result.warnings.is_empty(),
        "Port mapping '{}' should be valid",
        port
    );
}

#[rstest]
#[case("invalid")]
#[case("abc:def")]
#[case("8080:80:90")]
#[case("-1")]
fn test_validate_invalid_port_mappings(#[case] port: &str) {
    let content = format!(
        r#"
[services.app]
image = "nginx"
ports = ["{}"]
"#,
        port
    );
    let config = parse_config_str(&content).unwrap();
    let result = validate_config(&config);
    assert!(
        !result.warnings.is_empty(),
        "Port mapping '{}' should produce a warning",
        port
    );
}

// ============================================
// Full Configuration Validation Tests
// ============================================

#[test]
fn test_validate_complete_valid_config() {
    let content = r#"
min_version = "0.6.0"

[project]
name = "my-app"
description = "My application"

[tools]
node = "20"
uv = "0.5"

[env]
NODE_ENV = "development"

[scripts]
dev = "npm run dev"
test = "npm test"

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
    let result = validate_config(&config);
    assert!(result.is_ok());
    assert!(result.warnings.is_empty());
}

#[test]
fn test_validate_config_with_multiple_issues() {
    let content = r#"
min_version = "invalid"

[tools]
"my.weird.tool" = "latest"

[services.empty]
# No image or command

[services.app]
image = "nginx"
ports = ["invalid-port"]
"#;
    let config = parse_config_str(content).unwrap();
    let result = validate_config(&config);

    // Should have errors (invalid version)
    assert!(!result.is_ok());

    // Should have warnings (unusual tool name, empty service, invalid port)
    assert!(result.warnings.len() >= 2);
}

// ============================================
// Validation Result API Tests
// ============================================

#[test]
fn test_validation_result_is_ok() {
    let content = r#"
[tools]
node = "20"
"#;
    let config = parse_config_str(content).unwrap();
    let result = validate_config(&config);
    assert!(result.is_ok());
    assert!(result.errors.is_empty());
}

#[test]
fn test_validation_result_has_errors() {
    let content = r#"
min_version = "not-a-version"
"#;
    let config = parse_config_str(content).unwrap();
    let result = validate_config(&config);
    assert!(!result.is_ok());
    assert!(!result.errors.is_empty());
}

#[test]
fn test_validation_result_has_warnings() {
    let content = r#"
[services.empty]
ports = ["8080:80"]
"#;
    let config = parse_config_str(content).unwrap();
    let result = validate_config(&config);
    // Warnings don't affect is_ok()
    assert!(result.is_ok());
    assert!(!result.warnings.is_empty());
}

// ============================================
// Edge Cases
// ============================================

#[test]
fn test_validate_empty_tools_section() {
    let content = r#"
[tools]
"#;
    let config = parse_config_str(content).unwrap();
    let result = validate_config(&config);
    assert!(result.is_ok());
}

#[test]
fn test_validate_empty_services_section() {
    let content = r#"
[services]
"#;
    let config = parse_config_str(content).unwrap();
    let result = validate_config(&config);
    assert!(result.is_ok());
}

#[test]
fn test_validate_multiple_services() {
    let content = r#"
[services.db]
image = "postgres:16"
ports = ["5432:5432"]

[services.redis]
image = "redis:7"
ports = ["6379:6379"]

[services.api]
command = "python app.py"
ports = ["8000:8000"]
"#;
    let config = parse_config_str(content).unwrap();
    let result = validate_config(&config);
    assert!(result.is_ok());
    assert!(result.warnings.is_empty());
}
