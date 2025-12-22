//! Java (Temurin JDK) provider for vx
//!
//! This crate provides Java/JDK runtime support using the vx-runtime traits.
//! Uses Eclipse Temurin (formerly AdoptOpenJDK) as the JDK distribution.

pub mod config;
mod provider;
mod runtime;

pub use config::JavaConfig;
pub use provider::JavaProvider;
pub use runtime::JavaRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Java provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(JavaProvider::new())
}
