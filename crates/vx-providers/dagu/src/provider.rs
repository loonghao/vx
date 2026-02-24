//! dagu provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// dagu provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct DaguProvider;

impl Provider for DaguProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("dagu")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Dagu - Cron alternative with a Web UI")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("dagu", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(DaguProvider)
}
