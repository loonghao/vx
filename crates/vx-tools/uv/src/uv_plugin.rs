//! UV plugin implementation

use crate::uv_tool::{UvCommand, UvxTool};
use vx_plugin::{ToolBundle, VxTool};

/// UV plugin that manages UV-related tools
#[derive(Debug)]
pub struct UvPlugin;

impl UvPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl ToolBundle for UvPlugin {
    fn name(&self) -> &str {
        "uv"
    }

    fn description(&self) -> &str {
        "UV Python package management tools"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn tools(&self) -> Vec<Box<dyn VxTool>> {
        vec![Box::new(UvCommand::new()), Box::new(UvxTool::new())]
    }

    fn supports_tool(&self, tool_name: &str) -> bool {
        matches!(tool_name, "uv" | "uvx")
    }
}

impl Default for UvPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uv_plugin_creation() {
        let plugin = UvPlugin::new();
        assert_eq!(plugin.name(), "uv");
        assert!(!plugin.description().is_empty());
        assert!(!plugin.version().is_empty());
    }

    #[test]
    fn test_uv_plugin_tools() {
        let plugin = UvPlugin::new();
        let tools = plugin.tools();

        assert_eq!(tools.len(), 2);
        assert!(tools.iter().any(|t| t.name() == "uv"));
        assert!(tools.iter().any(|t| t.name() == "uvx"));
    }

    #[test]
    fn test_uv_plugin_supports_tool() {
        let plugin = UvPlugin::new();

        assert!(plugin.supports_tool("uv"));
        assert!(plugin.supports_tool("uvx"));
        assert!(!plugin.supports_tool("nonexistent"));
    }
}
