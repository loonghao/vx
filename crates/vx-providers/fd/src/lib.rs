//! fd provider for vx
//!
//! This crate provides the fd file finder provider for vx.
//! fd is a simple, fast and user-friendly alternative to find.

mod config;
mod provider;
mod runtime;

pub use config::FdUrlBuilder;
pub use provider::{FdProvider, create_provider};
pub use runtime::FdRuntime;
