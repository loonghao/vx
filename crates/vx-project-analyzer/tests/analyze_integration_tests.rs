//! Integration tests for analyze command with new options

use rstest::rstest;
use std::fs;
use tempfile::TempDir;
use vx_project_analyzer::{AnalyzerConfig, ProjectAnalyzer};

#[rstest]
#[tokio::test]
async fn test_analyze_with_missing_tools() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create a Python project
    let pyproject = r#"
[project]
name = "test"
version = "0.1.0"
dependencies = ["requests"]
"#;
    fs::write(root.join("pyproject.toml"), pyproject).unwrap();

    // Use check_tools=false to avoid environment-dependent tool availability checks.
    // The test validates that the analyzer detects required tools for a Python project,
    // regardless of whether those tools are installed in the test environment.
    let config = AnalyzerConfig {
        check_installed: false,
        check_tools: false,
        generate_sync_actions: false,
        max_depth: 1,
    };

    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(root).await.unwrap();

    // Should detect required tools for Python project (uv or python)
    assert!(
        !analysis.required_tools.is_empty(),
        "Should detect required tools for Python project, got: {:?}",
        analysis.required_tools.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    // With check_tools=false, is_available defaults to false, so all tools appear missing
    let missing_tools = analysis.missing_tools();
    assert!(
        !missing_tools.is_empty(),
        "With check_tools=false, required tools should appear as missing"
    );
}

#[rstest]
#[tokio::test]
async fn test_analyze_max_depth() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create a Python project
    let pyproject = r#"
[project]
name = "test"
version = "0.1.0"
"#;
    fs::write(root.join("pyproject.toml"), pyproject).unwrap();

    // Test with max_depth = 1 (only root)
    let config = AnalyzerConfig {
        check_installed: false,
        check_tools: false,
        generate_sync_actions: false,
        max_depth: 1,
    };

    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(root).await.unwrap();

    // Should only analyze root
    assert!(
        analysis.dependencies.is_empty(),
        "Should not analyze subdirectories with max_depth=1"
    );
}

#[rstest]
#[tokio::test]
async fn test_analyze_monorepo() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create root package.json
    let root_package = r#"{
  "name": "monorepo",
  "private": true
}"#;
    fs::write(root.join("package.json"), root_package).unwrap();

    // Create subdirectory with Cargo.toml
    let subdir = root.join("packages/my-rust-app");
    fs::create_dir_all(&subdir).unwrap();
    let cargo_toml = r#"
[package]
name = "my-rust-app"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(subdir.join("Cargo.toml"), cargo_toml).unwrap();

    // Test with max_depth = 2 (should include subdirectory)
    let config = AnalyzerConfig {
        check_installed: false,
        check_tools: false,
        generate_sync_actions: false,
        max_depth: 2,
    };

    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(root).await.unwrap();

    // Should detect both ecosystems
    assert!(
        analysis.ecosystems.len() >= 2,
        "Should detect multiple ecosystems in monorepo"
    );
}
