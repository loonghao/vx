//! kubectl provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// kubectl provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct KubectlProvider;

impl Provider for KubectlProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("kubectl")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("kubectl - Kubernetes command-line tool")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("kubectl", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(KubectlProvider)
}
