//! Go tool support for vx

use vx_core::VxPlugin;

mod go_plugin;
mod go_tool;

pub use go_plugin::GoPlugin;
pub use go_tool::GoTool;

/// Create a new Go plugin instance
pub fn create_plugin() -> Box<dyn VxPlugin> {
    Box::new(GoPlugin)
}
