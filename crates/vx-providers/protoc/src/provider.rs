//! protoc provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// protoc provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct ProtocProvider;

impl Provider for ProtocProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("protoc")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Protocol Buffers compiler")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("protoc", "protoc", ProviderSource::BuiltIn)
                .with_description("Protocol Buffers compiler")
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "protoc",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(ProtocProvider)
}
