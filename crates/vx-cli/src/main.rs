use vx_cli::{setup_tracing, VxCli};
use vx_core::PluginRegistry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup tracing
    setup_tracing();

    // Create plugin registry with all available plugins
    let mut registry = PluginRegistry::new();

    // Register Node.js plugin
    let _ = registry.register(Box::new(vx_tool_node::NodePlugin::new()));

    // Register Go plugin
    let _ = registry.register(Box::new(vx_tool_go::GoPlugin::new()));

    // Register Rust plugin
    let _ = registry.register(Box::new(vx_tool_rust::RustPlugin::new()));

    // Register UV plugin
    let _ = registry.register(Box::new(vx_tool_uv::UvPlugin::new()));

    // Create and run CLI
    let cli = VxCli::new(registry);
    cli.run().await
}
