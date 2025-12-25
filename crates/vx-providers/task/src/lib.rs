//! Task provider for vx
//!
//! This crate provides the Task (go-task) runner provider for vx.
//! Task is a task runner / simpler Make alternative written in Go.
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_task::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "task");
//! ```

mod config;
mod provider;
mod runtime;

pub use config::TaskUrlBuilder;
pub use provider::TaskProvider;
pub use runtime::TaskRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Task provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(TaskProvider::new())
}
