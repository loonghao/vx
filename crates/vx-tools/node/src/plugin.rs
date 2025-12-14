//! Node.js bundle implementation

use crate::npm_pm::NpmPackageManager;
use crate::tool::{NodeTool, NpmTool, NpxTool};
use vx_plugin::{PackageManager, ToolBundle, VxTool};

/// Node.js bundle that provides Node.js runtime and npm package manager
///
/// This bundle includes:
/// - `node` - Node.js JavaScript runtime
/// - `npm` - Node Package Manager (as both tool and package manager)
/// - `npx` - Node Package Runner
#[derive(Debug)]
pub struct NodePlugin;

impl NodePlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl ToolBundle for NodePlugin {
    fn name(&self) -> &str {
        "node"
    }

    fn description(&self) -> &str {
        "Node.js JavaScript runtime and package management tools"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn tools(&self) -> Vec<Box<dyn VxTool>> {
        vec![
            Box::new(NodeTool::new()),
            Box::new(NpmTool::new()),
            Box::new(NpxTool::new()),
        ]
    }

    fn package_managers(&self) -> Vec<Box<dyn PackageManager>> {
        vec![Box::new(NpmPackageManager::new())]
    }

    fn supports_tool(&self, tool_name: &str) -> bool {
        matches!(tool_name, "node" | "npm" | "npx" | "nodejs")
    }
}

impl Default for NodePlugin {
    fn default() -> Self {
        Self::new()
    }
}
