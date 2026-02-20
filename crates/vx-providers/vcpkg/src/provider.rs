//! vcpkg provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

/// vcpkg provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct VcpkgProvider;

impl Provider for VcpkgProvider {
    fn name(&self) -> &str {
        "vcpkg"
    }

    fn description(&self) -> &str {
        "C++ library manager for Windows, Linux, and macOS"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(ManifestDrivenRuntime::new(
            "vcpkg",
            "vcpkg",
            ProviderSource::BuiltIn,
        ))]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(VcpkgProvider)
}
