//! Jujutsu (jj) provider for vx
//!
//! This provider manages the `jj` Git-compatible DVCS tool.

mod config;
mod provider;
mod runtime;

pub use config::JjUrlBuilder;
pub use provider::{JjProvider, create_provider};
pub use runtime::JjRuntime;
