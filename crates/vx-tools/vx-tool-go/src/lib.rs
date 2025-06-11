//! Go tool support for vx

use vx_core::Plugin;

mod go_tool;
mod go_plugin;

pub use go_tool::GoTool;
pub use go_plugin::GoPlugin;

/// Create a new Go plugin instance
pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(GoPlugin::default())
}
