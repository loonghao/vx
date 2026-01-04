//! Tests for dependency audit functionality

use rstest::rstest;
use std::fs;
use tempfile::TempDir;
use vx_project_analyzer::{
    AnalyzerConfig, AuditFinding, AuditSeverity, ProjectAnalysis, ProjectAnalyzer,
};

#[rstest]
#[tokio::test]
async fn test_audit_missing_lockfile() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create a Node.js project with dependencies but no lockfile
    let package_json = r#"{
  "name": "test",
  "dependencies": {
    "react": "^18.0.0"
  }
}"#;
    fs::write(root.join("package.json"), package_json).unwrap();
    // Create node_modules to simulate installed deps
    fs::create_dir_all(root.join("node_modules")).unwrap();

    let config = AnalyzerConfig {
        check_installed: false,
        check_tools: false,
        generate_sync_actions: false,
        max_depth: 1,
    };

    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(root).await.unwrap();

    // Should have audit finding about missing lockfile
    let lockfile_findings: Vec<_> = analysis
        .audit_findings
        .iter()
        .filter(|f| f.title.contains("lockfile"))
        .collect();

    assert!(
        !lockfile_findings.is_empty(),
        "Should detect missing lockfile"
    );
}

#[rstest]
#[tokio::test]
async fn test_audit_unpinned_dependencies() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create a project with unpinned deps
    let package_json = r#"{
  "name": "test",
  "dependencies": {
    "react": "latest"
  }
}"#;
    fs::write(root.join("package.json"), package_json).unwrap();

    let config = AnalyzerConfig {
        check_installed: false,
        check_tools: false,
        generate_sync_actions: false,
        max_depth: 1,
    };

    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(root).await.unwrap();

    // Should have audit finding about unpinned deps
    let unpinned_findings: Vec<_> = analysis
        .audit_findings
        .iter()
        .filter(|f| f.title.contains("Unpinned"))
        .collect();

    assert!(
        !unpinned_findings.is_empty(),
        "Should detect unpinned dependencies"
    );
}

#[rstest]
#[tokio::test]
async fn test_audit_mixed_ecosystems() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create a project with both Node.js and Python indicators
    fs::write(root.join("package.json"), r#"{"name": "test"}"#).unwrap();
    fs::write(root.join("pyproject.toml"), r#"[project]\nname = 'test'"#).unwrap();

    let config = AnalyzerConfig {
        check_installed: false,
        check_tools: false,
        generate_sync_actions: false,
        max_depth: 1,
    };

    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(root).await.unwrap();

    // Should have audit finding about mixed ecosystems
    let mixed_findings: Vec<_> = analysis
        .audit_findings
        .iter()
        .filter(|f| f.title.contains("Mixed ecosystem"))
        .collect();

    assert!(!mixed_findings.is_empty(), "Should detect mixed ecosystems");
}

#[rstest]
#[tokio::test]
async fn test_audit_severity_levels() {
    let finding = AuditFinding::new(
        AuditSeverity::Critical,
        "Critical issue",
        "This is a critical problem",
    );

    assert_eq!(finding.severity, AuditSeverity::Critical);
    assert_eq!(finding.title, "Critical issue");
    assert_eq!(finding.detail, "This is a critical problem");
}

#[rstest]
#[tokio::test]
async fn test_has_critical_audits() {
    let mut analysis = ProjectAnalysis::new(tempfile::tempdir().unwrap().path().to_path_buf());

    analysis.audit_findings.push(AuditFinding::new(
        AuditSeverity::Critical,
        "Critical issue",
        "Details",
    ));
    analysis.audit_findings.push(AuditFinding::new(
        AuditSeverity::Warning,
        "Warning issue",
        "Details",
    ));

    assert!(analysis.has_critical_audits());
}
