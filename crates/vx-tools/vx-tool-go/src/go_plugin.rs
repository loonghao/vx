//! Go plugin implementation

use vx_core::{Plugin, Tool};
use crate::go_tool::GoTool;
use std::collections::HashMap;

/// Go plugin that provides Go tool support
pub struct GoPlugin;

impl GoPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GoPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for GoPlugin {
    fn name(&self) -> &str {
        "go"
    }

    fn description(&self) -> &str {
        "Go programming language support for vx"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn tools(&self) -> Vec<Box<dyn vx_core::AsyncTool>> {
        // For now, return empty vec since we need to implement AsyncTool wrapper
        vec![]
    }

    fn supports_tool(&self, tool_name: &str) -> bool {
        tool_name == "go" || tool_name == "golang"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_plugin() {
        let plugin = GoPlugin::default();

        assert_eq!(plugin.name(), "go");
        assert_eq!(plugin.description(), "Go programming language support for vx");
        assert!(plugin.supports_tool("go"));
        assert!(plugin.supports_tool("golang"));
        assert!(!plugin.supports_tool("python"));
    }
}
