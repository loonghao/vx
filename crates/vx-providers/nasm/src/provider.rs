//! nasm provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

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
        vec![Arc::new(
            ManifestDrivenRuntime::new("nasm", "nasm", ProviderSource::BuiltIn)
                .with_description("Netwide Assembler (NASM)")
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "nasm",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(NasmProvider)
}
