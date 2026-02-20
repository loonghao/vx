//! vcpkg provider implementation

use std::sync::Arc;
use vx_runtime::{ManifestDrivenRuntime, ProviderSource, Runtime, provider::Provider};

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
        vec![Arc::new(
            ManifestDrivenRuntime::new("vcpkg", "vcpkg", ProviderSource::BuiltIn)
                .with_fetch_versions(vx_starlark::make_fetch_versions_fn(
                    "vcpkg",
                    crate::PROVIDER_STAR,
                )),
        )]
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(VcpkgProvider)
}
