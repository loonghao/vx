//! Node.js tool support for vx
//!
//! This bundle provides:
//! - Node.js runtime (`node`)
//! - NPM package manager (`npm`) - both as tool and package manager
//! - NPX package runner (`npx`)

use vx_plugin::ToolBundle;

mod config;
mod npm_pm;
mod plugin;
mod tool;

pub use config::{
    create_install_config, get_install_methods, get_manual_instructions, supports_auto_install,
    Config, NodeUrlBuilder,
};
pub use npm_pm::NpmPackageManager;
pub use plugin::NodePlugin;
pub use tool::{NodeTool, NpmTool, NpxTool};

/// Create a new Node.js bundle instance
pub fn create_plugin() -> Box<dyn ToolBundle> {
    Box::new(NodePlugin)
}
