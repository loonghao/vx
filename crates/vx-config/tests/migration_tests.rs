//! Configuration migration tests
//!
//! Tests for migrating `.vx.toml` from v1 to v2 format.

use tempfile::TempDir;
use vx_config::{ConfigMigrator, ConfigVersion, MigrationOptions};

// ============================================
// Version Detection Tests
// ============================================

#[test]
fn test_detect_v1_simple_config() {
    let content = r#"
[tools]
node = "20"
uv = "latest"

[scripts]
dev = "npm run dev"
"#;
    let migrator = ConfigMigrator::new();
    let version = migrator.detect_version(content);
    assert_eq!(version, ConfigVersion::V1);
}

#[test]
fn test_detect_v2_with_min_version() {
    let content = r#"
min_version = "0.6.0"

[tools]
node = "20"
"#;
    let migrator = ConfigMigrator::new();
    let version = migrator.detect_version(content);
    assert_eq!(version, ConfigVersion::V2);
}

#[test]
fn test_detect_v2_with_project() {
    let content = r#"
[project]
name = "my-app"

[tools]
node = "20"
"#;
    let migrator = ConfigMigrator::new();
    let version = migrator.detect_version(content);
    assert_eq!(version, ConfigVersion::V2);
}

#[test]
fn test_detect_v2_with_hooks() {
    let content = r#"
[tools]
node = "20"

[hooks]
pre_setup = "echo 'Starting...'"
"#;
    let migrator = ConfigMigrator::new();
    let version = migrator.detect_version(content);
    assert_eq!(version, ConfigVersion::V2);
}

#[test]
fn test_detect_v2_with_services() {
    let content = r#"
[tools]
node = "20"

[services.database]
image = "postgres:16"
"#;
    let migrator = ConfigMigrator::new();
    let version = migrator.detect_version(content);
    assert_eq!(version, ConfigVersion::V2);
}

#[test]
fn test_detect_v2_with_detailed_tools() {
    let content = r#"
[tools.node]
version = "20"
postinstall = "corepack enable"
"#;
    let migrator = ConfigMigrator::new();
    let version = migrator.detect_version(content);
    assert_eq!(version, ConfigVersion::V2);
}

#[test]
fn test_detect_v2_with_detailed_scripts() {
    let content = r#"
[scripts.start]
command = "npm start"
description = "Start the server"
"#;
    let migrator = ConfigMigrator::new();
    let version = migrator.detect_version(content);
    assert_eq!(version, ConfigVersion::V2);
}

#[test]
fn test_detect_unknown_invalid_toml() {
    let content = "this is not valid toml [[[";
    let migrator = ConfigMigrator::new();
    let version = migrator.detect_version(content);
    assert_eq!(version, ConfigVersion::Unknown);
}

#[test]
fn test_detect_unknown_empty_config() {
    let content = "";
    let migrator = ConfigMigrator::new();
    let version = migrator.detect_version(content);
    assert_eq!(version, ConfigVersion::Unknown);
}

// ============================================
// Content Migration Tests
// ============================================

#[test]
fn test_migrate_simple_tools() {
    let content = r#"
[tools]
node = "20"
uv = "0.5"
"#;
    let migrator = ConfigMigrator::new();
    let options = MigrationOptions {
        add_comments: false,
        ..Default::default()
    };

    let result = migrator.migrate_content(content, &options).unwrap();

    assert!(result.migrated);
    assert_eq!(result.from_version, ConfigVersion::V1);
    assert_eq!(result.to_version, ConfigVersion::V2);
    assert!(result.content.contains("node"));
    assert!(result.content.contains("uv"));
}

#[test]
fn test_migrate_preserves_tool_versions() {
    let content = r#"
[tools]
node = "20.0.0"
go = "1.22"
python = "3.11"
"#;
    let migrator = ConfigMigrator::new();
    let options = MigrationOptions {
        add_comments: false,
        ..Default::default()
    };

    let result = migrator.migrate_content(content, &options).unwrap();

    // Parse the migrated content to verify
    let config = vx_config::parse_config_str(&result.content).unwrap();
    assert_eq!(config.get_tool_version("node"), Some("20.0.0".to_string()));
    assert_eq!(config.get_tool_version("go"), Some("1.22".to_string()));
    assert_eq!(config.get_tool_version("python"), Some("3.11".to_string()));
}

#[test]
fn test_migrate_simple_scripts() {
    let content = r#"
[scripts]
dev = "npm run dev"
test = "npm test"
build = "npm run build"
"#;
    let migrator = ConfigMigrator::new();
    let options = MigrationOptions {
        add_comments: false,
        ..Default::default()
    };

    let result = migrator.migrate_content(content, &options).unwrap();

    // Parse and verify
    let config = vx_config::parse_config_str(&result.content).unwrap();
    assert_eq!(
        config.get_script_command("dev"),
        Some("npm run dev".to_string())
    );
    assert_eq!(
        config.get_script_command("test"),
        Some("npm test".to_string())
    );
}

