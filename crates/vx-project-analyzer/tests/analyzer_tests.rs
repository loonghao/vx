//! Tests for the project analyzer

use rstest::rstest;
use std::path::PathBuf;
use tempfile::TempDir;
use vx_project_analyzer::{AnalyzerConfig, Ecosystem, ProjectAnalyzer};

/// Create a temp directory with Python project files
async fn create_python_project(dir: &TempDir) -> PathBuf {
    let root = dir.path().to_path_buf();

    // Create pyproject.toml
    let pyproject = r#"
[project]
name = "test-project"
version = "0.1.0"
dependencies = [
    "requests>=2.28",
    "click>=8.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=7.0",
    "ruff>=0.1.0",
]

[dependency-groups]
test = [
    "pytest-cov>=4.0",
]

[tool.uv.scripts]
test = "pytest tests/"
lint = "ruff check ."
"#;

    tokio::fs::write(root.join("pyproject.toml"), pyproject)
        .await
        .unwrap();

    root
}

/// Create a temp directory with Node.js project files
async fn create_nodejs_project(dir: &TempDir) -> PathBuf {
    let root = dir.path().to_path_buf();

    // Create package.json
    let package_json = r#"
{
    "name": "test-project",
    "version": "1.0.0",
    "dependencies": {
        "express": "^4.18.0",
        "lodash": "^4.17.0"
    },
    "devDependencies": {
        "jest": "^29.0.0",
        "typescript": "^5.0.0"
    },
    "scripts": {
        "test": "jest",
        "build": "tsc",
        "start": "node dist/index.js"
    }
}
"#;

    tokio::fs::write(root.join("package.json"), package_json)
        .await
        .unwrap();

    root
}

/// Create a temp directory with Rust project files
async fn create_rust_project(dir: &TempDir) -> PathBuf {
    let root = dir.path().to_path_buf();

    // Create Cargo.toml
    let cargo_toml = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }

[dev-dependencies]
rstest = "0.18"
"#;

    tokio::fs::write(root.join("Cargo.toml"), cargo_toml)
        .await
        .unwrap();

    root
}

#[rstest]
#[tokio::test]
async fn test_python_project_detection() {
    let dir = TempDir::new().unwrap();
    let root = create_python_project(&dir).await;

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    assert!(analysis.ecosystems.contains(&Ecosystem::Python));
}

#[rstest]
#[tokio::test]
async fn test_python_dependencies() {
    let dir = TempDir::new().unwrap();
    let root = create_python_project(&dir).await;

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    // Check dependencies were found
    let dep_names: Vec<_> = analysis.dependencies.iter().map(|d| &d.name).collect();
    assert!(dep_names.contains(&&"requests".to_string()));
    assert!(dep_names.contains(&&"click".to_string()));
    assert!(dep_names.contains(&&"pytest".to_string()));
}

#[rstest]
#[tokio::test]
async fn test_python_scripts() {
    let dir = TempDir::new().unwrap();
    let root = create_python_project(&dir).await;

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    // Check scripts were found
    let script_names: Vec<_> = analysis.scripts.iter().map(|s| &s.name).collect();
    assert!(script_names.contains(&&"test".to_string()));
    assert!(script_names.contains(&&"lint".to_string()));
}

#[rstest]
#[tokio::test]
async fn test_nodejs_project_detection() {
    let dir = TempDir::new().unwrap();
    let root = create_nodejs_project(&dir).await;

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    assert!(analysis.ecosystems.contains(&Ecosystem::NodeJs));
}

#[rstest]
#[tokio::test]
async fn test_nodejs_dependencies() {
    let dir = TempDir::new().unwrap();
    let root = create_nodejs_project(&dir).await;

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    // Check dependencies were found
    let dep_names: Vec<_> = analysis.dependencies.iter().map(|d| &d.name).collect();
    assert!(dep_names.contains(&&"express".to_string()));
    assert!(dep_names.contains(&&"lodash".to_string()));
    assert!(dep_names.contains(&&"jest".to_string()));
}

#[rstest]
#[tokio::test]
async fn test_nodejs_scripts() {
    let dir = TempDir::new().unwrap();
    let root = create_nodejs_project(&dir).await;

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    // Check scripts were found
    let script_names: Vec<_> = analysis.scripts.iter().map(|s| &s.name).collect();
    assert!(script_names.contains(&&"test".to_string()));
    assert!(script_names.contains(&&"build".to_string()));
    assert!(script_names.contains(&&"start".to_string()));
}

#[rstest]
#[tokio::test]
async fn test_rust_project_detection() {
    let dir = TempDir::new().unwrap();
    let root = create_rust_project(&dir).await;

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    assert!(analysis.ecosystems.contains(&Ecosystem::Rust));
}

#[rstest]
#[tokio::test]
async fn test_rust_dependencies() {
    let dir = TempDir::new().unwrap();
    let root = create_rust_project(&dir).await;

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    // Check dependencies were found
    let dep_names: Vec<_> = analysis.dependencies.iter().map(|d| &d.name).collect();
    assert!(dep_names.contains(&&"tokio".to_string()));
    assert!(dep_names.contains(&&"serde".to_string()));
    assert!(dep_names.contains(&&"rstest".to_string()));
}

#[rstest]
#[tokio::test]
async fn test_empty_project() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().to_path_buf();

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    assert!(analysis.ecosystems.is_empty());
    assert!(analysis.dependencies.is_empty());
}

#[rstest]
#[tokio::test]
async fn test_multi_ecosystem_project() {
    let dir = TempDir::new().unwrap();

    // Create both Python and Node.js files
    create_python_project(&dir).await;
    create_nodejs_project(&dir).await;

    let root = dir.path().to_path_buf();

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    // Should detect both ecosystems
    assert!(analysis.ecosystems.contains(&Ecosystem::Python));
    assert!(analysis.ecosystems.contains(&Ecosystem::NodeJs));
}
