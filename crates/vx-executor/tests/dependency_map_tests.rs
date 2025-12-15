//! Tests for the dependency map module

use rstest::rstest;
use vx_executor::{DependencyMap, Ecosystem, RuntimeDependency, ToolSpec};

#[rstest]
fn test_builtin_tools_registered() {
    let map = DependencyMap::new();

    // Node.js ecosystem
    assert!(map.contains("node"));
    assert!(map.contains("npm"));
    assert!(map.contains("npx"));
    assert!(map.contains("yarn"));
    assert!(map.contains("pnpm"));
    assert!(map.contains("bun"));

    // Python ecosystem
    assert!(map.contains("uv"));
    assert!(map.contains("uvx"));
    assert!(map.contains("pip"));

    // Rust ecosystem
    assert!(map.contains("cargo"));
    assert!(map.contains("rustup"));

    // Go ecosystem
    assert!(map.contains("go"));
}

#[rstest]
fn test_alias_resolution() {
    let map = DependencyMap::new();

    // nodejs -> node
    assert_eq!(map.resolve_name("nodejs"), Some("node"));
    assert_eq!(map.resolve_name("node"), Some("node"));

    // pip3 -> pip
    assert_eq!(map.resolve_name("pip3"), Some("pip"));

    // golang -> go
    assert_eq!(map.resolve_name("golang"), Some("go"));
}

#[rstest]
fn test_npm_depends_on_node() {
    let map = DependencyMap::new();

    let npm = map.get("npm").expect("npm should exist");
    let deps = npm.required_dependencies();

    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].tool_name, "node");
}

#[rstest]
fn test_uvx_command_prefix() {
    let map = DependencyMap::new();

    let uvx = map.get("uvx").expect("uvx should exist");

    assert_eq!(uvx.get_executable(), "uv");
    assert_eq!(uvx.command_prefix, vec!["tool", "run"]);
}

#[rstest]
fn test_standalone_tools_no_dependencies() {
    let map = DependencyMap::new();

    // uv has no dependencies
    let uv = map.get("uv").expect("uv should exist");
    assert!(uv.required_dependencies().is_empty());

    // bun has no dependencies
    let bun = map.get("bun").expect("bun should exist");
    assert!(bun.required_dependencies().is_empty());

    // node has no dependencies
    let node = map.get("node").expect("node should exist");
    assert!(node.required_dependencies().is_empty());
}

#[rstest]
fn test_install_order_simple() {
    let map = DependencyMap::new();

    // npm depends on node, so node should come first
    let order = map.get_install_order("npm");

    let node_pos = order.iter().position(|&t| t == "node");
    let npm_pos = order.iter().position(|&t| t == "npm");

    assert!(node_pos.is_some(), "node should be in install order");
    assert!(npm_pos.is_some(), "npm should be in install order");
    assert!(
        node_pos.unwrap() < npm_pos.unwrap(),
        "node should come before npm"
    );
}

#[rstest]
fn test_install_order_standalone() {
    let map = DependencyMap::new();

    // uv has no dependencies
    let order = map.get_install_order("uv");
    assert_eq!(order, vec!["uv"]);
}

#[rstest]
fn test_ecosystem_filtering() {
    let map = DependencyMap::new();

    let node_tools = map.by_ecosystem(Ecosystem::Node);
    let node_names: Vec<_> = node_tools.iter().map(|t| t.name.as_str()).collect();

    assert!(node_names.contains(&"node"));
    assert!(node_names.contains(&"npm"));
    assert!(node_names.contains(&"yarn"));
    assert!(node_names.contains(&"pnpm"));
    assert!(node_names.contains(&"bun"));

    // uv should not be in Node ecosystem
    assert!(!node_names.contains(&"uv"));
}

#[rstest]
fn test_custom_tool_registration() {
    let mut map = DependencyMap::empty();

    let custom_tool = ToolSpec::new("my-tool", "My custom tool")
        .with_alias("mt")
        .with_ecosystem(Ecosystem::Generic)
        .with_dependency(RuntimeDependency::required("node", "Requires Node.js"));

    map.register(custom_tool);

    assert!(map.contains("my-tool"));
    assert!(map.contains("mt"));
    assert_eq!(map.resolve_name("mt"), Some("my-tool"));

    let tool = map.get("my-tool").unwrap();
    assert_eq!(tool.required_dependencies().len(), 1);
}

#[rstest]
fn test_tool_spec_matches() {
    let spec = ToolSpec::new("node", "Node.js")
        .with_alias("nodejs")
        .with_alias("node.js");

    assert!(spec.matches("node"));
    assert!(spec.matches("nodejs"));
    assert!(spec.matches("node.js"));
    assert!(!spec.matches("npm"));
}

#[rstest]
fn test_runtime_dependency_builder() {
    let dep = RuntimeDependency::required("node", "Node.js runtime required")
        .with_min_version(">=18.0.0")
        .provided_by("node-bundle");

    assert!(dep.required);
    assert_eq!(dep.tool_name, "node");
    assert_eq!(dep.min_version, Some(">=18.0.0".to_string()));
    assert_eq!(dep.provided_by, Some("node-bundle".to_string()));
}

#[rstest]
fn test_optional_dependency() {
    let dep = RuntimeDependency::optional("docker", "Optional for containerization");

    assert!(!dep.required);
    assert_eq!(dep.tool_name, "docker");
}
