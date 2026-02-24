//! xcodebuild provider implementation

use std::sync::Arc;
use vx_runtime::{Runtime, provider::Provider};

/// xcodebuild provider (Starlark-driven)
#[derive(Debug, Default)]
pub struct XcodebuildProvider;

impl Provider for XcodebuildProvider {
    fn name(&self) -> &str {
        crate::star_metadata().name_or("xcodebuild")
    }

    fn description(&self) -> &str {
        crate::star_metadata().description_or("Apple Xcode build tools")
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vx_starlark::build_runtimes("xcodebuild", crate::PROVIDER_STAR, None)
    }
}

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(XcodebuildProvider)
}
