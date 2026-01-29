//! PowerShell provider implementation
//!
//! This provider manages PowerShell 7+ (pwsh) installation and execution.

use crate::runtime::PwshRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// PowerShell provider
///
/// Provides the PowerShell 7+ runtime.
#[derive(Debug, Clone, Default)]
pub struct PwshProvider;

impl PwshProvider {
    /// Create a new PowerShell provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for PwshProvider {
    fn name(&self) -> &str {
        "pwsh"
    }

    fn description(&self) -> &str {
        "PowerShell - Cross-platform shell and scripting language"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(PwshRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "pwsh" || name == "powershell" || name == "ps"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if self.supports(name) {
            Some(Arc::new(PwshRuntime::new()))
        } else {
            None
        }
    }
}

/// Create a new PowerShell provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(PwshProvider::new())
}
