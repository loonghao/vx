//! x-cmd provider implementation

use crate::runtime::XCmdRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// x-cmd provider
#[derive(Debug)]
pub struct XCmdProvider;

impl XCmdProvider {
    /// Create a new x-cmd provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for XCmdProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for XCmdProvider {
    fn name(&self) -> &str {
        "x-cmd"
    }

    fn description(&self) -> &str {
        "Provides x-cmd command-line toolbox support"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(XCmdRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "x-cmd" || name == "xcmd" || name == "x_cmd"
    }
}
