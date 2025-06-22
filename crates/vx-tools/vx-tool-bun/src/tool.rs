//! Simplified Bun tool implementation using vx-config system

use anyhow::Result;
use async_trait::async_trait;
use vx_plugin::{ConfigBasedTool, VxTool};

/// Bun tool implementation using configuration system
#[derive(Debug, Clone)]
pub struct BunConfigTool {
    inner: ConfigBasedTool,
}

impl BunConfigTool {
    /// Create a new Bun tool
    pub fn new() -> Self {
        Self {
            inner: ConfigBasedTool::new("bun"),
        }
    }
}

impl Default for BunConfigTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VxTool for BunConfigTool {
    fn name(&self) -> &str {
        "bun"
    }

    fn description(&self) -> &str {
        // For now, use static description - dynamic config loading would require async
        // In a real implementation, this could be cached or loaded at startup
        "Fast all-in-one JavaScript runtime"
    }

    fn aliases(&self) -> Vec<&str> {
        // For now, use static aliases - dynamic config loading would require async
        vec![]
    }

    async fn fetch_versions(
        &self,
        include_prerelease: bool,
    ) -> Result<Vec<vx_plugin::VersionInfo>> {
        self.inner.fetch_versions(include_prerelease).await
    }
}

/// Create a new bun tool instance
pub fn create_bun_tool() -> Box<dyn VxTool> {
    Box::new(BunConfigTool::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bun_config_tool() {
        let tool = BunConfigTool::new();
        assert_eq!(tool.name(), "bun");
        assert_eq!(tool.description(), "Fast all-in-one JavaScript runtime");
        assert!(tool.aliases().is_empty());
    }
}
