//! Actrun provider implementation
//!
//! Provides the Actrun CLI runner from Actionforge.

use crate::runtime::ActrunRuntime;
use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// Actrun provider
#[derive(Debug, Default)]
pub struct ActrunProvider;

impl ActrunProvider {
    /// Create a new Actrun provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for ActrunProvider {
    fn name(&self) -> &str {
        "actrun"
    }

    fn description(&self) -> &str {
        "Actrun - The runner executable of Actionforge"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ActrunRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "actrun"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "actrun" {
            Some(Arc::new(ActrunRuntime::new()))
        } else {
            None
        }
    }
}

/// Create the Actrun provider
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(ActrunProvider::new())
}
