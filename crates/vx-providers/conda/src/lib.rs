//! Conda provider for vx
//!
//! This crate provides the Conda provider for vx, enabling package, dependency
//! and environment management for any language, with special focus on Python
//! and scientific computing (including CUDA, PyTorch, TensorFlow).
//!
//! # Supported Runtimes
//!
//! - **`micromamba`** (Recommended) - Minimal standalone mamba, single binary
//! - `conda` - Full Conda installation (via Miniforge)
//! - `mamba` - Fast package manager (bundled with Miniforge)
//!
//! # Why Micromamba?
//!
//! Micromamba is recommended because:
//! - Single binary, no installer needed (fits vx's download → extract → use philosophy)
//! - Fast and lightweight (~10MB vs ~400MB+ for full Conda)
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
//! # Run command in the environment
//! vx micromamba run -n ml python train.py
//! ```

mod config;
mod provider;
mod runtime;

pub use config::CondaUrlBuilder;
pub use provider::{CondaProvider, create_provider};
pub use runtime::{CondaRuntime, MambaRuntime, MicromambaRuntime};
