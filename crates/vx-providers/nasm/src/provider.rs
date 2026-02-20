//! nasm provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// nasm provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct NasmProvider;

impl Provider for NasmProvider {
    fn name(&self) -> &str {
        "nasm"
    }

    fn description(&self) -> &str {
        "Netwide Assembler (NASM)"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("nasm", "nasm", ProviderSource::BuiltIn)
                .with_description("Netwide Assembler (NASM)"),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(NasmProvider)
}
