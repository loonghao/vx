//! Dynamic Tool Executor
//!
//! This crate provides a universal tool command forwarding system that:
//! - Detects tool dependencies automatically
//! - Auto-installs missing tools when needed
//! - Forwards commands to the appropriate tool executable
//!
//! # Architecture
//!
//! The executor uses a layered approach:
//! 1. **Tool Registry** - Maps tool names and aliases to their specifications
//! 2. **Dependency Resolver** - Determines what needs to be installed
//! 3. **Auto Installer** - Downloads and installs missing dependencies
//! 4. **Command Forwarder** - Executes the actual tool command
//!
//! # Example
//!
//! ```rust,no_run
//! use vx_executor::{DynamicExecutor, ExecutorConfig};
//!
//! async fn example() -> anyhow::Result<()> {
//!     let executor = DynamicExecutor::new(ExecutorConfig::default())?;
//!
//!     // Execute: vx npm install express
//!     // This will:
//!     // 1. Detect npm requires node
//!     // 2. Check if node is installed
//!     // 3. Auto-install node if missing
//!     // 4. Forward "install express" to npm
//!     let exit_code = executor.execute("npm", &["install".into(), "express".into()]).await?;
//!     Ok(())
//! }
//! ```

mod config;
mod dependency_map;
mod executor;
mod resolver;
mod tool_spec;

pub use config::ExecutorConfig;
pub use dependency_map::DependencyMap;
pub use executor::{execute_system_tool, DynamicExecutor};
pub use resolver::{ResolutionResult, ToolResolver, ToolStatus};
pub use tool_spec::{Ecosystem, RuntimeDependency, ToolSpec};

/// Result type for executor operations
pub type Result<T> = anyhow::Result<T>;
