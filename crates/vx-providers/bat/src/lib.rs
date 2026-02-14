//! bat provider for vx
//!
//! This crate provides the bat provider for vx.
//! bat is a cat clone with syntax highlighting and Git integration.

mod config;
mod provider;
mod runtime;

pub use config::BatUrlBuilder;
pub use provider::{BatProvider, create_provider};
pub use runtime::BatRuntime;
