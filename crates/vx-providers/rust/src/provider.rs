//! rust provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// rust provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct RustProvider;

impl Provider for RustProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("rust")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Rust programming language toolchain")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![
            Arc::new(
                ManifestDrivenRuntime::new("rustup", "rustup", ProviderSource::BuiltIn)
                    .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                        "rust",
                        crate::PROVIDER_STAR,
                    )),
            ),
            Arc::new(ManifestDrivenRuntime::new(
                "cargo",
                "cargo",
                ProviderSource::BuiltIn,
            )),
            Arc::new(ManifestDrivenRuntime::new(
                "rustc",
                "rustc",
                ProviderSource::BuiltIn,
            )),
        ]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(RustProvider)
}
