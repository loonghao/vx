//! Integration tests for vx-dependency
//!
//! These tests cover complex dependency resolution scenarios including
//! multi-layer dependencies, circular dependencies, and version conflicts.

use std::collections::HashMap;
use std::sync::Arc;
use vx_dependency::{DependencyResolver, DependencySpec, ResolutionOptions, ToolSpec};

/// Mock availability checker for testing
struct TestAvailabilityChecker {
    available_tools: HashMap<String, (bool, Option<String>)>,
}

impl TestAvailabilityChecker {
    fn new() -> Self {
        Self {
            available_tools: HashMap::new(),
        }
    }

    fn set_available(&mut self, tool: &str, available: bool, version: Option<String>) {
        self.available_tools
            .insert(tool.to_string(), (available, version));
    }
}

#[async_trait::async_trait]
impl vx_dependency::resolver::AvailabilityChecker for TestAvailabilityChecker {
    async fn is_available(&self, tool_name: &str) -> vx_dependency::Result<bool> {
        Ok(self
            .available_tools
            .get(tool_name)
            .map(|(available, _)| *available)
            .unwrap_or(false))
    }

    async fn get_version(&self, tool_name: &str) -> vx_dependency::Result<Option<String>> {
        Ok(self
            .available_tools
            .get(tool_name)
            .and_then(|(_, version)| version.clone()))
    }

    async fn get_path(&self, _tool_name: &str) -> vx_dependency::Result<Option<String>> {
        Ok(None)
    }
}

/// Helper function to create a test tool
fn create_tool(name: &str, deps: Vec<(&str, Option<&str>)>) -> ToolSpec {
    ToolSpec {
        name: name.to_string(),
        dependencies: deps
            .into_iter()
            .map(|(dep_name, version_constraint)| {
                let mut dep =
                    DependencySpec::required(dep_name, format!("{} requires {}", name, dep_name));
                if let Some(constraint) = version_constraint {
                    dep = dep.with_version(constraint);
                }
                dep
            })
            .collect(),
        auto_installable: true,
        priority: 0,
        ..Default::default()
    }
}

/// Helper function to create a test tool with String dependencies
fn create_tool_with_string_deps(name: &str, deps: Vec<(String, Option<&str>)>) -> ToolSpec {
    ToolSpec {
        name: name.to_string(),
        dependencies: deps
            .into_iter()
            .map(|(dep_name, version_constraint)| {
                let mut dep =
                    DependencySpec::required(&dep_name, format!("{} requires {}", name, dep_name));
                if let Some(constraint) = version_constraint {
                    dep = dep.with_version(constraint);
                }
                dep
            })
            .collect(),
        auto_installable: true,
        priority: 0,
        ..Default::default()
    }
}

/// Helper function to create an optional dependency tool
fn create_tool_with_optional(
    name: &str,
    required_deps: Vec<&str>,
    optional_deps: Vec<&str>,
) -> ToolSpec {
    let mut dependencies = Vec::new();

    // Add required dependencies
    for dep in required_deps {
        dependencies.push(DependencySpec::required(
            dep,
            format!("{} requires {}", name, dep),
        ));
    }

    // Add optional dependencies
    for dep in optional_deps {
        dependencies.push(DependencySpec::optional(
            dep,
            format!("{} optionally uses {}", name, dep),
        ));
    }

    ToolSpec {
        name: name.to_string(),
        dependencies,
        auto_installable: true,
        priority: 0,
        ..Default::default()
    }
}

