//! Starship provider implementation

use crate::runtime::StarshipRuntime;
use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// Starship provider
#[derive(Debug, Default)]
pub struct StarshipProvider;

impl StarshipProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for StarshipProvider {
    fn name(&self) -> &str {
        "starship"
    }

    fn description(&self) -> &str {
        "The minimal, blazing-fast, and infinitely customizable prompt for any shell"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(StarshipRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "starship"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "starship" {
            Some(Arc::new(StarshipRuntime::new()))
        } else {
            None
        }
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(StarshipProvider::new())
}
