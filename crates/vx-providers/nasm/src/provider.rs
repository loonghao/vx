//! nasm provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// nasm provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct NasmProvider;

impl Provider for NasmProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("nasm")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Netwide Assembler (NASM)")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("nasm", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(NasmProvider)
}
