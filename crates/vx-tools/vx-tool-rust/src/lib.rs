//! Rust tool support for vx

use vx_core::Plugin;

mod rust_tool;
mod rust_plugin;

pub use rust_tool::{CargoTool, RustcTool, RustupTool, RustdocTool, RustfmtTool, ClippyTool};
pub use rust_plugin::RustPlugin;

/// Create a new Rust plugin instance
pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(RustPlugin::default())
}
