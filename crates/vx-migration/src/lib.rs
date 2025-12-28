//! # vx-migration
//!
//! A pluggable migration framework for vx configuration and data.
//!
//! ## Features
//!
//! - **Plugin-based design**: Add migrations by implementing the `Migration` trait
//! - **Lifecycle hooks**: Support for pre/post migration hooks
//! - **Dependency management**: Define dependencies between migrations
//! - **Dry-run mode**: Preview changes without executing
//! - **History tracking**: Track all migration operations
//! - **Rollback support**: Reversible migrations can be rolled back
//!
//! ## Example
//!
//! ```rust,ignore
//! use vx_migration::prelude::*;
//! use vx_migration::migrations::create_default_engine;
//!
//! let engine = create_default_engine();
//! let options = MigrationOptions::default();
//! let result = engine.migrate(Path::new("./my-project"), &options).await?;
//! ```

pub mod context;
pub mod engine;
pub mod error;
pub mod history;
pub mod migrations;
pub mod registry;
pub mod traits;
pub mod types;
pub mod version;

pub use error::{MigrationError, MigrationResult};
pub use types::{MigrationCategory, MigrationMetadata, MigrationOptions, MigrationPriority};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::context::MigrationContext;
    pub use crate::engine::MigrationEngine;
    pub use crate::error::{MigrationError, MigrationResult};
    pub use crate::history::{MigrationHistory, MigrationHistoryEntry};
    pub use crate::registry::MigrationRegistry;
    pub use crate::traits::{Migration, MigrationHook};
    pub use crate::types::{
        Change, ChangeType, MigrationCategory, MigrationMetadata, MigrationOptions,
        MigrationPriority, MigrationReport, MigrationStepResult,
    };
    pub use crate::version::{Version, VersionRange};
}
