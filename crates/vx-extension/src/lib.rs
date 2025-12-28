//! # vx-extension
//!
//! Extension system for vx - allows users to extend vx functionality through scripts.
//!
//! Extensions can be written in any language that vx manages (Python, Node.js, etc.)
//! and are executed using the corresponding vx-managed runtime.
//!
//! ## Extension Types
//!
//! - **Command**: Provides new CLI commands via `vx x <extension> [subcommand]`
//! - **Hook**: Executes at specific lifecycle events (future)
//! - **Provider**: Provides new runtime support (future)
//!
//! ## Directory Structure
//!
//! ```text
//! ~/.vx/
//! ├── extensions/           # User-level extensions
//! │   └── my-extension/
//! │       ├── vx-extension.toml
//! │       └── main.py
//! │
//! ├── extensions-dev/       # Local development extensions (symlinks)
//! │   └── dev-ext -> /path/to/dev/extension
//! │
//! └── extensions-cache/     # Remote extension cache (future)
//! ```
//!
//! ## Example
//!
//! ```rust,no_run
//! use vx_extension::{ExtensionManager, ExtensionConfig};
//!
//! async fn example() -> anyhow::Result<()> {
//!     let manager = ExtensionManager::new()?;
//!
//!     // List all extensions
//!     let extensions = manager.list_extensions().await?;
//!
//!     // Execute an extension command
//!     let args: Vec<String> = vec!["subcommand".to_string(), "arg1".to_string()];
//!     manager.execute("my-extension", &args).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod discovery;
pub mod error;
pub mod executor;
pub mod manager;

// Re-exports
pub use config::{ExtensionConfig, ExtensionType, RuntimeRequirement};
pub use discovery::ExtensionDiscovery;
pub use error::{ExtensionError, ExtensionResult};
pub use executor::ExtensionExecutor;
pub use manager::ExtensionManager;

/// Extension metadata loaded from vx-extension.toml
#[derive(Debug, Clone)]
pub struct Extension {
    /// Extension name
    pub name: String,
    /// Extension configuration
    pub config: ExtensionConfig,
    /// Path to the extension directory
    pub path: std::path::PathBuf,
    /// Source of the extension (user, project, dev, builtin)
    pub source: ExtensionSource,
}

/// Source of an extension
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionSource {
    /// Local development extension (~/.vx/extensions-dev/)
    Dev,
    /// Project-level extension (.vx/extensions/)
    Project,
    /// User-level extension (~/.vx/extensions/)
    User,
    /// Built-in extension (shipped with vx)
    Builtin,
}

impl ExtensionSource {
    /// Get the priority of this source (higher = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            ExtensionSource::Dev => 4,
            ExtensionSource::Project => 3,
            ExtensionSource::User => 2,
            ExtensionSource::Builtin => 1,
        }
    }
}

impl std::fmt::Display for ExtensionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtensionSource::Dev => write!(f, "dev"),
            ExtensionSource::Project => write!(f, "project"),
            ExtensionSource::User => write!(f, "user"),
            ExtensionSource::Builtin => write!(f, "builtin"),
        }
    }
}
