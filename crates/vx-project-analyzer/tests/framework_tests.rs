//! Framework detection tests
//!
//! Tests for Electron and Tauri framework detection.

use rstest::rstest;
use std::fs;
use tempfile::TempDir;
use vx_project_analyzer::{AnalyzerConfig, ProjectAnalyzer, ProjectFramework};

/// Create a temporary directory with the given files
fn create_temp_project(files: &[(&str, &str)]) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    for (path, content) in files {
        let file_path = temp_dir.path().join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&file_path, content).unwrap();
    }
    temp_dir
}

// =============================================================================
// Electron Detection Tests
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_detect_electron_via_dependency() {
    let temp_dir = create_temp_project(&[(
        "package.json",
        r#"{
            "name": "my-electron-app",
            "devDependencies": {
                "electron": "^31.0.0"
            }
        }"#,
    )]);

    let analyzer = ProjectAnalyzer::new(AnalyzerConfig::default());
    let analysis = analyzer.analyze(temp_dir.path()).await.unwrap();

    assert_eq!(analysis.frameworks.len(), 1);
    assert_eq!(analysis.frameworks[0].framework, ProjectFramework::Electron);
    assert_eq!(analysis.frameworks[0].version, Some("31.0.0".to_string()));
}

#[rstest]
#[tokio::test]
async fn test_detect_electron_via_builder_config() {
    let temp_dir = create_temp_project(&[
        (
            "package.json",
            r#"{
                "name": "my-electron-app",
                "devDependencies": {
                    "electron": "^30.0.0",
                    "electron-builder": "^25.0.0"
                }
            }"#,
        ),
        ("electron-builder.json", r#"{"appId": "com.example.app"}"#),
    ]);

    let analyzer = ProjectAnalyzer::new(AnalyzerConfig::default());
    let analysis = analyzer.analyze(temp_dir.path()).await.unwrap();

    assert_eq!(analysis.frameworks.len(), 1);
    assert_eq!(analysis.frameworks[0].framework, ProjectFramework::Electron);
    assert_eq!(
        analysis.frameworks[0].build_tool,
        Some("electron-builder".to_string())
    );
}

#[rstest]
#[tokio::test]
async fn test_detect_electron_via_forge_config() {
    let temp_dir = create_temp_project(&[
        (
            "package.json",
            r#"{
                "name": "my-electron-app",
                "devDependencies": {
                    "electron": "^29.0.0",
                    "@electron-forge/cli": "^7.0.0"
                }
            }"#,
        ),
        ("forge.config.js", "module.exports = {};"),
    ]);

    let analyzer = ProjectAnalyzer::new(AnalyzerConfig::default());
    let analysis = analyzer.analyze(temp_dir.path()).await.unwrap();

    assert_eq!(analysis.frameworks.len(), 1);
    assert_eq!(analysis.frameworks[0].framework, ProjectFramework::Electron);
    assert_eq!(
        analysis.frameworks[0].build_tool,
        Some("electron-forge".to_string())
    );
}

#[rstest]
#[tokio::test]
async fn test_detect_electron_vite() {
    let temp_dir = create_temp_project(&[
        (
            "package.json",
            r#"{
                "name": "my-electron-vite-app",
                "devDependencies": {
                    "electron": "^28.0.0",
                    "electron-vite": "^2.0.0"
                }
            }"#,
        ),
        ("electron.vite.config.ts", "export default {};"),
    ]);

    let analyzer = ProjectAnalyzer::new(AnalyzerConfig::default());
    let analysis = analyzer.analyze(temp_dir.path()).await.unwrap();

    assert_eq!(analysis.frameworks.len(), 1);
    assert_eq!(analysis.frameworks[0].framework, ProjectFramework::Electron);
    assert_eq!(
        analysis.frameworks[0].metadata.get("bundler"),
        Some(&"electron-vite".to_string())
    );
}

#[rstest]
#[tokio::test]
async fn test_detect_electron_with_product_name() {
    let temp_dir = create_temp_project(&[(
        "package.json",
        r#"{
            "name": "my-electron-app",
            "productName": "My Awesome App",
            "devDependencies": {
                "electron": "^27.0.0"
            }
        }"#,
    )]);

    let analyzer = ProjectAnalyzer::new(AnalyzerConfig::default());
    let analysis = analyzer.analyze(temp_dir.path()).await.unwrap();

    assert_eq!(analysis.frameworks.len(), 1);
    assert_eq!(
        analysis.frameworks[0].metadata.get("productName"),
        Some(&"My Awesome App".to_string())
    );
}

#[rstest]
#[tokio::test]
async fn test_detect_electron_with_todesktop() {
    let temp_dir = create_temp_project(&[
        (
            "package.json",
            r#"{
                "name": "my-electron-app",
                "devDependencies": {
                    "electron": "^26.0.0"
                }
            }"#,
        ),
        ("todesktop.json", r#"{"id": "abc123"}"#),
    ]);

    let analyzer = ProjectAnalyzer::new(AnalyzerConfig::default());
    let analysis = analyzer.analyze(temp_dir.path()).await.unwrap();

    assert_eq!(analysis.frameworks.len(), 1);
    assert_eq!(
        analysis.frameworks[0].metadata.get("distribution"),
        Some(&"todesktop".to_string())
    );
}

