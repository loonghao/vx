//! Rust tool support for vx

use vx_plugin::VxPlugin;

mod config;
mod rust_plugin;
mod rust_tool;

pub use config::RustUrlBuilder;
pub use rust_plugin::RustPlugin;
pub use rust_tool::CargoTool;

/// Create a new Rust plugin instance
pub fn create_plugin() -> Box<dyn VxPlugin> {
    Box::new(RustPlugin)
}
