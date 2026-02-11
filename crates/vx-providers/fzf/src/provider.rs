//! fzf provider implementation

use crate::runtime::FzfRuntime;
use std::sync::Arc;
use vx_runtime::{provider::Provider, Runtime};

/// fzf provider
#[derive(Debug, Default)]
pub struct FzfProvider;

impl FzfProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for FzfProvider {
    fn name(&self) -> &str {
        "fzf"
    }

    fn description(&self) -> &str {
        "A command-line fuzzy finder"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(FzfRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "fzf"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "fzf" {
            Some(Arc::new(FzfRuntime::new()))
        } else {
            None
        }
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(FzfProvider::new())
}
