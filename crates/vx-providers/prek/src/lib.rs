//! prek Provider for vx
//!
//! This provider adds support for prek - a better `pre-commit` framework,
//! re-engineered in Rust. Single binary, fully compatible with pre-commit
//! configuration, faster and lighter.
//!
//! ## Features
//!
//! - Version management via GitHub releases
//! - Cross-platform support (Windows, macOS, Linux)
//! - Automatic installation and verification
//! - Drop-in replacement for pre-commit
//! - Native monorepo support
//!
//! ## Usage with vx
//!
//! ```bash
//! # Install prek
//! vx install prek
//!
//! # Install git hooks
//! vx prek install
//!
//! # Run all hooks
//! vx prek run --all-files
//!
//! # Auto-update hooks
//! vx prek auto-update
//! ```

use std::sync::Arc;

pub mod config;
pub mod provider;
pub mod runtime;

pub use config::PrekUrlBuilder;
pub use provider::PrekProvider;
pub use runtime::PrekRuntime;

/// Factory function to create the prek provider
pub fn create_provider() -> Arc<dyn vx_runtime::Provider> {
    Arc::new(provider::PrekProvider::new())
}