// =============================================================================
// Tauri Detection Tests
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_detect_tauri_via_src_tauri() {
    let temp_dir = create_temp_project(&[
        (
            "package.json",
            r#"{
                "name": "my-tauri-app",
                "devDependencies": {
                    "@tauri-apps/cli": "^2.0.0"
                }
            }"#,
        ),
        (
            "src-tauri/tauri.conf.json",
            r#"{
                "productName": "My Tauri App",
                "identifier": "com.example.myapp"
            }"#,
        ),
        (
            "src-tauri/Cargo.toml",
            r#"
[package]
name = "my-tauri-app"
version = "0.1.0"

[dependencies]
tauri = "2.0"
"#,
        ),
    ]);

    let analyzer = ProjectAnalyzer::new(AnalyzerConfig::default());
    let analysis = analyzer.analyze(temp_dir.path()).await.unwrap();

    assert!(analysis
        .frameworks
        .iter()
        .any(|f| f.framework == ProjectFramework::Tauri));

    let tauri_info = analysis
        .frameworks
        .iter()
        .find(|f| f.framework == ProjectFramework::Tauri)
        .unwrap();
    assert_eq!(tauri_info.version, Some("2.x".to_string()));
    assert_eq!(
        tauri_info.metadata.get("productName"),
        Some(&"My Tauri App".to_string())
    );
    assert_eq!(
        tauri_info.metadata.get("identifier"),
        Some(&"com.example.myapp".to_string())
    );
}

#[rstest]
#[tokio::test]
async fn test_detect_tauri_v1() {
    let temp_dir = create_temp_project(&[
        (
            "package.json",
            r#"{
                "name": "my-tauri-v1-app",
                "devDependencies": {
                    "@tauri-apps/cli": "^1.5.0"
                }
            }"#,
        ),
        (
            "src-tauri/tauri.conf.json",
            r#"{
                "package": {
                    "productName": "Tauri V1 App"
                },
                "tauri": {
                    "bundle": {
                        "identifier": "com.example.tauriv1"
                    }
                }
            }"#,
        ),
        (
            "src-tauri/Cargo.toml",
            r#"
[package]
name = "tauri-v1-app"
version = "0.1.0"

[dependencies]
tauri = "1.6"
"#,
        ),
    ]);

    let analyzer = ProjectAnalyzer::new(AnalyzerConfig::default());
    let analysis = analyzer.analyze(temp_dir.path()).await.unwrap();

    assert!(analysis
        .frameworks
        .iter()
        .any(|f| f.framework == ProjectFramework::Tauri));

    let tauri_info = analysis
        .frameworks
        .iter()
        .find(|f| f.framework == ProjectFramework::Tauri)
        .unwrap();
    assert_eq!(tauri_info.version, Some("1.x".to_string()));
}

#[rstest]
#[tokio::test]
async fn test_detect_tauri_via_dependency() {
    let temp_dir = create_temp_project(&[(
        "package.json",
        r#"{
            "name": "my-tauri-app",
            "dependencies": {
                "@tauri-apps/api": "^2.0.0"
            },
            "devDependencies": {
                "@tauri-apps/cli": "^2.0.0"
            }
        }"#,
    )]);

    let analyzer = ProjectAnalyzer::new(AnalyzerConfig::default());
    let analysis = analyzer.analyze(temp_dir.path()).await.unwrap();

    assert!(analysis
        .frameworks
        .iter()
        .any(|f| f.framework == ProjectFramework::Tauri));
}

#[rstest]
#[tokio::test]
async fn test_detect_tauri_required_tools() {
    let temp_dir = create_temp_project(&[
        (
            "package.json",
            r#"{
                "name": "my-tauri-app",
                "scripts": {
                    "tauri": "tauri",
                    "dev": "tauri dev"
                },
                "devDependencies": {
                    "@tauri-apps/cli": "^2.0.0"
                }
            }"#,
        ),
        ("src-tauri/tauri.conf.json", r#"{"productName": "Test"}"#),
        (
            "src-tauri/Cargo.toml",
            r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
tauri = "2.0"
"#,
        ),
    ]);

    let analyzer = ProjectAnalyzer::new(AnalyzerConfig::default());
    let analysis = analyzer.analyze(temp_dir.path()).await.unwrap();

    // Should require both Rust and Node.js
    assert!(analysis.required_tools.iter().any(|t| t.name == "rust"));
    assert!(analysis.required_tools.iter().any(|t| t.name == "node"));
    // Should require tauri-cli
    assert!(analysis
        .required_tools
        .iter()
        .any(|t| t.name == "tauri-cli"));
}

// =============================================================================
// Mixed Framework Tests
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_no_framework_for_plain_nodejs() {
    let temp_dir = create_temp_project(&[(
        "package.json",
        r#"{
            "name": "my-node-app",
            "dependencies": {
                "express": "^4.18.0"
            }
        }"#,
    )]);

    let analyzer = ProjectAnalyzer::new(AnalyzerConfig::default());
    let analysis = analyzer.analyze(temp_dir.path()).await.unwrap();

    assert!(analysis.frameworks.is_empty());
}

#[rstest]
#[tokio::test]
async fn test_no_framework_for_plain_rust() {
    let temp_dir = create_temp_project(&[(
        "Cargo.toml",
        r#"
[package]
name = "my-rust-app"
version = "0.1.0"

[dependencies]
tokio = "1.0"
"#,
    )]);

    let analyzer = ProjectAnalyzer::new(AnalyzerConfig::default());
    let analysis = analyzer.analyze(temp_dir.path()).await.unwrap();

    assert!(analysis.frameworks.is_empty());
}
