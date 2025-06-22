//! Python tool support for vx using Python Build Standalone

use vx_plugin::VxPlugin;

mod config;
mod python_plugin;
mod python_tool;

pub use config::PythonUrlBuilder;
pub use python_plugin::PythonPlugin;
pub use python_tool::{PipTool, PipxTool, PythonTool};

/// Create a new Python plugin instance
pub fn create_plugin() -> Box<dyn VxPlugin> {
    Box::new(PythonPlugin)
}
