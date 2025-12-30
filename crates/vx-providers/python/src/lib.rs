//! Python provider for vx
//!
//! This provider includes:
//! - Python programming language runtime (`python`)
//!
//! Uses python-build-standalone from Astral for portable Python distributions.
//!
//! **Note**: For pure Python development, we recommend using `uv` which provides:
//! - Faster package installation
//! - Built-in virtual environment management
//! - Automatic Python version management
//!
//! Example:
//! ```bash
//! # Install Python
//! vx install python 3.12
//!
//! # Run Python
//! vx python --version
//! vx python script.py
//!
//! # For development, prefer uv:
//! vx uv init my-project
//! vx uv add requests
//! vx uv run python script.py
//! ```

mod config;
mod provider;
mod runtime;

pub use config::PythonUrlBuilder;
pub use provider::PythonProvider;
pub use runtime::PythonRuntime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Python provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(PythonProvider::new())
}
