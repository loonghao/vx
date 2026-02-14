//! ripgrep provider for vx
//!
//! This crate provides the ripgrep provider for vx.
//! ripgrep recursively searches directories for a regex pattern.

mod config;
mod provider;
mod runtime;

pub use config::RipgrepUrlBuilder;
pub use provider::{RipgrepProvider, create_provider};
pub use runtime::RipgrepRuntime;
