//! Visual Studio Code provider for vx
//!
//! This provider includes:
//! - Visual Studio Code editor (`code`, `vscode`)

mod config;
mod provider;
mod runtime;

pub use config::VscodeUrlBuilder;
pub use provider::VscodeProvider;
pub use runtime::VscodeRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new VSCode provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(VscodeProvider::new())
}
