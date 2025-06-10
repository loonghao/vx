pub mod go_plugin;
pub mod node_plugin;
pub mod rust_plugin;
pub mod uv_plugin;

use crate::plugin::PluginRegistry;
use anyhow::Result;

/// Initialize and register all built-in plugins
pub fn register_builtin_plugins(registry: &mut PluginRegistry) -> Result<()> {
    // Register Go plugin
    let go_plugin = Box::new(go_plugin::GoPlugin::new());
    registry.register_plugin(go_plugin)?;

    // Register Node.js plugin
    let node_plugin = Box::new(node_plugin::NodePlugin::new());
    registry.register_plugin(node_plugin)?;

    // Register Rust plugin
    let rust_plugin = Box::new(rust_plugin::RustPlugin::new());
    registry.register_plugin(rust_plugin)?;

    // Register UV plugin
    let uv_plugin = Box::new(uv_plugin::UvPlugin::new());
    registry.register_plugin(uv_plugin)?;

    crate::ui::UI::success(&format!("Registered {} built-in plugins", 4));
    Ok(())
}
