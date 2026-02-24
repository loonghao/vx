//! node provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// node provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct NodeProvider;

impl Provider for NodeProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("node")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Node.js JavaScript runtime")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("node", crate::PROVIDER_STAR, Some("node"))
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(NodeProvider)
}
