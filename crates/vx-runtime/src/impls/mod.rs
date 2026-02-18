//! Real implementations of runtime traits
//!
//! This module provides lightweight production implementations of the abstract traits
//! defined in `traits.rs`.
//!
//! Heavy implementations (HTTP client, installer, context factory) have been moved to
//! `vx-runtime-http` crate (RFC 0032) for faster compilation.

mod command_executor;
mod file_system;
mod path_provider;

pub use command_executor::RealCommandExecutor;
pub use file_system::RealFileSystem;
pub use path_provider::RealPathProvider;