#[test]
fn test_migrate_settings() {
    let content = r#"
[settings]
auto_install = true
cache_duration = "7d"
"#;
    let migrator = ConfigMigrator::new();
    let options = MigrationOptions {
        add_comments: false,
        ..Default::default()
    };

    let result = migrator.migrate_content(content, &options).unwrap();

    // Parse and verify
    let config = vx_config::parse_config_str(&result.content).unwrap();
    let settings = config.settings.unwrap();
    assert_eq!(settings.auto_install, Some(true));
    assert_eq!(settings.cache_duration, Some("7d".to_string()));
}

#[test]
fn test_migrate_env_variables() {
    let content = r#"
[env]
NODE_ENV = "development"
DEBUG = "true"
API_URL = "http://localhost:3000"
"#;
    let migrator = ConfigMigrator::new();
    let options = MigrationOptions {
        add_comments: false,
        ..Default::default()
    };

    let result = migrator.migrate_content(content, &options).unwrap();

    // Parse and verify
    let config = vx_config::parse_config_str(&result.content).unwrap();
    let env = config.env.unwrap();
    assert_eq!(env.vars.get("NODE_ENV"), Some(&"development".to_string()));
    assert_eq!(env.vars.get("DEBUG"), Some(&"true".to_string()));
}

#[test]
fn test_migrate_full_v1_config() {
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
    let migrator = ConfigMigrator::new();
    let options = MigrationOptions {
        add_comments: false,
        ..Default::default()
    };

    let result = migrator.migrate_content(content, &options).unwrap();

    assert!(result.migrated);
    assert!(!result.changes.is_empty());

    // Verify the migrated config is valid
    let config = vx_config::parse_config_str(&result.content).unwrap();
    assert_eq!(config.get_tool_version("node"), Some("20".to_string()));
    assert_eq!(
        config.get_script_command("dev"),
        Some("npm run dev".to_string())
    );
}

// ============================================
// Migration Options Tests
// ============================================

#[test]
fn test_migrate_with_comments() {
    let content = r#"
[tools]
node = "20"
"#;
    let migrator = ConfigMigrator::new();
    let options = MigrationOptions {
        add_comments: true,
        ..Default::default()
    };

    let result = migrator.migrate_content(content, &options).unwrap();

    // Should contain comment headers
    assert!(result.content.contains("# VX Project Configuration"));
}

#[test]
fn test_migrate_without_comments() {
    let content = r#"
[tools]
node = "20"
"#;
    let migrator = ConfigMigrator::new();
    let options = MigrationOptions {
        add_comments: false,
        ..Default::default()
    };

    let result = migrator.migrate_content(content, &options).unwrap();

    // Should not contain comment headers
    assert!(!result.content.contains("# VX Project Configuration"));
}

// ============================================
// File Migration Tests
// ============================================

#[test]
fn test_migrate_file_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".vx.toml");

    // Write v1 config
    std::fs::write(
        &config_path,
        r#"
[tools]
node = "20"

[scripts]
dev = "npm run dev"
"#,
    )
    .unwrap();

    let migrator = ConfigMigrator::new();
    let options = MigrationOptions {
        dry_run: true,
        backup: false,
        add_comments: false,
        ..Default::default()
    };

    let result = migrator.migrate_file(&config_path, &options).unwrap();

    assert!(result.migrated);

    // File should not be modified in dry run
    let file_content = std::fs::read_to_string(&config_path).unwrap();
    assert!(file_content.contains("[tools]"));
    assert!(file_content.contains("node = \"20\""));
}

#[test]
fn test_migrate_file_with_backup() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".vx.toml");

    // Write v1 config
    std::fs::write(
        &config_path,
        r#"
[tools]
node = "20"
"#,
    )
    .unwrap();

    let migrator = ConfigMigrator::new();
    let options = MigrationOptions {
        backup: true,
        dry_run: false,
        add_comments: false,
        ..Default::default()
    };

    let result = migrator.migrate_file(&config_path, &options).unwrap();

    assert!(result.migrated);
    assert!(result.backup_path.is_some());

    // Backup file should exist
    let backup_path = result.backup_path.unwrap();
    assert!(std::path::Path::new(&backup_path).exists());
}

#[test]
fn test_migrate_file_without_backup() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".vx.toml");

    // Write v1 config
    std::fs::write(
        &config_path,
        r#"
[tools]
node = "20"
"#,
    )
    .unwrap();

    let migrator = ConfigMigrator::new();
    let options = MigrationOptions {
        backup: false,
        dry_run: false,
        add_comments: false,
        ..Default::default()
    };

    let result = migrator.migrate_file(&config_path, &options).unwrap();

    assert!(result.migrated);
    assert!(result.backup_path.is_none());
}

