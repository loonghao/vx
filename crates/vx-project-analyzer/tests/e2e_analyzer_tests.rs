//! E2E tests for project analyzer
//!
//! These tests analyze real-world projects including:
//! - The vx project itself
//! - Popular open source projects (cloned from GitHub)

use rstest::rstest;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use vx_project_analyzer::{AnalyzerConfig, Ecosystem, ProjectAnalyzer};

/// Get the root directory of the vx project
fn vx_project_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Go up from crates/vx-project-analyzer to project root
    manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Clone a GitHub repository to a temp directory
async fn clone_repo(url: &str, dir: &Path) -> bool {
    let output = Command::new("git")
        .args(["clone", "--depth", "1", url, dir.to_str().unwrap()])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

// ============================================
// VX Project Self-Analysis Tests
// ============================================

#[rstest]
#[tokio::test]
async fn test_analyze_vx_project() {
    let root = vx_project_root();

    let config = AnalyzerConfig {
        check_installed: true,
        check_tools: true,
        generate_sync_actions: true,
        max_depth: 3,
    };

    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    // VX is a Rust project
    assert!(
        analysis.ecosystems.contains(&Ecosystem::Rust),
        "VX should be detected as a Rust project"
    );

    // Should have dependencies (from root Cargo.toml)
    assert!(
        !analysis.dependencies.is_empty(),
        "VX should have dependencies"
    );

    // Check for known root-level dependencies
    let dep_names: Vec<_> = analysis.dependencies.iter().map(|d| &d.name).collect();

    // vx-cli is a direct dependency in root Cargo.toml
    assert!(
        dep_names.contains(&&"vx-cli".to_string()),
        "VX should depend on vx-cli"
    );

    // tokio is a direct dependency
    assert!(
        dep_names.contains(&&"tokio".to_string()),
        "VX should depend on tokio"
    );

    // Should have scripts (from justfile detection)
    assert!(
        !analysis.scripts.is_empty(),
        "VX should have detected scripts"
    );

    // Should have required tools
    assert!(
        !analysis.required_tools.is_empty(),
        "VX should have required tools"
    );

    // Rust toolchain should be required
    let tool_names: Vec<_> = analysis.required_tools.iter().map(|t| &t.name).collect();
    assert!(
        tool_names.contains(&&"rust".to_string()),
        "VX should require rust toolchain"
    );
}

#[rstest]
#[tokio::test]
async fn test_vx_project_has_cargo_lock() {
    let root = vx_project_root();

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    // Dependencies should be marked as installed (Cargo.lock exists)
    let installed_count = analysis
        .dependencies
        .iter()
        .filter(|d| d.is_installed)
        .count();
    assert!(
        installed_count > 0,
        "Some dependencies should be marked as installed"
    );
}

// ============================================
// Popular Open Source Project Tests
// ============================================

/// Test analyzing a Python project (uv - Python package manager)
#[rstest]
#[tokio::test]
#[ignore = "requires network access to clone repository"]
async fn test_analyze_uv_project() {
    let dir = TempDir::new().unwrap();
    let repo_path = dir.path().join("uv");

    // Clone uv repository
    if !clone_repo("https://github.com/astral-sh/uv.git", &repo_path).await {
        eprintln!("Failed to clone uv repository, skipping test");
        return;
    }

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&repo_path).await.unwrap();

    // UV is a Rust project
    assert!(
        analysis.ecosystems.contains(&Ecosystem::Rust),
        "uv should be detected as a Rust project"
    );

    // Should have many dependencies
    assert!(
        analysis.dependencies.len() > 10,
        "uv should have many dependencies"
    );
}

