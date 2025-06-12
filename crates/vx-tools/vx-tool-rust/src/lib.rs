//! Rust tool support for vx

use vx_core::VxPlugin;

mod rust_plugin;
mod rust_tool;

pub use rust_plugin::RustPlugin;
pub use rust_tool::{CargoTool, ClippyTool, RustcTool, RustdocTool, RustfmtTool, RustupTool};

/// Create a new Rust plugin instance
pub fn create_plugin() -> Box<dyn VxPlugin> {
    Box::new(RustPlugin)
}