#[test]
fn test_migrate_already_v2_skips() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".vx.toml");

    // Write v2 config
    std::fs::write(
        &config_path,
        r#"
min_version = "0.6.0"

[tools]
node = "20"
"#,
    )
    .unwrap();

    let migrator = ConfigMigrator::new();
    let options = MigrationOptions {
        force: false,
        ..Default::default()
    };

    let result = migrator.migrate_file(&config_path, &options).unwrap();

    assert!(!result.migrated);
    assert!(result.warnings.iter().any(|w: &String| w.contains("already v2")));
}

#[test]
fn test_migrate_already_v2_with_force() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".vx.toml");

    // Write v2 config
    std::fs::write(
        &config_path,
        r#"
min_version = "0.6.0"

[tools]
node = "20"
"#,
    )
    .unwrap();

    let migrator = ConfigMigrator::new();
    let options = MigrationOptions {
        force: true,
        backup: false,
        dry_run: false,
        add_comments: false,
    };

    let result = migrator.migrate_file(&config_path, &options).unwrap();

    assert!(result.migrated);
}

// ============================================
// Migration Changes Tracking Tests
// ============================================

#[test]
fn test_migration_tracks_changes() {
    let content = r#"
[tools]
node = "20"
go = "1.22"

[scripts]
dev = "npm run dev"

[settings]
auto_install = true
"#;
    let migrator = ConfigMigrator::new();
    let options = MigrationOptions {
        add_comments: false,
        ..Default::default()
    };

    let result = migrator.migrate_content(content, &options).unwrap();

    // Should have tracked changes
    assert!(!result.changes.is_empty());

    // Check for specific changes
    let changes_str = result.changes.join("\n");
    assert!(changes_str.contains("tools.node"));
    assert!(changes_str.contains("tools.go"));
    assert!(changes_str.contains("scripts.dev"));
    assert!(changes_str.contains("settings"));
}

// ============================================
// Edge Cases
// ============================================

#[test]
fn test_migrate_empty_sections() {
    let content = r#"
[tools]

[scripts]

[settings]
"#;
    let migrator = ConfigMigrator::new();
    let options = MigrationOptions {
        add_comments: false,
        ..Default::default()
    };

    // Should not error on empty sections
    let result = migrator.migrate_content(content, &options);
    assert!(result.is_ok());
}

#[test]
fn test_migrate_preserves_v2_fields() {
    let content = r#"
[tools]
node = "20"

[hooks]
pre_setup = "echo 'Starting...'"

[services.database]
image = "postgres:16"
"#;
    let migrator = ConfigMigrator::new();
    let options = MigrationOptions {
        add_comments: false,
        force: true,
        ..Default::default()
    };

    let result = migrator.migrate_content(content, &options).unwrap();

    // Parse and verify v2 fields are preserved
    let config = vx_config::parse_config_str(&result.content).unwrap();
    assert!(config.hooks.is_some());
    assert!(!config.services.is_empty());
}

#[test]
fn test_migrate_detailed_tool_config() {
    let content = r#"
[tools.node]
version = "20"
postinstall = "corepack enable"
os = ["linux", "darwin"]
"#;
    let migrator = ConfigMigrator::new();
    let options = MigrationOptions {
        add_comments: false,
        force: true,
        ..Default::default()
    };

    let result = migrator.migrate_content(content, &options).unwrap();

    // Parse and verify
    let config = vx_config::parse_config_str(&result.content).unwrap();
    assert_eq!(config.get_tool_version("node"), Some("20".to_string()));
}

#[test]
fn test_migrate_detailed_script_config() {
    let content = r#"
[scripts.start]
command = "npm start"
description = "Start the server"
cwd = "src"
"#;
    let migrator = ConfigMigrator::new();
    let options = MigrationOptions {
        add_comments: false,
        force: true,
        ..Default::default()
    };

    let result = migrator.migrate_content(content, &options).unwrap();

    // Parse and verify
    let config = vx_config::parse_config_str(&result.content).unwrap();
    assert_eq!(
        config.get_script_command("start"),
        Some("npm start".to_string())
    );
}

// ============================================
// ConfigVersion Display Tests
// ============================================

#[test]
fn test_config_version_display() {
    assert_eq!(format!("{}", ConfigVersion::V1), "v1");
    assert_eq!(format!("{}", ConfigVersion::V2), "v2");
    assert_eq!(format!("{}", ConfigVersion::Unknown), "unknown");
}

// ============================================
// MigrationOptions Default Tests
// ============================================

#[test]
fn test_migration_options_default() {
    let options = MigrationOptions::default();
    assert!(options.backup);
    assert!(!options.force);
    assert!(!options.dry_run);
    assert!(options.add_comments);
}
