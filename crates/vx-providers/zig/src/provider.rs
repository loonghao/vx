//! zig provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// zig provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct ZigProvider;

impl Provider for ZigProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("zig")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Zig - A general-purpose programming language")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("zig", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(ZigProvider)
}
