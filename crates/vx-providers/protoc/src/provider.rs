//! protoc provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// protoc provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct ProtocProvider;

impl Provider for ProtocProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("protoc")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("protoc - Protocol Buffers compiler")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("protoc", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(ProtocProvider)
}
