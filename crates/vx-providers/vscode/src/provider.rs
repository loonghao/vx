//! vscode provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// vscode provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct VsCodeProvider;

impl Provider for VsCodeProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("vscode")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Visual Studio Code - Code editing. Redefined.")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("code", "code", ProviderSource::BuiltIn)
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "vscode",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(VsCodeProvider)
}
