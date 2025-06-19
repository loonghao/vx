//! Go plugin implementation

use crate::tool::GoTool;
use vx_plugin::{VxPlugin, VxTool};

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

#[async_trait::async_trait]
impl VxPlugin for GoPlugin {
    fn name(&self) -> &str {
        "go"
    }

    fn description(&self) -> &str {
        "Go programming language support for vx"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn tools(&self) -> Vec<Box<dyn VxTool>> {
        vec![Box::new(GoTool::new())]
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
        let plugin = GoPlugin;

        assert_eq!(plugin.name(), "go");
        assert_eq!(
            plugin.description(),
            "Go programming language support for vx"
        );
        assert!(plugin.supports_tool("go"));
        assert!(plugin.supports_tool("golang"));
        assert!(!plugin.supports_tool("python"));
    }
}
