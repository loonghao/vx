//! vcpkg provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// vcpkg provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct VcpkgProvider;

impl Provider for VcpkgProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("vcpkg")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("C++ library manager for Windows, Linux, and macOS")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("vcpkg", crate::PROVIDER_STAR, Some("vcpkg"))
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(VcpkgProvider)
}
