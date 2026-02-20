//! awscli provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

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
        vec![Arc::new(
            ManifestDrivenRuntime::new("aws", "aws", ProviderSource::BuiltIn).with_fetch_versions(
                vx_starlark::make_fetch_versions_fn("awscli", crate::PROVIDER_STAR),
            ),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(AwsCliProvider)
}
