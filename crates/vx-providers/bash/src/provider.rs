//! bash provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// bash provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct BashProvider;

impl Provider for BashProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("bash")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("GNU Bourne Again SHell - the standard Unix shell")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("bash", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(BashProvider)
}
