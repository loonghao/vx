//! jq provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// jq provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct JqProvider;

impl Provider for JqProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("jq")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("jq - Command-line JSON processor")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("jq", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(JqProvider)
}
