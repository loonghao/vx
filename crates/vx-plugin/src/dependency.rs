//! Tool dependency resolution system
//!
//! This module handles tool dependencies and aliases, allowing automatic
//! installation and execution of dependent tools.

use std::collections::HashMap;

/// Tool dependency resolver
pub struct DependencyResolver {
    /// Maps dependent tools to their parent tools
    /// e.g., "npx" -> "node", "uvx" -> "uv"
    dependencies: HashMap<String, String>,

    /// Maps tools to their executable paths within the installation
    /// e.g., "npx" -> "bin/npx.exe" (relative to node installation)
    executable_paths: HashMap<String, String>,
}

impl DependencyResolver {
    /// Create a new dependency resolver with default mappings
    pub fn new() -> Self {
        let mut dependencies = HashMap::new();
        let mut executable_paths = HashMap::new();

        // Node.js ecosystem
        dependencies.insert("npx".to_string(), "node".to_string());
        dependencies.insert("npm".to_string(), "node".to_string());

        // UV ecosystem
        dependencies.insert("uvx".to_string(), "uv".to_string());

        // Rust ecosystem
        dependencies.insert("cargo".to_string(), "rust".to_string());
        dependencies.insert("rustc".to_string(), "rust".to_string());
        dependencies.insert("rustup".to_string(), "rust".to_string());

        // Executable paths within installations
        // Note: npx and npm have their own executables in Node.js installations
        #[cfg(windows)]
        {
            executable_paths.insert("npx".to_string(), "npx.cmd".to_string());
            executable_paths.insert("npm".to_string(), "npm.cmd".to_string());
        }
        #[cfg(not(windows))]
        {
            executable_paths.insert("npx".to_string(), "npx".to_string());
            executable_paths.insert("npm".to_string(), "npm".to_string());
        }

        // uvx is actually a subcommand of uv, not a separate executable
        // We'll handle this specially in the execution logic
        #[cfg(windows)]
        executable_paths.insert("uvx".to_string(), "uv.bat".to_string());
        #[cfg(not(windows))]
        executable_paths.insert("uvx".to_string(), "uv".to_string());
        executable_paths.insert("cargo".to_string(), "bin/cargo.exe".to_string());
        executable_paths.insert("rustc".to_string(), "bin/rustc.exe".to_string());

        Self {
            dependencies,
            executable_paths,
        }
    }

    /// Check if a tool is a dependent tool (has a parent)
    pub fn is_dependent_tool(&self, tool_name: &str) -> bool {
        self.dependencies.contains_key(tool_name)
    }

    /// Get the parent tool for a dependent tool
    pub fn get_parent_tool(&self, tool_name: &str) -> Option<&str> {
        self.dependencies.get(tool_name).map(|s| s.as_str())
    }

    /// Get the executable path for a tool within its parent installation
    pub fn get_executable_path(&self, tool_name: &str) -> Option<&str> {
        self.executable_paths.get(tool_name).map(|s| s.as_str())
    }

    /// Get all dependent tools for a parent tool
    pub fn get_dependent_tools(&self, parent_tool: &str) -> Vec<&str> {
        self.dependencies
            .iter()
            .filter_map(|(dependent, parent)| {
                if parent == parent_tool {
                    Some(dependent.as_str())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Resolve tool dependency chain
    /// Returns (actual_tool_to_install, dependent_tool_name)
    pub fn resolve_dependency(&self, tool_name: &str) -> (String, Option<String>) {
        if let Some(parent) = self.get_parent_tool(tool_name) {
            (parent.to_string(), Some(tool_name.to_string()))
        } else {
            (tool_name.to_string(), None)
        }
    }

    /// Check if uninstalling a tool would affect dependent tools
    pub fn check_uninstall_impact(&self, tool_name: &str) -> Vec<String> {
        self.get_dependent_tools(tool_name)
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_resolution() {
        let resolver = DependencyResolver::new();

        // Test npx -> node dependency
        assert!(resolver.is_dependent_tool("npx"));
        assert_eq!(resolver.get_parent_tool("npx"), Some("node"));

        // Test uvx -> uv dependency
        assert!(resolver.is_dependent_tool("uvx"));
        assert_eq!(resolver.get_parent_tool("uvx"), Some("uv"));

        // Test independent tool
        assert!(!resolver.is_dependent_tool("go"));
        assert_eq!(resolver.get_parent_tool("go"), None);
    }

    #[test]
    fn test_dependency_chain_resolution() {
        let resolver = DependencyResolver::new();

        // Test npx resolution
        let (parent, dependent) = resolver.resolve_dependency("npx");
        assert_eq!(parent, "node");
        assert_eq!(dependent, Some("npx".to_string()));

        // Test independent tool resolution
        let (parent, dependent) = resolver.resolve_dependency("go");
        assert_eq!(parent, "go");
        assert_eq!(dependent, None);
    }

    #[test]
    fn test_uninstall_impact() {
        let resolver = DependencyResolver::new();

        // Test node uninstall impact
        let impact = resolver.check_uninstall_impact("node");
        assert!(impact.contains(&"npx".to_string()));
        assert!(impact.contains(&"npm".to_string()));

        // Test independent tool uninstall
        let impact = resolver.check_uninstall_impact("go");
        assert!(impact.is_empty());
    }

    #[test]
    fn test_executable_paths() {
        let resolver = DependencyResolver::new();

        #[cfg(windows)]
        {
            assert_eq!(resolver.get_executable_path("npx"), Some("npx.cmd"));
            assert_eq!(resolver.get_executable_path("npm"), Some("npm.cmd"));
        }

        #[cfg(not(windows))]
        {
            assert_eq!(resolver.get_executable_path("npx"), Some("npx"));
            assert_eq!(resolver.get_executable_path("npm"), Some("npm"));
        }

        assert_eq!(resolver.get_executable_path("cargo"), Some("bin/cargo.exe"));
        assert_eq!(resolver.get_executable_path("unknown"), None);
    }
}
