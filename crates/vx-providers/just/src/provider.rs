//! just provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// just provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct JustProvider;

impl Provider for JustProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("just")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("just")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("just", crate::PROVIDER_STAR, Some("just"))
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(JustProvider)
}
