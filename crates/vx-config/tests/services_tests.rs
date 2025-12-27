//! Service configuration tests
//!
//! Tests for service orchestration configuration parsing and validation.

use vx_config::{parse_config_str, validate_config};

// ============================================
// Basic Service Configuration Tests
// ============================================

#[test]
fn test_parse_service_with_image() {
    let content = r#"
[services.database]
image = "postgres:16"
"#;
    let config = parse_config_str(content).unwrap();
    assert_eq!(config.services.len(), 1);

    let db = config.services.get("database").unwrap();
    assert_eq!(db.image, Some("postgres:16".to_string()));
}

#[test]
fn test_parse_service_with_command() {
    let content = r#"
[services.api]
command = "python app.py"
"#;
    let config = parse_config_str(content).unwrap();

    let api = config.services.get("api").unwrap();
    assert_eq!(api.command, Some("python app.py".to_string()));
}

#[test]
fn test_parse_service_with_ports() {
    let content = r#"
[services.web]
image = "nginx"
ports = ["80:80", "443:443"]
"#;
    let config = parse_config_str(content).unwrap();

    let web = config.services.get("web").unwrap();
    assert_eq!(web.ports, vec!["80:80".to_string(), "443:443".to_string()]);
}

#[test]
fn test_parse_service_with_volumes() {
    let content = r#"
[services.database]
image = "postgres:16"
volumes = ["./data:/var/lib/postgresql/data", "./init:/docker-entrypoint-initdb.d"]
"#;
    let config = parse_config_str(content).unwrap();

    let db = config.services.get("database").unwrap();
    assert_eq!(db.volumes.len(), 2);
    assert!(db.volumes[0].contains("data"));
}

#[test]
fn test_parse_service_with_environment() {
    let content = r#"
[services.database]
image = "postgres:16"
env = { POSTGRES_PASSWORD = "secret", POSTGRES_USER = "admin" }
"#;
    let config = parse_config_str(content).unwrap();

    let db = config.services.get("database").unwrap();
    let env = &db.env;
    assert_eq!(env.get("POSTGRES_PASSWORD"), Some(&"secret".to_string()));
    assert_eq!(env.get("POSTGRES_USER"), Some(&"admin".to_string()));
}

// ============================================
// Service Dependencies Tests
// ============================================

#[test]
fn test_parse_service_with_depends_on() {
    let content = r#"
[services.database]
image = "postgres:16"

[services.api]
image = "my-api:latest"
depends_on = ["database"]
"#;
    let config = parse_config_str(content).unwrap();

    let api = config.services.get("api").unwrap();
    assert_eq!(api.depends_on, vec!["database".to_string()]);
}

#[test]
fn test_parse_service_with_multiple_dependencies() {
    let content = r#"
[services.database]
image = "postgres:16"

[services.redis]
image = "redis:7"

[services.api]
image = "my-api:latest"
depends_on = ["database", "redis"]
"#;
    let config = parse_config_str(content).unwrap();

    let api = config.services.get("api").unwrap();
    assert_eq!(api.depends_on.len(), 2);
    assert!(api.depends_on.contains(&"database".to_string()));
    assert!(api.depends_on.contains(&"redis".to_string()));
}

// ============================================
// Service Healthcheck Tests
// ============================================

#[test]
fn test_parse_service_with_healthcheck() {
    let content = r#"
[services.database]
image = "postgres:16"
healthcheck = "pg_isready -U postgres"
"#;
    let config = parse_config_str(content).unwrap();

    let db = config.services.get("database").unwrap();
    assert_eq!(db.healthcheck, Some("pg_isready -U postgres".to_string()));
}

// ============================================
// Multiple Services Tests
// ============================================

#[test]
fn test_parse_multiple_services() {
    let content = r#"
[services.database]
image = "postgres:16"
ports = ["5432:5432"]

[services.redis]
image = "redis:7"
ports = ["6379:6379"]

[services.api]
image = "my-api:latest"
ports = ["8000:8000"]
depends_on = ["database", "redis"]

[services.web]
image = "nginx"
ports = ["80:80"]
depends_on = ["api"]
"#;
    let config = parse_config_str(content).unwrap();

    assert_eq!(config.services.len(), 4);
    assert!(config.services.contains_key("database"));
    assert!(config.services.contains_key("redis"));
    assert!(config.services.contains_key("api"));
    assert!(config.services.contains_key("web"));
}

// ============================================
// Service Working Directory Tests
// ============================================

#[test]
fn test_parse_service_with_working_dir() {
    let content = r#"
[services.api]
image = "my-api:latest"
working_dir = "/app"
"#;
    let config = parse_config_str(content).unwrap();

    let api = config.services.get("api").unwrap();
    assert_eq!(api.working_dir, Some("/app".to_string()));
}

// ============================================
// Full Service Configuration Tests
// ============================================

#[test]
fn test_parse_full_service_config() {
    let content = r#"
[services.database]
image = "postgres:16"
ports = ["5432:5432"]
volumes = ["./data:/var/lib/postgresql/data"]
env = { POSTGRES_PASSWORD = "secret", POSTGRES_DB = "myapp" }
healthcheck = "pg_isready"
"#;
    let config = parse_config_str(content).unwrap();

    let db = config.services.get("database").unwrap();

    // Verify all fields
    assert_eq!(db.image, Some("postgres:16".to_string()));
    assert_eq!(db.ports, vec!["5432:5432".to_string()]);
    assert_eq!(db.volumes.len(), 1);
    assert_eq!(db.env.len(), 2);
    assert!(db.healthcheck.is_some());
}

// ============================================
// Empty Service Tests
// ============================================

#[test]
fn test_parse_empty_services() {
    let content = r#"
[services]
"#;
    let config = parse_config_str(content).unwrap();
    assert!(config.services.is_empty());
}

// ============================================
// Service Default Values Tests
// ============================================

#[test]
fn test_service_default_values() {
    let content = r#"
[services.minimal]
image = "alpine"
"#;
    let config = parse_config_str(content).unwrap();

    let svc = config.services.get("minimal").unwrap();

    // Check defaults
    assert!(svc.ports.is_empty());
    assert!(svc.volumes.is_empty());
    assert!(svc.env.is_empty());
    assert!(svc.depends_on.is_empty());
    assert!(svc.healthcheck.is_none());
    assert!(svc.command.is_none());
}

// ============================================
// Service Validation Integration Tests
// ============================================

#[test]
fn test_validate_service_without_image_or_command() {
    let content = r#"
[services.invalid]
ports = ["8080:80"]
"#;
    let config = parse_config_str(content).unwrap();
    let result = validate_config(&config);

    // Should produce a warning
    assert!(!result.warnings.is_empty());
}

#[test]
fn test_validate_service_with_invalid_port() {
    let content = r#"
[services.api]
image = "my-api"
ports = ["not-a-port"]
"#;
    let config = parse_config_str(content).unwrap();
    let result = validate_config(&config);

    // Should produce a warning about invalid port
    assert!(!result.warnings.is_empty());
}

#[test]
fn test_validate_service_with_valid_config() {
    let content = r#"
[services.api]
image = "my-api:latest"
ports = ["8000:8000"]
"#;
    let config = parse_config_str(content).unwrap();
    let result = validate_config(&config);

    assert!(result.is_ok());
    assert!(result.warnings.is_empty());
}
