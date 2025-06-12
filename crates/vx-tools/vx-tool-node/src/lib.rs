//! Node.js tool support for vx

use vx_core::VxPlugin;

mod node_plugin;
mod node_tool;

pub use node_plugin::NodePlugin;
pub use node_tool::{NodeTool, NpmTool, NpxTool};

/// Create a new Node.js plugin instance
pub fn create_plugin() -> Box<dyn VxPlugin> {
    Box::new(NodePlugin)
}
