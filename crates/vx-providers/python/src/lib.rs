//! Python provider for vx
//!
//! This provider includes:
//! - Python programming language runtime (`python`)
//! - pip package installer (`pip`) - bundled with Python
//!
//! Uses python-build-standalone for portable Python distributions:
//! - Direct download from GitHub releases
//! - Consistent installation across platforms
//! - Support for Python 3.9 - 3.15 (3.7 and 3.8 are EOL)
//!
//! Example:
//! ```bash
//! # Install Python
//! vx install python@3.12
//!
//! # Run Python
//! vx python@3.11 --version
//! vx python script.py
//!
//! # pip is also available
//! vx pip install requests
//! ```

mod provider;
mod runtime;

pub use provider::PythonProvider;
pub use runtime::{PipRuntime, PythonRuntime};

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Python provider instance
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(PythonProvider::new())
}
