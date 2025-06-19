//! # vx-dependency
//!
//! Advanced dependency resolution and management system for vx tools.
//!
//! This crate provides intelligent dependency resolution with support for:
//! - Multi-layer dependency chains
//! - Circular dependency detection
//! - Version constraint resolution
//! - Parallel dependency installation
//! - Caching and performance optimization
//!
//! ## Example
//!
//! ```rust,no_run
//! use vx_dependency::{DependencyResolver, ToolSpec, DependencySpec};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut resolver = DependencyResolver::new();
//!     
//!     // Register tools and their dependencies
//!     resolver.register_tool(ToolSpec {
//!         name: "yarn".to_string(),
//!         dependencies: vec![
//!             DependencySpec::required("node", ">=16.0.0")
//!         ],
//!         ..Default::default()
//!     });
//!     
//!     // Resolve dependencies for yarn
//!     let resolution = resolver.resolve("yarn").await?;
//!     println!("Install order: {:?}", resolution.install_order);
//!     
//!     Ok(())
//! }
//! ```

pub mod dependency;
pub mod graph;
pub mod resolver;
pub mod types;
pub mod version;

// Re-export main types
pub use graph::{
    DependencyGraph, DependencyNode, GraphStats, NodeState, ResolutionResult, VersionConflict,
};
pub use resolver::{DependencyResolver, ResolutionOptions};
pub use types::{DependencySpec, DependencyType, ToolSpec, VersionConstraint};
pub use version::{Version, VersionMatcher, VersionRange};

/// Result type for dependency operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for dependency operations
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Tool '{tool}' not found")]
    ToolNotFound { tool: String },

    #[error("Circular dependency detected: {cycle:?}")]
    CircularDependency { cycle: Vec<String> },

    #[error("Version conflict for tool '{tool}': required {required}, found {found}")]
    VersionConflict {
        tool: String,
        required: String,
        found: String,
    },

    #[error("Dependency resolution failed: {message}")]
    ResolutionFailed { message: String },

    #[error("Invalid version constraint: {constraint}")]
    InvalidVersionConstraint { constraint: String },

    #[error("Tool '{tool}' has unresolvable dependencies: {dependencies:?}")]
    UnresolvableDependencies {
        tool: String,
        dependencies: Vec<String>,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Version information for the crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
    }
}
