//! awscli provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// awscli provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct AwsCliProvider;

impl Provider for AwsCliProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("awscli")
    }

    fn description(&self) -> &str {
        crate::star_metadata()
            .description_or("AWS CLI - Unified command line interface to Amazon Web Services")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("awscli", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(AwsCliProvider)
}
