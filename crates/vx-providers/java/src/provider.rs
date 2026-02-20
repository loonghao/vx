//! java provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// java provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct JavaProvider;

impl Provider for JavaProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("java")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Java Development Kit")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("java", "java", ProviderSource::BuiltIn)
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "java",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(JavaProvider)
}
