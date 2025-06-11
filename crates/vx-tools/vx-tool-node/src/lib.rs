//! Node.js tool support for vx

use vx_core::VxPlugin;

mod node_tool;
mod node_plugin;

pub use node_tool::{NodeTool, NpmTool, NpxTool};
pub use node_plugin::NodePlugin;

/// Create a new Node.js plugin instance
pub fn create_plugin() -> Box<dyn VxPlugin> {
    Box::new(NodePlugin::default())
}
