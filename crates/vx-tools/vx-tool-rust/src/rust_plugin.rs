//! Rust plugin implementation

use crate::rust_tool::CargoTool;
use vx_core::{VxPlugin, VxTool};

/// Rust plugin that provides Rust toolchain tools
#[derive(Debug, Default)]
pub struct RustPlugin;

impl RustPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl VxPlugin for RustPlugin {
    fn name(&self) -> &str {
        "rust"
    }

    fn description(&self) -> &str {
        "Rust toolchain support for vx"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn tools(&self) -> Vec<Box<dyn VxTool>> {
        vec![Box::new(CargoTool::new())]
    }

    fn supports_tool(&self, tool_name: &str) -> bool {
        matches!(tool_name, "cargo")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_plugin_creation() {
        let plugin = RustPlugin::new();
        assert_eq!(plugin.name(), "rust");
        assert!(!plugin.description().is_empty());
    }

    #[test]
    fn test_rust_plugin_tools() {
        let plugin = RustPlugin::new();
        let tools = plugin.tools();

        assert_eq!(tools.len(), 1);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        assert!(tool_names.contains(&"cargo"));
    }

    #[test]
    fn test_rust_plugin_supports_tool() {
        let plugin = RustPlugin::new();

        assert!(plugin.supports_tool("cargo"));
        assert!(!plugin.supports_tool("rustc"));
        assert!(!plugin.supports_tool("rustup"));
        assert!(!plugin.supports_tool("rustdoc"));
        assert!(!plugin.supports_tool("rustfmt"));
        assert!(!plugin.supports_tool("clippy"));
        assert!(!plugin.supports_tool("nonexistent"));
    }

    #[test]
    fn test_rust_plugin_version() {
        let plugin = RustPlugin::new();
        assert!(!plugin.version().is_empty());
    }
}