/// Test analyzing a Node.js project (express)
#[rstest]
#[tokio::test]
#[ignore = "requires network access to clone repository"]
async fn test_analyze_express_project() {
    let dir = TempDir::new().unwrap();
    let repo_path = dir.path().join("express");

    // Clone express repository
    if !clone_repo("https://github.com/expressjs/express.git", &repo_path).await {
        eprintln!("Failed to clone express repository, skipping test");
        return;
    }

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&repo_path).await.unwrap();

    // Express is a Node.js project
    assert!(
        analysis.ecosystems.contains(&Ecosystem::NodeJs),
        "express should be detected as a Node.js project"
    );

    // Should have dependencies
    assert!(
        !analysis.dependencies.is_empty(),
        "express should have dependencies"
    );

    // Should have scripts from package.json
    assert!(
        !analysis.scripts.is_empty(),
        "express should have npm scripts"
    );
}

/// Test analyzing a Python project (requests)
#[rstest]
#[tokio::test]
#[ignore = "requires network access to clone repository"]
async fn test_analyze_requests_project() {
    let dir = TempDir::new().unwrap();
    let repo_path = dir.path().join("requests");

    // Clone requests repository
    if !clone_repo("https://github.com/psf/requests.git", &repo_path).await {
        eprintln!("Failed to clone requests repository, skipping test");
        return;
    }

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&repo_path).await.unwrap();

    // Requests is a Python project
    assert!(
        analysis.ecosystems.contains(&Ecosystem::Python),
        "requests should be detected as a Python project"
    );
}

/// Test analyzing a Rust project (ripgrep)
#[rstest]
#[tokio::test]
#[ignore = "requires network access to clone repository"]
async fn test_analyze_ripgrep_project() {
    let dir = TempDir::new().unwrap();
    let repo_path = dir.path().join("ripgrep");

    // Clone ripgrep repository
    if !clone_repo("https://github.com/BurntSushi/ripgrep.git", &repo_path).await {
        eprintln!("Failed to clone ripgrep repository, skipping test");
        return;
    }

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&repo_path).await.unwrap();

    // Ripgrep is a Rust project
    assert!(
        analysis.ecosystems.contains(&Ecosystem::Rust),
        "ripgrep should be detected as a Rust project"
    );

    // Should have dependencies
    assert!(
        !analysis.dependencies.is_empty(),
        "ripgrep should have dependencies"
    );

    // Check for some known dependencies
    let dep_names: Vec<_> = analysis.dependencies.iter().map(|d| &d.name).collect();
    assert!(
        dep_names.contains(&&"regex".to_string()) || dep_names.contains(&&"grep".to_string()),
        "ripgrep should have regex-related dependencies"
    );
}

/// Test analyzing a mixed Python/Rust project (ruff)
#[rstest]
#[tokio::test]
#[ignore = "requires network access to clone repository"]
async fn test_analyze_ruff_project() {
    let dir = TempDir::new().unwrap();
    let repo_path = dir.path().join("ruff");

    // Clone ruff repository
    if !clone_repo("https://github.com/astral-sh/ruff.git", &repo_path).await {
        eprintln!("Failed to clone ruff repository, skipping test");
        return;
    }

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&repo_path).await.unwrap();

    // Ruff is primarily a Rust project but may have Python config
    assert!(
        analysis.ecosystems.contains(&Ecosystem::Rust),
        "ruff should be detected as a Rust project"
    );

    // Should have many dependencies
    assert!(
        analysis.dependencies.len() > 5,
        "ruff should have multiple dependencies"
    );
}

/// Test analyzing a Node.js project (vite)
#[rstest]
#[tokio::test]
#[ignore = "requires network access to clone repository"]
async fn test_analyze_vite_project() {
    let dir = TempDir::new().unwrap();
    let repo_path = dir.path().join("vite");

    // Clone vite repository
    if !clone_repo("https://github.com/vitejs/vite.git", &repo_path).await {
        eprintln!("Failed to clone vite repository, skipping test");
        return;
    }

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&repo_path).await.unwrap();

    // Vite is a Node.js project
    assert!(
        analysis.ecosystems.contains(&Ecosystem::NodeJs),
        "vite should be detected as a Node.js project"
    );

    // Should have scripts
    assert!(!analysis.scripts.is_empty(), "vite should have npm scripts");
}

