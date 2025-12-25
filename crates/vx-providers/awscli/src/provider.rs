//! AWS CLI provider implementation

use crate::runtime::AwsCliRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// AWS CLI provider
#[derive(Debug)]
pub struct AwsCliProvider;

impl AwsCliProvider {
    /// Create a new AWS CLI provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for AwsCliProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for AwsCliProvider {
    fn name(&self) -> &str {
        "awscli"
    }

    fn description(&self) -> &str {
        "Provides AWS CLI v2 support for Amazon Web Services"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(AwsCliRuntime::new())]
    }
}
