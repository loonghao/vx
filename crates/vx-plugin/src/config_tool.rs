//! Configuration-based tool implementation
//!
//! This module provides a simplified way to implement tools using the vx-config system.

use crate::VxTool;
use anyhow::Result;
use async_trait::async_trait;

/// A simple tool implementation that uses the vx-config system for configuration
#[derive(Debug, Clone)]
pub struct ConfigBasedTool {
    name: String,
}

impl ConfigBasedTool {
    /// Create a new config-based tool
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[async_trait]
impl VxTool for ConfigBasedTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        // This will be overridden by specific tool implementations
        "A tool managed by vx configuration system"
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<crate::VersionInfo>> {
        // Simple placeholder implementation
        let _ = include_prerelease;
        Ok(vec![
            crate::VersionInfo::new("1.0.0"),
            crate::VersionInfo::new("1.1.0"),
            crate::VersionInfo::new("latest"),
        ])
    }
}
