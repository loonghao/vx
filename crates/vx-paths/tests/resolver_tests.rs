//! PathResolver tests for conda-tools

use tempfile::TempDir;
use vx_paths::{PathManager, PathResolver, ToolSource};

// ============================================================================
// Conda-tools Resolver Tests
// ============================================================================

#[test]
fn test_conda_tools_resolver() {
    let temp_dir = TempDir::new().unwrap();
    let manager = PathManager::with_base_dir(temp_dir.path()).unwrap();
    let resolver = PathResolver::new(manager);

    // Initially no conda tools
    let location = resolver.find_in_conda_tools("pytorch").unwrap();
    assert!(location.is_none());

    // Create a conda tool installation
    // ~/.vx/conda-tools/pytorch/2.2.0/env/bin/python (Unix)
    // ~/.vx/conda-tools/pytorch/2.2.0/env/Scripts/python.exe (Windows)
    let bin_dir = resolver.manager().conda_tool_bin_dir("pytorch", "2.2.0");
    std::fs::create_dir_all(&bin_dir).unwrap();
    let exe_name = if cfg!(windows) {
        "python.exe"
    } else {
        "python"
    };
    let exe_path = bin_dir.join(exe_name);
    std::fs::write(&exe_path, "fake python").unwrap();

    // Should find the conda tool
    let location = resolver.find_in_conda_tools("pytorch").unwrap();
    assert!(location.is_some());
    let loc = location.unwrap();
    assert_eq!(loc.version, "2.2.0");
    assert_eq!(loc.source, ToolSource::CondaTools);
    assert_eq!(loc.path, exe_path);
}

#[test]
fn test_conda_tools_multiple_versions() {
    let temp_dir = TempDir::new().unwrap();
    let manager = PathManager::with_base_dir(temp_dir.path()).unwrap();
    let resolver = PathResolver::new(manager);

    // Create multiple versions of a conda tool
    for version in &["2.0.0", "2.1.0", "2.2.0"] {
        let bin_dir = resolver.manager().conda_tool_bin_dir("pytorch", version);
        std::fs::create_dir_all(&bin_dir).unwrap();
        let exe_name = if cfg!(windows) {
            "python.exe"
        } else {
            "python"
        };
        let exe_path = bin_dir.join(exe_name);
        std::fs::write(&exe_path, "fake python").unwrap();
    }

    // find_in_conda_tools should return the latest version
    let location = resolver.find_in_conda_tools("pytorch").unwrap();
    assert!(location.is_some());
    let loc = location.unwrap();
    assert_eq!(loc.version, "2.2.0");

    // find_all_in_conda_tools should return all versions
    let locations = resolver.find_all_in_conda_tools("pytorch").unwrap();
    assert_eq!(locations.len(), 3);
    let versions: Vec<&str> = locations.iter().map(|l| l.version.as_str()).collect();
    assert!(versions.contains(&"2.0.0"));
    assert!(versions.contains(&"2.1.0"));
    assert!(versions.contains(&"2.2.0"));
}

#[test]
fn test_tool_source_display() {
    assert_eq!(ToolSource::Store.to_string(), "store");
    assert_eq!(ToolSource::NpmTools.to_string(), "npm-tools");
    assert_eq!(ToolSource::PipTools.to_string(), "pip-tools");
    assert_eq!(ToolSource::CondaTools.to_string(), "conda-tools");
}

#[test]
fn test_find_tool_includes_conda_tools() {
    let temp_dir = TempDir::new().unwrap();
    let manager = PathManager::with_base_dir(temp_dir.path()).unwrap();
    let resolver = PathResolver::new(manager);

    // Create a conda tool
    let bin_dir = resolver.manager().conda_tool_bin_dir("mytool", "1.0.0");
    std::fs::create_dir_all(&bin_dir).unwrap();
    let exe_name = if cfg!(windows) {
        "mytool.exe"
    } else {
        "mytool"
    };
    let exe_path = bin_dir.join(exe_name);
    std::fs::write(&exe_path, "fake tool").unwrap();

    // find_tool should find it
    let location = resolver.find_tool("mytool").unwrap();
    assert!(location.is_some());
    let loc = location.unwrap();
    assert_eq!(loc.source, ToolSource::CondaTools);

    // find_all_tools should include it
    let locations = resolver.find_all_tools("mytool").unwrap();
    assert!(!locations.is_empty());
    assert!(locations.iter().any(|l| l.source == ToolSource::CondaTools));
}
