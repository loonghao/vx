//! MSBuild provider implementation
//!
//! Provides MSBuild as a bundled runtime with .NET SDK.

use crate::runtime::MsbuildRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// MSBuild provider
///
/// MSBuild is bundled with .NET SDK (cross-platform) and Visual Studio (Windows).
/// This provider exposes it as a runtime that users can invoke directly.
#[derive(Debug, Default)]
pub struct MsbuildProvider;

impl MsbuildProvider {
    /// Create a new MSBuild provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for MsbuildProvider {
    fn name(&self) -> &str {
        // Sourced from provider.star: `def name(): return "msbuild"`
        crate::star_metadata().name_or("msbuild")
    }

    fn description(&self) -> &str {
        // Sourced from provider.star: `def description(): return "..."`
        use std::sync::OnceLock;
        static DESC: OnceLock<&'static str> = OnceLock::new();
        DESC.get_or_init(|| {
            let s = crate::star_metadata()
                .description
                .as_deref()
                .unwrap_or("Microsoft Build Engine - bundled with .NET SDK");
            Box::leak(s.to_string().into_boxed_str())
        })
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(MsbuildRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        matches!(name, "msbuild" | "msbuild.exe")
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if self.supports(name) {
            Some(Arc::new(MsbuildRuntime::new()))
        } else {
            None
        }
    }
}
