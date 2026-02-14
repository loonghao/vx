//! Starship provider for vx
//!
//! This crate provides the Starship cross-shell prompt provider for vx.
//! Starship is the minimal, blazing-fast, and infinitely customizable prompt for any shell.

mod config;
mod provider;
mod runtime;

pub use config::StarshipUrlBuilder;
pub use provider::{StarshipProvider, create_provider};
pub use runtime::StarshipRuntime;