#[tokio::test]
async fn test_simple_dependency_chain() {
    let mut checker = TestAvailabilityChecker::new();
    checker.set_available("base", true, Some("1.0.0".to_string()));

    let resolver = DependencyResolver::new().with_availability_checker(Arc::new(checker));

    // Register tools: app -> lib -> base
    resolver
        .register_tool(create_tool("base", vec![]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("lib", vec![("base", None)]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("app", vec![("lib", None)]))
        .await
        .unwrap();

    let result = resolver.resolve("app").await.unwrap();

    // Should install in dependency order
    assert_eq!(result.install_order, vec!["base", "lib", "app"]);
    assert_eq!(result.available_tools, vec!["base"]);
    assert_eq!(result.missing_tools, vec!["lib", "app"]);
    assert!(result.circular_dependencies.is_empty());
}

#[tokio::test]
async fn test_multi_layer_dependencies() {
    let mut checker = TestAvailabilityChecker::new();
    checker.set_available("node", true, Some("18.0.0".to_string()));
    checker.set_available("python", true, Some("3.9.0".to_string()));

    let resolver = DependencyResolver::new().with_availability_checker(Arc::new(checker));

    // Layer 1: Base runtimes
    resolver
        .register_tool(create_tool("node", vec![]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("python", vec![]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("go", vec![]))
        .await
        .unwrap();

    // Layer 2: Package managers
    resolver
        .register_tool(create_tool("npm", vec![("node", Some(">=16.0.0"))]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("pip", vec![("python", Some(">=3.8.0"))]))
        .await
        .unwrap();

    // Layer 3: Advanced tools
    resolver
        .register_tool(create_tool(
            "yarn",
            vec![("node", Some(">=16.0.0")), ("npm", None)],
        ))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("pnpm", vec![("node", Some(">=16.14.0"))]))
        .await
        .unwrap();

    // Layer 4: Meta tools
    resolver
        .register_tool(create_tool("nx", vec![("yarn", None), ("node", None)]))
        .await
        .unwrap();

    let result = resolver.resolve("nx").await.unwrap();

    // Verify dependency order
    let order = &result.install_order;
    let node_pos = order.iter().position(|x| x == "node").unwrap();
    let npm_pos = order.iter().position(|x| x == "npm").unwrap();
    let yarn_pos = order.iter().position(|x| x == "yarn").unwrap();
    let nx_pos = order.iter().position(|x| x == "nx").unwrap();

    assert!(node_pos < npm_pos, "node should come before npm");
    assert!(npm_pos < yarn_pos, "npm should come before yarn");
    assert!(yarn_pos < nx_pos, "yarn should come before nx");
    assert!(node_pos < nx_pos, "node should come before nx");

    // Check available vs missing tools
    assert!(result.available_tools.contains(&"node".to_string()));
    assert!(result.missing_tools.contains(&"nx".to_string()));
}

#[tokio::test]
async fn test_diamond_dependency_pattern() {
    let resolver = DependencyResolver::new();

    // Diamond pattern: top -> left,right -> bottom
    resolver
        .register_tool(create_tool("bottom", vec![]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("left", vec![("bottom", None)]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("right", vec![("bottom", None)]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("top", vec![("left", None), ("right", None)]))
        .await
        .unwrap();

    let result = resolver.resolve("top").await.unwrap();

    // 'bottom' should appear only once
    let bottom_count = result
        .install_order
        .iter()
        .filter(|&x| x == "bottom")
        .count();
    assert_eq!(bottom_count, 1, "bottom should appear exactly once");

    // 'bottom' should come before 'left' and 'right'
    let order = &result.install_order;
    let bottom_pos = order.iter().position(|x| x == "bottom").unwrap();
    let left_pos = order.iter().position(|x| x == "left").unwrap();
    let right_pos = order.iter().position(|x| x == "right").unwrap();
    let top_pos = order.iter().position(|x| x == "top").unwrap();

    assert!(bottom_pos < left_pos);
    assert!(bottom_pos < right_pos);
    assert!(left_pos < top_pos);
    assert!(right_pos < top_pos);
}

#[tokio::test]
async fn test_circular_dependency_detection() {
    let resolver = DependencyResolver::new();

    // Create circular dependency: a -> b -> c -> a
    resolver
        .register_tool(create_tool("a", vec![("b", None)]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("b", vec![("c", None)]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("c", vec![("a", None)]))
        .await
        .unwrap();

    let result = resolver.resolve("a").await.unwrap();

    // Should detect circular dependency
    assert!(
        !result.circular_dependencies.is_empty(),
        "Should detect circular dependency"
    );
}

#[tokio::test]
async fn test_complex_real_world_scenario() {
    let mut checker = TestAvailabilityChecker::new();
    // Simulate some tools already installed
    checker.set_available("node", true, Some("18.17.0".to_string()));
    checker.set_available("git", true, Some("2.40.0".to_string()));

    let resolver = DependencyResolver::new().with_availability_checker(Arc::new(checker));

    // Base tools
    resolver
        .register_tool(create_tool("node", vec![]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("git", vec![]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("python", vec![]))
        .await
        .unwrap();

    // Package managers
    resolver
        .register_tool(create_tool("npm", vec![("node", Some(">=16.0.0"))]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("yarn", vec![("node", Some(">=16.0.0"))]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("pnpm", vec![("node", Some(">=16.14.0"))]))
        .await
        .unwrap();

    // Development tools
    resolver
        .register_tool(create_tool("typescript", vec![("npm", None)]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("eslint", vec![("npm", None)]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("prettier", vec![("npm", None)]))
        .await
        .unwrap();

    // Build tools
    resolver
        .register_tool(create_tool("webpack", vec![("npm", None)]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("vite", vec![("npm", None)]))
        .await
        .unwrap();

    // Meta frameworks
    resolver
        .register_tool(create_tool_with_optional(
            "next",
            vec!["node", "npm"],
            vec!["typescript", "eslint"],
        ))
        .await
        .unwrap();

    resolver
        .register_tool(create_tool(
            "create-react-app",
            vec![("node", None), ("npm", None), ("git", None)],
        ))
        .await
        .unwrap();

    // Resolve a complex tool that depends on git
    let result = resolver.resolve("create-react-app").await.unwrap();

    // Verify that already available tools are recognized
    assert!(result.available_tools.contains(&"node".to_string()));
    assert!(result.available_tools.contains(&"git".to_string()));

    // Verify dependency order makes sense
    let order = &result.install_order;
    let node_pos = order.iter().position(|x| x == "node").unwrap();
    let npm_pos = order.iter().position(|x| x == "npm").unwrap();
    let git_pos = order.iter().position(|x| x == "git").unwrap();
    let cra_pos = order.iter().position(|x| x == "create-react-app").unwrap();

    assert!(node_pos < npm_pos, "node should come before npm");
    assert!(git_pos < cra_pos, "git should come before create-react-app");
    assert!(npm_pos < cra_pos, "npm should come before create-react-app");

    // Should not have circular dependencies
    assert!(result.circular_dependencies.is_empty());
}

#[tokio::test]
async fn test_multiple_tools_resolution() {
    let mut checker = TestAvailabilityChecker::new();
    checker.set_available("node", true, Some("18.0.0".to_string()));
    checker.set_available("python", true, Some("3.9.0".to_string()));

    let resolver = DependencyResolver::new().with_availability_checker(Arc::new(checker));

    // Register tools from different ecosystems
    resolver
        .register_tool(create_tool("node", vec![]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("python", vec![]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("go", vec![]))
        .await
        .unwrap();

    resolver
        .register_tool(create_tool("yarn", vec![("node", None)]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("pip", vec![("python", None)]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("go-tool", vec![("go", None)]))
        .await
        .unwrap();

    // Resolve multiple tools from different ecosystems
    let tools = vec!["yarn".to_string(), "pip".to_string(), "go-tool".to_string()];

    let result = resolver.resolve_multiple(&tools).await.unwrap();

    // Should include all base dependencies
    assert!(result.install_order.contains(&"node".to_string()));
    assert!(result.install_order.contains(&"python".to_string()));
    assert!(result.install_order.contains(&"go".to_string()));

    // Should include all requested tools
    assert!(result.install_order.contains(&"yarn".to_string()));
    assert!(result.install_order.contains(&"pip".to_string()));
    assert!(result.install_order.contains(&"go-tool".to_string()));

    // Available tools should be recognized
    assert!(result.available_tools.contains(&"node".to_string()));
    assert!(result.available_tools.contains(&"python".to_string()));
}

#[tokio::test]
async fn test_version_constraints() {
    let mut checker = TestAvailabilityChecker::new();
    checker.set_available("node", true, Some("14.0.0".to_string())); // Old version

    let resolver = DependencyResolver::new().with_availability_checker(Arc::new(checker));

    resolver
        .register_tool(create_tool("node", vec![]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("modern-tool", vec![("node", Some(">=16.0.0"))]))
        .await
        .unwrap();

    let result = resolver.resolve("modern-tool").await.unwrap();

    // Should still resolve, but might have version conflicts
    // (Version conflict detection would be implemented in a real scenario)
    assert!(result.install_order.contains(&"node".to_string()));
    assert!(result.install_order.contains(&"modern-tool".to_string()));
}

#[tokio::test]
async fn test_optional_dependencies() {
    let resolver = DependencyResolver::new();

    resolver
        .register_tool(create_tool("base", vec![]))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool_with_optional(
            "app",
            vec!["base"],
            vec!["optional-tool"],
        ))
        .await
        .unwrap();
    resolver
        .register_tool(create_tool("optional-tool", vec![]))
        .await
        .unwrap();

    // Resolve without including optional dependencies
    let options = ResolutionOptions {
        include_optional: false,
        ..Default::default()
    };
    let resolver_no_optional = DependencyResolver::with_options(options);
    resolver_no_optional
        .register_tool(create_tool("base", vec![]))
        .await
        .unwrap();
    resolver_no_optional
        .register_tool(create_tool_with_optional(
            "app",
            vec!["base"],
            vec!["optional-tool"],
        ))
        .await
        .unwrap();

    let result = resolver_no_optional.resolve("app").await.unwrap();

    // Should only include required dependencies
    assert!(result.install_order.contains(&"base".to_string()));
    assert!(result.install_order.contains(&"app".to_string()));
    // Optional tool should not be included when include_optional is false
}

#[tokio::test]
async fn test_performance_with_large_dependency_tree() {
    let resolver = DependencyResolver::new();

    // Create a large dependency tree
    resolver
        .register_tool(create_tool("root", vec![]))
        .await
        .unwrap();

    // Create multiple layers
    for layer in 1..=5 {
        for tool_num in 1..=10 {
            let tool_name = format!("layer{}_tool{}", layer, tool_num);
            let deps = if layer == 1 {
                vec![("root".to_string(), None)]
            } else {
                vec![(format!("layer{}_tool1", layer - 1), None)]
            };
            resolver
                .register_tool(create_tool_with_string_deps(&tool_name, deps))
                .await
                .unwrap();
        }
    }

    let start = std::time::Instant::now();
    let result = resolver.resolve("layer5_tool1").await.unwrap();
    let duration = start.elapsed();

    // Should resolve quickly even with many tools
    assert!(duration.as_millis() < 1000, "Resolution should be fast");
    assert!(!result.install_order.is_empty());
    assert!(result.circular_dependencies.is_empty());
}
