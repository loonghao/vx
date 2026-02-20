//! zig provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// zig provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct ZigProvider;

impl Provider for ZigProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("zig")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Zig programming language and toolchain")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("zig", "zig", ProviderSource::BuiltIn)
                .with_description("Zig programming language and toolchain")
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "zig",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(ZigProvider)
}
