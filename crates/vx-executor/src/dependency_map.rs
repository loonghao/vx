//! Dependency mapping for tools
//!
//! This module provides a comprehensive mapping of tools to their dependencies,
//! supporting various ecosystems (Node.js, Python, Rust, Go, etc.)

use crate::tool_spec::{Ecosystem, RuntimeDependency, ToolSpec};
use std::collections::HashMap;

/// A registry of tool specifications and their dependencies
#[derive(Debug, Default)]
pub struct DependencyMap {
    /// Map of tool name to specification
    tools: HashMap<String, ToolSpec>,
    /// Map of alias to primary tool name
    aliases: HashMap<String, String>,
}

impl DependencyMap {
    /// Create a new dependency map with built-in tool definitions
    pub fn new() -> Self {
        let mut map = Self::default();
        map.register_builtin_tools();
        map
    }

    /// Create an empty dependency map (for testing)
    pub fn empty() -> Self {
        Self::default()
    }

    /// Register a tool specification
    pub fn register(&mut self, spec: ToolSpec) {
        // Register aliases
        for alias in &spec.aliases {
            self.aliases.insert(alias.clone(), spec.name.clone());
        }
        self.tools.insert(spec.name.clone(), spec);
    }

    /// Get a tool specification by name or alias
    pub fn get(&self, name: &str) -> Option<&ToolSpec> {
        // First try direct lookup
        if let Some(spec) = self.tools.get(name) {
            return Some(spec);
        }
        // Then try alias lookup
        if let Some(primary) = self.aliases.get(name) {
            return self.tools.get(primary);
        }
        None
    }

