//! Node.js plugin implementation

use vx_core::{VxPlugin, VxTool};
use crate::node_tool::{NodeTool, NpmTool, NpxTool};

/// Node.js plugin that manages Node.js-related tools
#[derive(Debug)]
pub struct NodePlugin;

impl NodePlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl VxPlugin for NodePlugin {
    fn name(&self) -> &str {
        "node"
    }

    fn description(&self) -> &str {
        "Node.js JavaScript runtime and package management tools"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn tools(&self) -> Vec<Box<dyn VxTool>> {
        vec![
            Box::new(NodeTool::new()),
            Box::new(NpmTool::new()),
            Box::new(NpxTool::new()),
        ]
    }

    fn supports_tool(&self, tool_name: &str) -> bool {
        matches!(tool_name, "node" | "npm" | "npx")
    }
}

impl Default for NodePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_node_plugin() {
        let plugin = NodePlugin::default();
        
        assert_eq!(plugin.name(), "node");
        assert_eq!(plugin.description(), "Node.js support for vx");
        assert!(plugin.supports_tool("node"));
        assert!(plugin.supports_tool("nodejs"));
        assert!(!plugin.supports_tool("python"));
        
        let tools = plugin.tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name(), "node");
    }
}
