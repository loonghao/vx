//! vscode provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// vscode provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct VsCodeProvider;

impl Provider for VsCodeProvider {
    fn name(&self) -> &str {
        "vscode"
    }

    fn description(&self) -> &str {
        "Visual Studio Code - Code editing. Redefined."
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "code",
            "code",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(VsCodeProvider)
}
