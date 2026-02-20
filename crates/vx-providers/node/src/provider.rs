//! node provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// node provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct NodeProvider;

impl Provider for NodeProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("node")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Node.js JavaScript runtime")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![
            Arc::new(
                ManifestDrivenRuntime::new("node", "node", ProviderSource::BuiltIn)
                    .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                        "node",
                        crate::PROVIDER_STAR,
                    )),
            ),
            Arc::new(ManifestDrivenRuntime::new(
                "npm",
                "npm",
                ProviderSource::BuiltIn,
            )),
            Arc::new(ManifestDrivenRuntime::new(
                "npx",
                "npx",
                ProviderSource::BuiltIn,
            )),
            Arc::new(ManifestDrivenRuntime::new(
                "corepack",
                "corepack",
                ProviderSource::BuiltIn,
            )),
        ]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(NodeProvider)
}
