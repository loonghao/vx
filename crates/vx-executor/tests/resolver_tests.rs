//! Tests for the tool resolver module

use rstest::rstest;
use vx_executor::{ExecutorConfig, ToolResolver, ToolStatus};

#[rstest]
fn test_resolver_creation() {
    let config = ExecutorConfig::default();
    let resolver = ToolResolver::new(config);
    assert!(resolver.is_ok());
}

#[rstest]
fn test_known_tools() {
    let config = ExecutorConfig::default();
    let resolver = ToolResolver::new(config).unwrap();

    assert!(resolver.is_known_tool("node"));
    assert!(resolver.is_known_tool("npm"));
    assert!(resolver.is_known_tool("uv"));
    assert!(resolver.is_known_tool("cargo"));
    assert!(resolver.is_known_tool("go"));
}

#[rstest]
fn test_unknown_tool() {
    let config = ExecutorConfig::default();
    let resolver = ToolResolver::new(config).unwrap();

    assert!(!resolver.is_known_tool("nonexistent-tool-xyz"));
}

#[rstest]
fn test_tool_status_is_available() {
    use std::path::PathBuf;

    // VxManaged is available
    let vx_managed = ToolStatus::VxManaged {
        version: "1.0.0".into(),
        path: PathBuf::from("/usr/bin/node"),
    };
    assert!(vx_managed.is_available());

    // SystemAvailable is available
    let system = ToolStatus::SystemAvailable {
        path: PathBuf::from("/usr/bin/node"),
    };
    assert!(system.is_available());

    // NotInstalled is not available
    assert!(!ToolStatus::NotInstalled.is_available());

    // Unknown is not available
    assert!(!ToolStatus::Unknown.is_available());
}

#[rstest]
fn test_tool_status_executable_path() {
    use std::path::PathBuf;

    let path = PathBuf::from("/usr/bin/node");

    let vx_managed = ToolStatus::VxManaged {
        version: "1.0.0".into(),
        path: path.clone(),
    };
    assert_eq!(vx_managed.executable_path(), Some(&path));

    let system = ToolStatus::SystemAvailable { path: path.clone() };
    assert_eq!(system.executable_path(), Some(&path));

    assert_eq!(ToolStatus::NotInstalled.executable_path(), None);
    assert_eq!(ToolStatus::Unknown.executable_path(), None);
}

#[rstest]
fn test_resolve_known_tool() {
    let config = ExecutorConfig::default();
    let resolver = ToolResolver::new(config).unwrap();

    // Resolve npm - it should identify node as a dependency
    let resolution = resolver.resolve("npm");
    assert!(resolution.is_ok());

    let result = resolution.unwrap();
    assert_eq!(result.tool, "npm");
}

#[rstest]
fn test_resolve_tool_with_prefix() {
    let config = ExecutorConfig::default();
    let resolver = ToolResolver::new(config).unwrap();

    // uvx should have command prefix
    let resolution = resolver.resolve("uvx");
    assert!(resolution.is_ok());

    let result = resolution.unwrap();
    assert_eq!(result.command_prefix, vec!["tool", "run"]);
}

#[rstest]
fn test_get_spec() {
    let config = ExecutorConfig::default();
    let resolver = ToolResolver::new(config).unwrap();

    let npm_spec = resolver.get_spec("npm");
    assert!(npm_spec.is_some());

    let spec = npm_spec.unwrap();
    assert_eq!(spec.name, "npm");
    assert!(!spec.required_dependencies().is_empty());
}

#[rstest]
fn test_all_known_tools() {
    let config = ExecutorConfig::default();
    let resolver = ToolResolver::new(config).unwrap();

    let tools = resolver.known_tools();

    // Should have a reasonable number of tools
    assert!(tools.len() >= 10);

    // Should include common tools
    assert!(tools.contains(&"node"));
    assert!(tools.contains(&"npm"));
    assert!(tools.contains(&"uv"));
}
