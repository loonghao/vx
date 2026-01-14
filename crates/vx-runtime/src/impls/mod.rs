//! Real implementations of runtime traits
//!
//! This module provides production implementations of the abstract traits
//! defined in `traits.rs`.

mod command_executor;
mod context;
mod file_system;
mod http_client;
mod installer;
mod path_provider;

pub use command_executor::RealCommandExecutor;
pub use context::{create_runtime_context, create_runtime_context_with_base};
pub use file_system::RealFileSystem;
pub use http_client::RealHttpClient;
pub use installer::RealInstaller;
pub use path_provider::RealPathProvider;
