//! protoc provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// protoc provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct ProtocProvider;

impl Provider for ProtocProvider {
    fn name(&self) -> &str {
        "protoc"
    }

    fn description(&self) -> &str {
        "Protocol Buffers compiler"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("protoc", "protoc", ProviderSource::BuiltIn)
                .with_description("Protocol Buffers compiler"),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(ProtocProvider)
}
