//! vscode provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// vscode provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct VscodeProvider;

impl Provider for VscodeProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("vscode")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Visual Studio Code")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("vscode", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(VscodeProvider)
}
