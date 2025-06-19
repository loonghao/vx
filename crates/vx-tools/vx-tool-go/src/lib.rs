//! Go tool support for vx

use vx_plugin::VxPlugin;

mod config;
mod plugin;
mod tool;

pub use plugin::GoPlugin;
pub use tool::GoTool;

/// Create a new Go plugin instance
pub fn create_plugin() -> Box<dyn VxPlugin> {
    Box::new(GoPlugin)
}
