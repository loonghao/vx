//! Node.js tool support for vx

use vx_plugin::VxPlugin;

mod config;
mod plugin;
mod tool;

pub use config::{
    create_install_config, get_install_methods, get_manual_instructions, supports_auto_install,
    Config, NodeUrlBuilder,
};
pub use plugin::NodePlugin;
pub use tool::{NodeTool, NpmTool, NpxTool};

/// Create a new Node.js plugin instance
pub fn create_plugin() -> Box<dyn VxPlugin> {
    Box::new(NodePlugin)
}
