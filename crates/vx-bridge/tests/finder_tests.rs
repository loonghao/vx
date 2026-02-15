//! Tests for the ExecutableFinder.

use std::path::PathBuf;
use tempfile::TempDir;
use vx_bridge::{ExecutableFinder, SearchStrategy};

// ============================================
// ExecutableFinder with AbsolutePaths strategy
// ============================================

#[test]
fn test_finder_absolute_paths_existing() {
    let temp_dir = TempDir::new().unwrap();
    let exe_path = temp_dir.path().join("test_tool.exe");
    std::fs::write(&exe_path, b"fake-exe").unwrap();

    let finder = ExecutableFinder::new(vec![SearchStrategy::AbsolutePaths(vec![exe_path.clone()])]);

    let result = finder.find();
    assert!(result.is_some(), "should find existing absolute path");
    assert_eq!(result.unwrap(), exe_path);
}

#[test]
fn test_finder_absolute_paths_nonexistent() {
    let finder = ExecutableFinder::new(vec![SearchStrategy::AbsolutePaths(vec![PathBuf::from(
        "/nonexistent/path/tool.exe",
    )])]);

    let result = finder.find();
    assert!(result.is_none(), "should not find nonexistent path");
}

#[test]
fn test_finder_multiple_strategies_first_match_wins() {
    let temp_dir = TempDir::new().unwrap();

    let first_path = temp_dir.path().join("first_tool.exe");
    let second_path = temp_dir.path().join("second_tool.exe");
    std::fs::write(&first_path, b"first").unwrap();
    std::fs::write(&second_path, b"second").unwrap();

    let finder = ExecutableFinder::new(vec![
        SearchStrategy::AbsolutePaths(vec![first_path.clone()]),
        SearchStrategy::AbsolutePaths(vec![second_path.clone()]),
    ]);

    let result = finder.find();
    assert_eq!(
        result.unwrap(),
        first_path,
        "first matching strategy should win"
    );
}

#[test]
fn test_finder_skips_failed_strategy_and_tries_next() {
    let temp_dir = TempDir::new().unwrap();
    let existing = temp_dir.path().join("real_tool.exe");
    std::fs::write(&existing, b"real").unwrap();

    let finder = ExecutableFinder::new(vec![
        SearchStrategy::AbsolutePaths(vec![PathBuf::from("/no/such/file.exe")]),
        SearchStrategy::AbsolutePaths(vec![existing.clone()]),
    ]);

    let result = finder.find();
    assert_eq!(
        result.unwrap(),
        existing,
        "should fallback to second strategy"
    );
}

#[test]
fn test_finder_no_strategies_returns_none() {
    let finder = ExecutableFinder::new(vec![]);
    assert!(
        finder.find().is_none(),
        "empty strategies should return None"
    );
}

// ============================================
// SystemPath strategy
// ============================================

#[test]
fn test_finder_system_path_finds_common_tool() {
    let tool_name = if cfg!(windows) { "cmd" } else { "sh" };

    let finder = ExecutableFinder::new(vec![SearchStrategy::SystemPath(tool_name.to_string())]);

    let result = finder.find();
    assert!(
        result.is_some(),
        "{} should be found in system PATH",
        tool_name
    );
}

#[test]
fn test_finder_system_path_nonexistent_tool() {
    let finder = ExecutableFinder::new(vec![SearchStrategy::SystemPath(
        "vx_nonexistent_tool_xyz_12345".to_string(),
    )]);

    let result = finder.find();
    assert!(result.is_none(), "nonexistent tool should not be found");
}
