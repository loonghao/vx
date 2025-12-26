//! Spack provider implementation
//!
//! Provides the Spack runtime for HPC and scientific computing.

use crate::runtime::SpackRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Spack provider
#[derive(Debug, Default)]
pub struct SpackProvider;

impl SpackProvider {
    /// Create a new Spack provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for SpackProvider {
    fn name(&self) -> &str {
        "spack"
    }

    fn description(&self) -> &str {
        "Spack - A flexible package manager for HPC and scientific computing"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(SpackRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "spack"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "spack" {
            Some(Arc::new(SpackRuntime::new()))
        } else {
            None
        }
    }
}
