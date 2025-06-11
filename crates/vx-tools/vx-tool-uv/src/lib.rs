//! UV tool support for vx

use vx_core::Plugin;

mod uv_tool;
mod uv_plugin;

pub use uv_tool::{UvCommand, UvxTool};
pub use uv_plugin::UvPlugin;

/// Create a new UV plugin instance
pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(UvPlugin::default())
}
