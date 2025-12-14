//! Go tool support for vx

use vx_plugin::ToolBundle;

mod config;
mod plugin;
mod tool;

pub use plugin::GoPlugin;
pub use tool::GoTool;

/// Create a new Go plugin instance
pub fn create_plugin() -> Box<dyn ToolBundle> {
    Box::new(GoPlugin)
}
