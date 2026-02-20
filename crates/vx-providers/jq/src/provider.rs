//! jq provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// jq provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct JqProvider;

impl Provider for JqProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("jq")
    }

    fn description(&self) -> &str {
        crate::star_metadata()
            .description_or("A lightweight and flexible command-line JSON processor")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("jq", "jq", ProviderSource::BuiltIn).with_fetch_versions(
                vx_starlark::make_fetch_versions_fn("jq", crate::PROVIDER_STAR),
            ),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(JqProvider)
}
