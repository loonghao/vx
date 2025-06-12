//! UV tool support for vx

use vx_core::VxPlugin;

mod uv_plugin;
mod uv_tool;

pub use uv_plugin::UvPlugin;
pub use uv_tool::{UvCommand, UvxTool};

/// Create a new UV plugin instance
pub fn create_plugin() -> Box<dyn VxPlugin> {
    Box::new(UvPlugin)
}
