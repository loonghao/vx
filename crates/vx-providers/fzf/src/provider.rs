//! fzf provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// fzf provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct FzfProvider;

impl Provider for FzfProvider {
    fn name(&self) -> &str {
        "fzf"
    }

    fn description(&self) -> &str {
        "A command-line fuzzy finder"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(
            ManifestDrivenRuntime::new("fzf", "fzf", ProviderSource::BuiltIn)
                .with_description("A command-line fuzzy finder"),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(FzfProvider)
}
