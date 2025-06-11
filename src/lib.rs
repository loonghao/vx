pub mod cli;
pub mod config_figment;
pub mod executor;
pub mod install_configs;
pub mod installer;
pub mod package_ecosystem;
pub mod package_manager;
pub mod package_managers_impl;
pub mod tool;
pub mod tool_manager;
pub mod tool_registry;
pub mod tools;
pub mod tracing_setup;
pub mod ui;
pub mod universal_package_router;
pub mod version;

// Re-export tracing setup function
pub use tracing_setup::init_tracing;

// Note: Macros are automatically available at crate root due to #[macro_export]

pub use anyhow::Result;
