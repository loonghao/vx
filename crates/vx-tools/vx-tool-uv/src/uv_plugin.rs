//! UV plugin implementation

use anyhow::Result;
use std::collections::HashMap;
use vx_core::{Tool, Plugin};

use crate::uv_tool::{UvCommand, UvxTool};

/// UV plugin that manages UV-related tools
#[derive(Debug)]
pub struct UvPlugin {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl UvPlugin {
    pub fn new() -> Self {
        let mut tools: HashMap<String, Box<dyn Tool>> = HashMap::new();
        
        // Register UV tools
        tools.insert("uv".to_string(), Box::new(UvCommand::new()));
        tools.insert("uvx".to_string(), Box::new(UvxTool::new()));
        
        Self { tools }
    }
}

impl Plugin for UvPlugin {
    fn name(&self) -> &str {
        "uv"
    }

    fn description(&self) -> &str {
        "UV Python package management tools"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn list_tools(&self) -> Result<Vec<String>> {
        Ok(self.tools.keys().cloned().collect())
    }

    fn get_tool(&self, name: &str) -> Result<Option<&dyn Tool>> {
        Ok(self.tools.get(name).map(|tool| tool.as_ref()))
    }

    fn supports_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    fn get_supported_platforms(&self) -> Vec<String> {
        vec![
            "windows".to_string(),
            "macos".to_string(),
            "linux".to_string(),
        ]
    }

    fn get_homepage(&self) -> Option<String> {
        Some("https://docs.astral.sh/uv/".to_string())
    }

    fn get_repository(&self) -> Option<String> {
        Some("https://github.com/astral-sh/uv".to_string())
    }

    fn get_license(&self) -> Option<String> {
        Some("MIT OR Apache-2.0".to_string())
    }

    fn get_keywords(&self) -> Vec<String> {
        vec![
            "python".to_string(),
            "package-manager".to_string(),
            "uv".to_string(),
            "fast".to_string(),
        ]
    }

    fn get_categories(&self) -> Vec<String> {
        vec!["development-tools".to_string()]
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
        let tools = plugin.list_tools().unwrap();
        
        assert!(tools.contains(&"uv".to_string()));
        assert!(tools.contains(&"uvx".to_string()));
        assert_eq!(tools.len(), 2);
    }

    #[test]
    fn test_uv_plugin_get_tool() {
        let plugin = UvPlugin::new();
        
        assert!(plugin.get_tool("uv").unwrap().is_some());
        assert!(plugin.get_tool("uvx").unwrap().is_some());
        assert!(plugin.get_tool("nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_uv_plugin_supports_tool() {
        let plugin = UvPlugin::new();
        
        assert!(plugin.supports_tool("uv"));
        assert!(plugin.supports_tool("uvx"));
        assert!(!plugin.supports_tool("nonexistent"));
    }

    #[test]
    fn test_uv_plugin_metadata() {
        let plugin = UvPlugin::new();
        
        assert!(plugin.get_homepage().is_some());
        assert!(plugin.get_repository().is_some());
        assert!(plugin.get_license().is_some());
        assert!(!plugin.get_keywords().is_empty());
        assert!(!plugin.get_categories().is_empty());
    }
}
