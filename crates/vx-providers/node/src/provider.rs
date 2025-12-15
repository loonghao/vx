//! Node.js provider implementation
//!
//! This module provides the NodeProvider which bundles Node.js, NPM, and NPX runtimes.

use crate::runtime::{NodeRuntime, NpmRuntime, NpxRuntime};
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Node.js provider that bundles Node.js runtime and package management tools
///
/// This provider includes:
/// - `node` - Node.js JavaScript runtime
/// - `npm` - Node Package Manager
/// - `npx` - Node Package Runner
#[derive(Debug, Default)]
pub struct NodeProvider;

impl NodeProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for NodeProvider {
    fn name(&self) -> &str {
        "node"
    }

    fn description(&self) -> &str {
        "Node.js JavaScript runtime and package management tools"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![
            Arc::new(NodeRuntime::new()),
            Arc::new(NpmRuntime::new()),
            Arc::new(NpxRuntime::new()),
        ]
    }
}
