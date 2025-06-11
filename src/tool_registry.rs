// Tool registry - simple HashMap-based tool management
// Replaces the complex PluginRegistry with a straightforward approach

use crate::tool::{Tool, ToolInfo};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Simple tool registry
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new tool registry with all built-in tools
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };

        // Register all built-in tools
        registry.register_builtin_tools();

        registry
    }

    /// Register a tool
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|tool| tool.as_ref())
    }

    /// Check if a tool is registered
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get all registered tool names
    pub fn tool_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Get information about all tools
    pub fn get_all_tool_info(&self) -> Vec<ToolInfo> {
        self.tools
            .values()
            .map(|tool| ToolInfo::from_tool(tool.as_ref()))
            .collect()
    }

    /// Get information about a specific tool
    pub fn get_tool_info(&self, name: &str) -> Result<ToolInfo> {
        let tool = self
            .get(name)
            .ok_or_else(|| anyhow!("Tool '{}' not found", name))?;
        Ok(ToolInfo::from_tool(tool))
    }

    /// Register all built-in tools
    fn register_builtin_tools(&mut self) {
        // Import tool implementations
        use crate::tools::rust::{CargoTool, RustcTool};
        use crate::tools::{GoTool, NodeTool, UvTool};

        // Register tools
        self.register(Box::new(UvTool::new()));
        self.register(Box::new(NodeTool::new()));
        self.register(Box::new(GoTool::new()));
        self.register(Box::new(CargoTool::new()));
        self.register(Box::new(RustcTool::new()));
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ToolRegistry::new();
        assert!(!registry.tool_names().is_empty());
    }

    #[test]
    fn test_tool_registration() {
        let registry = ToolRegistry::new();

        // Should have built-in tools
        assert!(registry.has_tool("uv"));
        assert!(registry.has_tool("node"));
        assert!(registry.has_tool("go"));
        assert!(registry.has_tool("cargo"));
        assert!(registry.has_tool("rustc"));
    }

    #[test]
    fn test_tool_retrieval() {
        let registry = ToolRegistry::new();

        let uv_tool = registry.get("uv");
        assert!(uv_tool.is_some());
        assert_eq!(uv_tool.unwrap().name(), "uv");

        let nonexistent = registry.get("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_tool_info() {
        let registry = ToolRegistry::new();

        let info = registry.get_tool_info("uv");
        assert!(info.is_ok());

        let info = info.unwrap();
        assert_eq!(info.name, "uv");
        assert!(!info.description.is_empty());
    }
}
