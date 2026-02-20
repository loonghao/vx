//! MSVC Build Tools provider implementation
//!
//! Provides the MSVC Build Tools runtime for C/C++ compilation on Windows.
//! Downloads directly from Microsoft's official servers.
//!
//! Provider metadata (name, description, aliases, platform constraints) is
//! sourced from `provider.star` via `crate::star_metadata()`, so that the
//! Starlark file remains the single source of truth.

use crate::runtime::MsvcRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// MSVC Build Tools provider
#[derive(Debug, Default)]
pub struct MsvcProvider;

impl MsvcProvider {
    /// Create a new MSVC provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for MsvcProvider {
    fn name(&self) -> &str {
        // Sourced from provider.star: `def name(): return "msvc"`
        crate::star_metadata().name_or("msvc")
    }

    fn description(&self) -> &str {
        // Sourced from provider.star: `def description(): return "..."`
        // We need a &'static str; use OnceLock to leak once.
        use std::sync::OnceLock;
        static DESC: OnceLock<&'static str> = OnceLock::new();
        DESC.get_or_init(|| {
            let s = crate::star_metadata()
                .description
                .as_deref()
                .unwrap_or("MSVC Build Tools - Microsoft Visual C++ compiler and tools");
            Box::leak(s.to_string().into_boxed_str())
        })
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(MsvcRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        // Check against the primary runtime name and all aliases from provider.star
        if name == self.name() {
            return true;
        }
        // Check aliases defined in provider.star runtimes
        crate::star_metadata()
            .runtimes
            .iter()
            .any(|r| r.name.as_deref() == Some(name) || r.aliases.iter().any(|a| a == name))
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if self.supports(name) {
            Some(Arc::new(MsvcRuntime::new()))
        } else {
            None
        }
    }
}
