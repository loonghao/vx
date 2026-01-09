//! Tests for RuntimeMap using manifest-based construction (RFC 0017)
//!
//! These tests verify that RuntimeMap correctly loads runtime specifications
//! from provider.toml manifests (single source of truth).

use rstest::rstest;
use vx_manifest::ProviderManifest;
use vx_resolver::{Ecosystem, RuntimeMap};

/// Helper function to create a RuntimeMap from test manifests
fn create_test_runtime_map() -> RuntimeMap {
    let manifests = vec![
        create_node_manifest(),
        create_python_manifest(),
        create_rust_manifest(),
    ];
    RuntimeMap::from_manifests(&manifests)
}

fn create_node_manifest() -> ProviderManifest {
    let toml = r#"
[provider]
name = "node"
ecosystem = "nodejs"

[[runtimes]]
name = "node"
description = "Node.js JavaScript runtime"
executable = "node"
aliases = ["nodejs"]
priority = 100

[[runtimes]]
name = "npm"
description = "Node.js package manager"
executable = "npm"
bundled_with = "node"

[[runtimes]]
name = "yarn"
description = "Fast, reliable dependency management"
executable = "yarn"

[dependencies.node]
runtime = "node"
required = true
reason = "yarn requires Node.js runtime"
min_version = "12.0.0"
max_version = "22.99.99"
recommended_version = "20"
"#;
    ProviderManifest::parse(toml).expect("Failed to parse node manifest")
}

fn create_python_manifest() -> ProviderManifest {
    let toml = r#"
[provider]
name = "python"
ecosystem = "python"

[[runtimes]]
name = "python"
description = "Python programming language"
executable = "python"
aliases = ["python3", "py"]
priority = 100

[[runtimes]]
name = "uv"
description = "An extremely fast Python package installer"
executable = "uv"
priority = 100

[[runtimes]]
name = "uvx"
description = "Python application runner"
executable = "uv"
command_prefix = ["tool", "run"]
bundled_with = "uv"

[[runtimes]]
name = "pip"
description = "Python package installer"
executable = "pip"
aliases = ["pip3"]

[dependencies.python]
runtime = "python"
required = true
reason = "pip requires Python runtime"
"#;
    ProviderManifest::parse(toml).expect("Failed to parse python manifest")
}

fn create_rust_manifest() -> ProviderManifest {
    let toml = r#"
[provider]
name = "rust"
ecosystem = "rust"

[[runtimes]]
name = "rustup"
description = "The Rust toolchain installer"
executable = "rustup"
priority = 100

[[runtimes]]
name = "cargo"
description = "Rust package manager and build tool"
executable = "cargo"
managed_by = "rustup"

[[runtimes]]
name = "rustc"
description = "The Rust compiler"
executable = "rustc"
managed_by = "rustup"
"#;
    ProviderManifest::parse(toml).expect("Failed to parse rust manifest")
}

#[rstest]
fn test_runtime_map_creation() {
    let map = create_test_runtime_map();

    assert!(map.contains("node"));
    assert!(map.contains("nodejs")); // alias
    assert!(map.contains("npm"));
    assert!(map.contains("uv"));
    assert!(map.contains("cargo"));
}

#[rstest]
fn test_python_runtime_registration() {
    let map = create_test_runtime_map();

    // Python should be registered
    assert!(map.contains("python"));
    assert!(map.contains("python3")); // alias
    assert!(map.contains("py")); // alias

    // Check Python spec
    let python = map.get("python").unwrap();
    assert_eq!(python.name, "python");
    assert_eq!(python.ecosystem, Ecosystem::Python);
    assert!(python.aliases.contains(&"python3".to_string()));
    assert!(python.aliases.contains(&"py".to_string()));
}

#[rstest]
fn test_alias_resolution() {
    let map = create_test_runtime_map();

    assert_eq!(map.resolve_name("nodejs"), Some("node"));
    assert_eq!(map.resolve_name("node"), Some("node"));
    assert_eq!(map.resolve_name("pip3"), Some("pip"));
    assert_eq!(map.resolve_name("python3"), Some("python"));
    assert_eq!(map.resolve_name("py"), Some("python"));
}

#[rstest]
fn test_dependency_lookup() {
    let map = create_test_runtime_map();

    let npm = map.get("npm").unwrap();
    assert_eq!(npm.required_dependencies().len(), 1);
    assert_eq!(npm.required_dependencies()[0].runtime_name, "node");
}

#[rstest]
fn test_ecosystem_filtering() {
    let map = create_test_runtime_map();

    let node_runtimes = map.by_ecosystem(Ecosystem::Node);
    assert!(node_runtimes.iter().any(|t| t.name == "node"));
    assert!(node_runtimes.iter().any(|t| t.name == "npm"));
    assert!(node_runtimes.iter().any(|t| t.name == "yarn"));
}

#[rstest]
fn test_install_order() {
    let map = create_test_runtime_map();

    // npm depends on node, so node should come first
    let order = map.get_install_order("npm");
    let node_pos = order.iter().position(|&t| t == "node");
    let npm_pos = order.iter().position(|&t| t == "npm");

    assert!(node_pos.is_some());
    assert!(npm_pos.is_some());
    assert!(node_pos.unwrap() < npm_pos.unwrap());
}

#[rstest]
fn test_uvx_command_prefix() {
    let map = create_test_runtime_map();

    let uvx = map.get("uvx").unwrap();
    assert_eq!(uvx.get_executable(), "uv");
    assert_eq!(uvx.command_prefix, vec!["tool", "run"]);
}

#[rstest]
fn test_standalone_runtimes() {
    let map = create_test_runtime_map();

    // uv has no dependencies
    let uv = map.get("uv").unwrap();
    assert!(uv.required_dependencies().is_empty());
}

#[rstest]
fn test_empty_runtime_map() {
    let map = RuntimeMap::empty();
    assert!(!map.contains("node"));
    assert!(map.runtime_names().is_empty());
}

#[rstest]
fn test_managed_by_creates_dependency() {
    let map = create_test_runtime_map();

    // cargo should have rustup as a dependency
    let cargo = map.get("cargo").unwrap();
    assert_eq!(cargo.required_dependencies().len(), 1);
    assert_eq!(cargo.required_dependencies()[0].runtime_name, "rustup");

    // rustc should also have rustup as a dependency
    let rustc = map.get("rustc").unwrap();
    assert_eq!(rustc.required_dependencies().len(), 1);
    assert_eq!(rustc.required_dependencies()[0].runtime_name, "rustup");
}
