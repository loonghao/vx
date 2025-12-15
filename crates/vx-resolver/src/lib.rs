//! Runtime Resolver
//!
//! This crate provides a universal runtime command forwarding system that:
//! - Detects runtime dependencies automatically
//! - Auto-installs missing runtimes when needed
//! - Forwards commands to the appropriate runtime executable
//!
//! # Architecture
//!
//! The resolver uses a layered approach:
//! 1. **Runtime Registry** - Maps runtime names and aliases to their specifications
//! 2. **Dependency Resolver** - Determines what needs to be installed
//! 3. **Auto Installer** - Downloads and installs missing dependencies
//! 4. **Command Forwarder** - Executes the actual runtime command
//!
//! # Example
//!
//! ```rust,ignore
//! use vx_resolver::{Executor, ResolverConfig};
//!
//! async fn example() -> anyhow::Result<()> {
//!     let executor = Executor::new(ResolverConfig::default())?;
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
mod executor;
mod resolver;
mod runtime_map;
mod runtime_spec;

pub use config::ResolverConfig;
pub use executor::{execute_system_runtime, Executor};
pub use resolver::{ResolutionResult, Resolver, RuntimeStatus};
pub use runtime_map::RuntimeMap;
pub use runtime_spec::{Ecosystem, RuntimeDependency, RuntimeSpec};

/// Result type for resolver operations
pub type Result<T> = anyhow::Result<T>;
