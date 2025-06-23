use std::fs;
use tempfile::TempDir;
use vx_installer::{
    InstallConfigBuilder, Installer, LifecycleAction, LifecycleHooks, PostInstallAction,
};

#[test]
fn test_lifecycle_hooks_creation() {
    let hooks = LifecycleHooks::default();
    assert!(hooks.pre_install.is_empty());
    assert!(hooks.post_install.is_empty());
    assert!(hooks.pre_uninstall.is_empty());
    assert!(hooks.post_uninstall.is_empty());
    assert!(hooks.pre_update.is_empty());
    assert!(hooks.post_update.is_empty());
    assert!(hooks.pre_switch.is_empty());
    assert!(hooks.post_switch.is_empty());
}

#[test]
fn test_lifecycle_action_creation() {
    let action = LifecycleAction::FlattenDirectory {
        source_pattern: "test".to_string(),
    };

    match action {
        LifecycleAction::FlattenDirectory { source_pattern } => {
            assert_eq!(source_pattern, "test");
        }
        _ => panic!("Wrong action type"),
    }
}

#[test]
fn test_lifecycle_action_variants() {
    let actions = vec![
        LifecycleAction::CreateDirectory {
            path: "test-dir".to_string(),
        },
        LifecycleAction::RemoveFiles {
            pattern: "*.tmp".to_string(),
        },
        LifecycleAction::SetExecutable {
            path: "bin/tool".to_string(),
        },
        LifecycleAction::CreateConfig {
            path: "config.toml".to_string(),
            content: "key = \"value\"".to_string(),
        },
        LifecycleAction::HealthCheck {
            command: "tool --version".to_string(),
            expected_exit_code: Some(0),
        },
        LifecycleAction::CleanupTemp {
            pattern: ".tmp".to_string(),
        },
    ];

    assert_eq!(actions.len(), 6);
}

#[test]
fn test_install_config_builder() {
    let temp_dir = TempDir::new().unwrap();
    let install_dir = temp_dir.path().to_path_buf();

    let config = InstallConfigBuilder::new()
        .tool_name("test-tool")
        .version("1.0.0")
        .install_dir(install_dir.clone())
        .force(true)
        .build();

    assert_eq!(config.tool_name, "test-tool");
    assert_eq!(config.version, "1.0.0");
    assert_eq!(config.install_dir, install_dir);
    assert!(config.force);
}

#[test]
fn test_lifecycle_hooks_builder() {
    let action = LifecycleAction::CreateDirectory {
        path: "test-dir".to_string(),
    };

    let config = InstallConfigBuilder::new()
        .tool_name("test")
        .version("1.0.0")
        .install_dir(std::path::PathBuf::from("/tmp"))
        .post_install_action(action)
        .build();

    assert_eq!(config.lifecycle_hooks.post_install.len(), 1);
}

#[test]
fn test_multiple_lifecycle_actions() {
    let actions = vec![
        LifecycleAction::CreateDirectory {
            path: "bin".to_string(),
        },
        LifecycleAction::SetExecutable {
            path: "bin/tool".to_string(),
        },
    ];

    let mut hooks = LifecycleHooks::default();
    hooks.post_install.extend(actions);

    let config = InstallConfigBuilder::new()
        .tool_name("test")
        .version("1.0.0")
        .install_dir(std::path::PathBuf::from("/tmp"))
        .lifecycle_hooks(hooks)
        .build();

    assert_eq!(config.lifecycle_hooks.post_install.len(), 2);
}

#[test]
fn test_post_install_action_backward_compatibility() {
    // Test that the deprecated PostInstallAction type alias still works
    let _action: PostInstallAction = LifecycleAction::CreateDirectory {
        path: "test".to_string(),
    };
}

#[tokio::test]
async fn test_create_directory_action() {
    let temp_dir = TempDir::new().unwrap();
    let installer = Installer::new().await.unwrap();

    let test_dir = "test-subdir";
    installer
        .create_directory(temp_dir.path(), test_dir)
        .unwrap();

    let created_path = temp_dir.path().join(test_dir);
    assert!(created_path.exists());
    assert!(created_path.is_dir());
}

#[tokio::test]
async fn test_cleanup_temp_action() {
    let temp_dir = TempDir::new().unwrap();
    let installer = Installer::new().await.unwrap();

    // Create temporary files
    let temp_file1 = temp_dir.path().join("test1.tmp");
    let temp_file2 = temp_dir.path().join("test2.tmp");
    let normal_file = temp_dir.path().join("normal.txt");

    fs::write(&temp_file1, "test content 1").unwrap();
    fs::write(&temp_file2, "test content 2").unwrap();
    fs::write(&normal_file, "normal content").unwrap();

    assert!(temp_file1.exists());
    assert!(temp_file2.exists());
    assert!(normal_file.exists());

    // Clean up .tmp files
    installer.cleanup_temp(temp_dir.path(), ".tmp").unwrap();

    // Temp files should be removed, normal file should remain
    assert!(!temp_file1.exists());
    assert!(!temp_file2.exists());
    assert!(normal_file.exists());
}

#[tokio::test]
async fn test_create_config_action() {
    let temp_dir = TempDir::new().unwrap();
    let installer = Installer::new().await.unwrap();

    let config_content = r#"
[tool]
name = "test-tool"
version = "1.0.0"
"#;

    installer
        .create_config(temp_dir.path(), "config.toml", config_content)
        .unwrap();

    let config_path = temp_dir.path().join("config.toml");
    assert!(config_path.exists());

    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("test-tool"));
    assert!(content.contains("1.0.0"));
}

#[test]
fn test_lifecycle_hooks_serialization() {
    let mut hooks = LifecycleHooks::default();
    hooks.post_install.push(LifecycleAction::CreateDirectory {
        path: "bin".to_string(),
    });

    let serialized = serde_json::to_string(&hooks).unwrap();
    let deserialized: LifecycleHooks = serde_json::from_str(&serialized).unwrap();

    assert_eq!(hooks.post_install.len(), deserialized.post_install.len());
}
