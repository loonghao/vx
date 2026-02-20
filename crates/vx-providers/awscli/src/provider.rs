//! awscli provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// awscli provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct AwsCliProvider;

impl Provider for AwsCliProvider {
    fn name(&self) -> &str {
        "awscli"
    }

    fn description(&self) -> &str {
        "AWS CLI - Unified command line interface to Amazon Web Services"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "aws",
            "aws",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(AwsCliProvider)
}
