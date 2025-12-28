//! Tests for the sync manager

use rstest::rstest;
use tempfile::TempDir;
use vx_project_analyzer::{AnalyzerConfig, ProjectAnalyzer, SyncAction, VxConfigSnapshot};

#[rstest]
#[tokio::test]
async fn test_sync_manager_generate_tool_actions() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create a Python project
    let pyproject = r#"
[project]
name = "test"
version = "0.1.0"
dependencies = ["requests"]

[tool.uv.scripts]
test = "pytest"
"#;
    tokio::fs::write(root.join("pyproject.toml"), pyproject)
        .await
        .unwrap();

    let config = AnalyzerConfig {
        check_installed: false,
        check_tools: false,
        generate_sync_actions: true,
        max_depth: 1,
    };

    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(root).await.unwrap();

    // Should have sync actions for tools
    let tool_actions: Vec<_> = analysis
        .sync_actions
        .iter()
        .filter(|a| matches!(a, SyncAction::AddTool { .. }))
        .collect();

    assert!(!tool_actions.is_empty());
}

#[rstest]
#[tokio::test]
async fn test_sync_manager_generate_script_actions() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create a Python project with scripts
    let pyproject = r#"
[project]
name = "test"
version = "0.1.0"

[tool.uv.scripts]
test = "pytest tests/"
lint = "ruff check ."
"#;
    tokio::fs::write(root.join("pyproject.toml"), pyproject)
        .await
        .unwrap();

    let config = AnalyzerConfig {
        check_installed: false,
        check_tools: false,
        generate_sync_actions: true,
        max_depth: 1,
    };

    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(root).await.unwrap();

    // Should have sync actions for scripts
    let script_actions: Vec<_> = analysis
        .sync_actions
        .iter()
        .filter(|a| matches!(a, SyncAction::AddScript { .. }))
        .collect();

    assert!(!script_actions.is_empty());
}

#[rstest]
#[tokio::test]
async fn test_vx_config_snapshot_load() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create a .vx.toml file
    let vx_toml = r#"
[tools]
node = "20.0.0"
uv = "0.4.0"

[scripts]
test = "npm test"
build = "npm run build"
"#;
    tokio::fs::write(root.join(".vx.toml"), vx_toml)
        .await
        .unwrap();

    let snapshot = VxConfigSnapshot::load(&root.join(".vx.toml"))
        .await
        .unwrap()
        .unwrap();

    assert_eq!(snapshot.tools.get("node"), Some(&"20.0.0".to_string()));
    assert_eq!(snapshot.tools.get("uv"), Some(&"0.4.0".to_string()));
    assert_eq!(snapshot.scripts.get("test"), Some(&"npm test".to_string()));
}

#[rstest]
#[tokio::test]
async fn test_sync_manager_dry_run() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create a Python project
    let pyproject = r#"
[project]
name = "test"
version = "0.1.0"

[tool.uv.scripts]
test = "pytest"
"#;
    tokio::fs::write(root.join("pyproject.toml"), pyproject)
        .await
        .unwrap();

    let config = AnalyzerConfig {
        check_installed: false,
        check_tools: false,
        generate_sync_actions: true,
        max_depth: 1,
    };

    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(root).await.unwrap();

    let sync_manager = analyzer.sync_manager();
    let result = sync_manager
        .apply_actions(root, &analysis.sync_actions, true)
        .await
        .unwrap();

    // Dry run should not create .vx.toml
    assert!(!root.join(".vx.toml").exists());
    assert!(!result.would_apply.is_empty());
}

#[rstest]
#[tokio::test]
async fn test_sync_manager_apply_actions() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create a Python project
    let pyproject = r#"
[project]
name = "test"
version = "0.1.0"

[tool.uv.scripts]
test = "pytest"
"#;
    tokio::fs::write(root.join("pyproject.toml"), pyproject)
        .await
        .unwrap();

    let config = AnalyzerConfig {
        check_installed: false,
        check_tools: false,
        generate_sync_actions: true,
        max_depth: 1,
    };

    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(root).await.unwrap();

    let sync_manager = analyzer.sync_manager();
    let result = sync_manager
        .apply_actions(root, &analysis.sync_actions, false)
        .await
        .unwrap();

    // Should create .vx.toml
    assert!(root.join(".vx.toml").exists());
    assert!(!result.applied.is_empty());
}
