//! NuGet provider implementation

use crate::runtime::NugetRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// NuGet provider
#[derive(Debug)]
pub struct NugetProvider;

impl NugetProvider {
    /// Create a new NuGet provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for NugetProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for NugetProvider {
    fn name(&self) -> &str {
        "nuget"
    }

    fn description(&self) -> &str {
        "Provides NuGet package manager support for .NET"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(NugetRuntime::new())]
    }
}
