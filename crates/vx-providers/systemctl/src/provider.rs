//! systemctl provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// systemctl provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct SystemctlProvider;

impl Provider for SystemctlProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("systemctl")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("systemd system and service manager")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("systemctl", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(SystemctlProvider)
}