    /// Check if a tool is known
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name) || self.aliases.contains_key(name)
    }

    /// Get all tool names
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// Get tools by ecosystem
    pub fn by_ecosystem(&self, ecosystem: Ecosystem) -> Vec<&ToolSpec> {
        self.tools
            .values()
            .filter(|spec| spec.ecosystem == ecosystem)
            .collect()
    }

    /// Resolve the primary tool name from a name or alias
    pub fn resolve_name<'a>(&'a self, name: &'a str) -> Option<&'a str> {
        if self.tools.contains_key(name) {
            Some(name)
        } else {
            self.aliases.get(name).map(|s| s.as_str())
        }
    }

    /// Register all built-in tool definitions
    fn register_builtin_tools(&mut self) {
        // ============ Node.js Ecosystem ============

        // Node.js runtime
        self.register(
            ToolSpec::new("node", "Node.js JavaScript runtime")
                .with_alias("nodejs")
                .with_ecosystem(Ecosystem::Node)
                .with_priority(100), // High priority - base runtime
        );

        // npm - bundled with Node.js
        self.register(
            ToolSpec::new("npm", "Node.js package manager")
                .with_ecosystem(Ecosystem::Node)
                .with_dependency(
                    RuntimeDependency::required("node", "npm is bundled with Node.js")
                        .provided_by("node"),
                ),
        );

        // npx - bundled with Node.js
        self.register(
            ToolSpec::new("npx", "Node.js package runner")
                .with_ecosystem(Ecosystem::Node)
                .with_dependency(
                    RuntimeDependency::required("node", "npx is bundled with Node.js")
                        .provided_by("node"),
                ),
        );

        // yarn - requires Node.js
        self.register(
            ToolSpec::new("yarn", "Fast, reliable, and secure dependency management")
                .with_ecosystem(Ecosystem::Node)
                .with_dependency(RuntimeDependency::required(
                    "node",
                    "yarn requires Node.js runtime",
                )),
        );

        // pnpm - requires Node.js
        self.register(
            ToolSpec::new("pnpm", "Fast, disk space efficient package manager")
                .with_ecosystem(Ecosystem::Node)
                .with_dependency(RuntimeDependency::required(
                    "node",
                    "pnpm requires Node.js runtime",
                )),
        );

        // bun - standalone runtime (no dependencies)
        self.register(
            ToolSpec::new("bun", "Incredibly fast JavaScript runtime and toolkit")
                .with_alias("bunx")
                .with_ecosystem(Ecosystem::Node)
                .with_priority(90),
        );

        // ============ Python Ecosystem ============

        // uv - standalone Python package manager
        self.register(
            ToolSpec::new(
                "uv",
                "An extremely fast Python package installer and resolver",
            )
            .with_ecosystem(Ecosystem::Python)
            .with_priority(100), // Standalone, no dependencies
        );

        // uvx - uv tool runner (alias for "uv tool run")
        self.register(
            ToolSpec::new("uvx", "Python application runner")
                .with_ecosystem(Ecosystem::Python)
                .with_executable("uv")
                .with_command_prefix(vec!["tool", "run"])
                .with_dependency(
                    RuntimeDependency::required("uv", "uvx is part of uv").provided_by("uv"),
                ),
        );

        // pip - requires Python (but uv can replace it)
        self.register(
            ToolSpec::new("pip", "Python package installer")
                .with_alias("pip3")
                .with_ecosystem(Ecosystem::Python)
                .with_dependency(RuntimeDependency::required(
                    "python",
                    "pip requires Python runtime",
                )),
        );

        // pipx - Python application runner
        self.register(
            ToolSpec::new(
                "pipx",
                "Install and run Python applications in isolated environments",
            )
            .with_ecosystem(Ecosystem::Python)
            .with_dependency(RuntimeDependency::required(
                "python",
                "pipx requires Python runtime",
            )),
        );

        // ============ Rust Ecosystem ============

        // rustup - Rust toolchain installer
        self.register(
            ToolSpec::new("rustup", "The Rust toolchain installer")
                .with_ecosystem(Ecosystem::Rust)
                .with_priority(100),
        );

        // cargo - Rust package manager
        self.register(
            ToolSpec::new("cargo", "Rust package manager and build tool")
                .with_ecosystem(Ecosystem::Rust)
                .with_dependency(
                    RuntimeDependency::required("rustup", "cargo is installed via rustup")
                        .provided_by("rustup"),
                ),
        );

        // rustc - Rust compiler
        self.register(
            ToolSpec::new("rustc", "The Rust compiler")
                .with_ecosystem(Ecosystem::Rust)
                .with_dependency(
                    RuntimeDependency::required("rustup", "rustc is installed via rustup")
                        .provided_by("rustup"),
                ),
        );

        // cargo-binstall - Binary installation for cargo
        self.register(
            ToolSpec::new("cargo-binstall", "Binary installation for Rust projects")
                .with_ecosystem(Ecosystem::Rust)
                .with_dependency(RuntimeDependency::required(
                    "cargo",
                    "cargo-binstall requires cargo",
                )),
        );

        // ============ Go Ecosystem ============

        // go - Go programming language
        self.register(
            ToolSpec::new("go", "The Go programming language")
                .with_alias("golang")
                .with_ecosystem(Ecosystem::Go)
                .with_priority(100),
        );

        // ============ Generic Tools ============

        // git - Version control
        self.register(ToolSpec::new("git", "Distributed version control system"));

        // make - Build automation
        self.register(ToolSpec::new("make", "Build automation tool").with_alias("gmake"));

        // cmake - Cross-platform build system
        self.register(ToolSpec::new(
            "cmake",
            "Cross-platform build system generator",
        ));

        // docker - Container platform
        self.register(ToolSpec::new("docker", "Container platform"));

        // kubectl - Kubernetes CLI
        self.register(ToolSpec::new("kubectl", "Kubernetes command-line tool").with_alias("k8s"));
    }

    /// Get the installation order for a tool and its dependencies
    ///
    /// Returns a topologically sorted list of tools to install,
    /// with dependencies coming before dependents.
    pub fn get_install_order<'a>(&'a self, tool_name: &'a str) -> Vec<&'a str> {
        let mut order = Vec::new();
        let mut visited = std::collections::HashSet::new();

        self.visit_dependencies(tool_name, &mut order, &mut visited);
        order
    }

    /// Recursively visit dependencies (DFS)
    fn visit_dependencies<'a>(
        &'a self,
        tool_name: &'a str,
        order: &mut Vec<&'a str>,
        visited: &mut std::collections::HashSet<&'a str>,
    ) {
        if visited.contains(tool_name) {
            return;
        }
        visited.insert(tool_name);

        if let Some(spec) = self.get(tool_name) {
            // Visit dependencies first
            for dep in &spec.dependencies {
                if dep.required {
                    // Use the provider if specified, otherwise the dependency name
                    let dep_name = dep.provided_by.as_deref().unwrap_or(&dep.tool_name);
                    self.visit_dependencies(dep_name, order, visited);
                }
            }
            // Then add this tool
            order.push(&spec.name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_map_creation() {
        let map = DependencyMap::new();

        assert!(map.contains("node"));
        assert!(map.contains("nodejs")); // alias
        assert!(map.contains("npm"));
        assert!(map.contains("uv"));
        assert!(map.contains("cargo"));
    }

    #[test]
    fn test_alias_resolution() {
        let map = DependencyMap::new();

        assert_eq!(map.resolve_name("nodejs"), Some("node"));
        assert_eq!(map.resolve_name("node"), Some("node"));
        assert_eq!(map.resolve_name("pip3"), Some("pip"));
    }

    #[test]
    fn test_dependency_lookup() {
        let map = DependencyMap::new();

        let npm = map.get("npm").unwrap();
        assert_eq!(npm.required_dependencies().len(), 1);
        assert_eq!(npm.required_dependencies()[0].tool_name, "node");
    }

    #[test]
    fn test_ecosystem_filtering() {
        let map = DependencyMap::new();

        let node_tools = map.by_ecosystem(Ecosystem::Node);
        assert!(node_tools.iter().any(|t| t.name == "node"));
        assert!(node_tools.iter().any(|t| t.name == "npm"));
        assert!(node_tools.iter().any(|t| t.name == "yarn"));
    }

    #[test]
    fn test_install_order() {
        let map = DependencyMap::new();

        // npm depends on node, so node should come first
        let order = map.get_install_order("npm");
        let node_pos = order.iter().position(|&t| t == "node");
        let npm_pos = order.iter().position(|&t| t == "npm");

        assert!(node_pos.is_some());
        assert!(npm_pos.is_some());
        assert!(node_pos.unwrap() < npm_pos.unwrap());
    }

    #[test]
    fn test_uvx_command_prefix() {
        let map = DependencyMap::new();

        let uvx = map.get("uvx").unwrap();
        assert_eq!(uvx.get_executable(), "uv");
        assert_eq!(uvx.command_prefix, vec!["tool", "run"]);
    }

    #[test]
    fn test_standalone_tools() {
        let map = DependencyMap::new();

        // uv has no dependencies
        let uv = map.get("uv").unwrap();
        assert!(uv.required_dependencies().is_empty());

        // bun has no dependencies
        let bun = map.get("bun").unwrap();
        assert!(bun.required_dependencies().is_empty());
    }
}