// ============================================
// JSON Output Tests
// ============================================

#[rstest]
#[tokio::test]
async fn test_vx_project_json_serialization() {
    let root = vx_project_root();

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    // Should be serializable to JSON
    let json = serde_json::to_string_pretty(&analysis);
    assert!(json.is_ok(), "Analysis should be serializable to JSON");

    let json_str = json.unwrap();
    assert!(
        json_str.contains("\"ecosystems\""),
        "JSON should contain ecosystems"
    );
    assert!(
        json_str.contains("\"dependencies\""),
        "JSON should contain dependencies"
    );
    assert!(
        json_str.contains("\"scripts\""),
        "JSON should contain scripts"
    );
}

// ============================================
// Edge Case Tests
// ============================================

#[rstest]
#[tokio::test]
async fn test_analyze_nonexistent_directory() {
    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);

    let result = analyzer.analyze(Path::new("/nonexistent/path/12345")).await;

    // Should handle gracefully (either error or empty analysis)
    // The implementation may vary, but it shouldn't panic
    match result {
        Ok(analysis) => {
            assert!(analysis.ecosystems.is_empty());
        }
        Err(_) => {
            // Error is also acceptable for nonexistent path
        }
    }
}

#[rstest]
#[tokio::test]
async fn test_analyze_empty_directory() {
    let dir = TempDir::new().unwrap();

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(dir.path()).await.unwrap();

    assert!(analysis.ecosystems.is_empty());
    assert!(analysis.dependencies.is_empty());
    assert!(analysis.scripts.is_empty());
}

#[rstest]
#[tokio::test]
async fn test_analyze_with_malformed_config() {
    let dir = TempDir::new().unwrap();

    // Create a malformed pyproject.toml
    let malformed = "this is not valid toml [[[";
    tokio::fs::write(dir.path().join("pyproject.toml"), malformed)
        .await
        .unwrap();

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);

    // Should handle gracefully without panicking
    let result = analyzer.analyze(dir.path()).await;

    // May succeed with partial results or fail gracefully
    match result {
        Ok(analysis) => {
            // Partial analysis is acceptable
            println!("Partial analysis: {:?}", analysis.ecosystems);
        }
        Err(e) => {
            // Error is also acceptable for malformed config
            println!("Expected error for malformed config: {}", e);
        }
    }
}

// ============================================
// Performance Tests
// ============================================

#[rstest]
#[tokio::test]
async fn test_analyze_vx_project_performance() {
    let root = vx_project_root();

    let config = AnalyzerConfig::default();
    let analyzer = ProjectAnalyzer::new(config);

    let start = std::time::Instant::now();
    let _analysis = analyzer.analyze(&root).await.unwrap();
    let duration = start.elapsed();

    // Analysis should complete in reasonable time (< 5 seconds)
    assert!(
        duration.as_secs() < 5,
        "Analysis took too long: {:?}",
        duration
    );

    println!("VX project analysis completed in {:?}", duration);
}

// ============================================
// Sync Action Tests
// ============================================

#[rstest]
#[tokio::test]
async fn test_vx_project_sync_actions() {
    let root = vx_project_root();

    let config = AnalyzerConfig {
        check_installed: true,
        check_tools: true,
        generate_sync_actions: true,
        max_depth: 3,
    };

    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(&root).await.unwrap();

    // Should generate some sync actions
    // (unless .vx.toml is already perfectly in sync)
    println!(
        "Generated {} sync actions for VX project",
        analysis.sync_actions.len()
    );

    // Verify sync actions are valid
    for action in &analysis.sync_actions {
        let action_str = format!("{}", action);
        assert!(
            !action_str.is_empty(),
            "Sync action should have string representation"
        );
    }
}
