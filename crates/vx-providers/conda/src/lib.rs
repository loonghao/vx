//! Conda provider for vx
//!
//! This crate provides the Conda provider for vx, enabling package, dependency
//! and environment management for any language, with special focus on Python
//! and scientific computing (including CUDA, PyTorch, TensorFlow).
//!
//! # Supported Runtimes
//!
//! - **`micromamba`** (Recommended) - Minimal standalone mamba, single binary
//! - `conda` - Full Conda installation (via Miniforge, requires installer)
//! - `mamba` - Fast package manager (bundled with Miniforge)
//!
//! # Why Micromamba?
//!
//! Micromamba is recommended because:
//! - Single binary, no installer needed
//! - Fast and lightweight (~10MB)
//! - Fully compatible with conda environments and packages
//! - Can install PyTorch, TensorFlow, CUDA, etc.
//!
//! # Example Usage
//!
//! ```bash
//! # Install micromamba
//! vx install micromamba
//!
//! # Create a conda environment with PyTorch and CUDA
//! vx micromamba create -n ml python=3.11 pytorch pytorch-cuda=12.1 -c pytorch -c nvidia
//!
//! # Activate and use the environment
//! vx micromamba run -n ml python train.py
//! ```
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_conda::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "conda");
//! ```

mod config;
mod provider;
mod runtime;

pub use config::CondaUrlBuilder;
pub use provider::CondaProvider;
pub use runtime::{CondaRuntime, MambaRuntime, MicromambaRuntime};

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new Conda provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(CondaProvider::new())
}
