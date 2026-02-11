//! yq provider implementation

use crate::runtime::YqRuntime;
use std::sync::Arc;
use vx_runtime::{provider::Provider, Runtime};

/// yq provider
#[derive(Debug, Default)]
pub struct YqProvider;

impl YqProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for YqProvider {
    fn name(&self) -> &str {
        "yq"
    }

    fn description(&self) -> &str {
        "A portable command-line YAML, JSON, XML, CSV, TOML and properties processor"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(YqRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "yq"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "yq" {
            Some(Arc::new(YqRuntime::new()))
        } else {
            None
        }
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(YqProvider::new())
}
