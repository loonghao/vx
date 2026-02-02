//! .NET SDK provider implementation
//!
//! Provides the .NET SDK runtime for C# and F# development.

use crate::runtime::DotnetRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// .NET SDK provider
#[derive(Debug, Default)]
pub struct DotnetProvider;

impl DotnetProvider {
    /// Create a new .NET SDK provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for DotnetProvider {
    fn name(&self) -> &str {
        "dotnet"
    }

    fn description(&self) -> &str {
        ".NET SDK - Free, cross-platform, open-source developer platform for C#, F#, and VB.NET"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(DotnetRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        matches!(name, "dotnet" | "dotnet-sdk")
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if self.supports(name) {
            Some(Arc::new(DotnetRuntime::new()))
        } else {
            None
        }
    }
}
