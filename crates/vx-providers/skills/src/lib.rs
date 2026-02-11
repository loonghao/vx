//! Vercel Skills provider for vx
//!
//! This crate provides the Vercel Skills CLI tool provider for vx.
//! Skills is the open agent skills tool for managing AI coding agent skills
//! across multiple AI assistants (Claude Code, Cursor, CodeBuddy, Codex, etc.).
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_skills::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "skills");
//! ```

mod config;
mod provider;
mod runtime;

pub use config::SkillsUrlBuilder;
pub use provider::SkillsProvider;
pub use runtime::SkillsRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Skills provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(SkillsProvider::new())
}
