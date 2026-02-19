//! 7-Zip provider implementation

use crate::runtime::SevenZipRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// 7-Zip file archiver provider
#[derive(Debug)]
pub struct SevenZipProvider;

impl SevenZipProvider {
    /// Create a new 7-Zip provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for SevenZipProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for SevenZipProvider {
    fn name(&self) -> &str {
        "7zip"
    }

    fn description(&self) -> &str {
        "7-Zip file archiver with high compression ratio"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(SevenZipRuntime::new())]
    }
}
