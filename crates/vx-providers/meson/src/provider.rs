//! meson provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// meson provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct MesonProvider;

impl Provider for MesonProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("meson")
    }

    fn description(&self) -> &str {
        crate::star_metadata()
            .description_or("Meson - An extremely fast and user friendly build system")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("meson", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(MesonProvider)
}
