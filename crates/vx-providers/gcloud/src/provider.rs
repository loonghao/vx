//! Google Cloud CLI provider implementation

use crate::runtime::{BqRuntime, GcloudRuntime, GsutilRuntime};
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Google Cloud CLI provider
#[derive(Debug)]
pub struct GcloudProvider;

impl GcloudProvider {
    /// Create a new gcloud provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for GcloudProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for GcloudProvider {
    fn name(&self) -> &str {
        "gcloud"
    }

    fn description(&self) -> &str {
        "Provides Google Cloud SDK/gcloud CLI support for Google Cloud Platform"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![
            Arc::new(GcloudRuntime::new()),
            Arc::new(GsutilRuntime::new()),
            Arc::new(BqRuntime::new()),
        ]
    }
}
