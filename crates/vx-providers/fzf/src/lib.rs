//! fzf provider for vx
//!
//! This crate provides the fzf fuzzy finder provider for vx.
//! fzf is a general-purpose command-line fuzzy finder.

mod config;
mod provider;
mod runtime;

pub use config::FzfUrlBuilder;
pub use provider::{create_provider, FzfProvider};
pub use runtime::FzfRuntime;
