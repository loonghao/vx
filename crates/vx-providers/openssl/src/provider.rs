//! openssl provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// openssl provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct OpensslProvider;

impl Provider for OpensslProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("openssl")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Cryptography and SSL/TLS toolkit")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("openssl", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(OpensslProvider)
}
