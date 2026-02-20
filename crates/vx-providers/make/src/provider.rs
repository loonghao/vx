//! Make provider implementation

use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

use crate::MakeRuntime;

/// Make provider
#[derive(Debug, Default)]
pub struct MakeProvider;

impl MakeProvider {
    /// Create a new Make provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for MakeProvider {
    fn name(&self) -> &str {
        // Sourced from provider.star: `def name(): return "make"`
        crate::star_metadata().name_or("make")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("make")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(MakeRuntime::new())]
    }
}
