//! Windows Package Manager provider implementation

use crate::runtime::WingetRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Windows Package Manager provider
#[derive(Debug)]
pub struct WingetProvider;

impl WingetProvider {
    /// Create a new winget provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for WingetProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for WingetProvider {
    fn name(&self) -> &str {
        // Sourced from provider.star: `def name(): return "winget"`
        crate::star_metadata().name_or("winget")
    }

    fn description(&self) -> &str {
        // Sourced from provider.star: `def description(): return "..."`
        use std::sync::OnceLock;
        static DESC: OnceLock<&'static str> = OnceLock::new();
        DESC.get_or_init(|| {
            let s = crate::star_metadata()
                .description
                .as_deref()
                .unwrap_or("Provides Windows Package Manager (winget) support");
            Box::leak(s.to_string().into_boxed_str())
        })
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(WingetRuntime::new())]
    }
}
