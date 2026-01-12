//! Package manager registry

use crate::managers::{
    AptManager, ChocolateyManager, HomebrewManager, SystemPackageManager, WingetManager,
};
use crate::{Result, SystemPmError};
use std::collections::HashMap;
use std::sync::Arc;

/// Registry of available package managers
pub struct PackageManagerRegistry {
    managers: HashMap<String, Arc<dyn SystemPackageManager>>,
}

impl PackageManagerRegistry {
    /// Create a new registry with default managers
    pub fn new() -> Self {
        let mut registry = Self {
            managers: HashMap::new(),
        };

        // Register default managers
        registry.register(Arc::new(ChocolateyManager::new()));
        registry.register(Arc::new(WingetManager::new()));
        registry.register(Arc::new(HomebrewManager::new()));
        registry.register(Arc::new(AptManager::new()));

        registry
    }

    /// Register a package manager
    pub fn register(&mut self, manager: Arc<dyn SystemPackageManager>) {
        self.managers.insert(manager.name().to_string(), manager);
    }

    /// Get a package manager by name
    pub fn get(&self, name: &str) -> Result<Arc<dyn SystemPackageManager>> {
        self.managers
            .get(name)
            .cloned()
            .ok_or_else(|| SystemPmError::PackageManagerNotFound(name.to_string()))
    }

    /// Get all registered package managers
    pub fn all(&self) -> Vec<Arc<dyn SystemPackageManager>> {
        self.managers.values().cloned().collect()
    }

    /// Get all package managers supported on the current platform
    pub fn for_current_platform(&self) -> Vec<Arc<dyn SystemPackageManager>> {
        self.managers
            .values()
            .filter(|m| m.is_current_platform_supported())
            .cloned()
            .collect()
    }

    /// Get the preferred package manager for the current platform
    pub async fn get_preferred(&self) -> Option<Arc<dyn SystemPackageManager>> {
        let mut available: Vec<_> = Vec::new();

        for manager in self.for_current_platform() {
            if manager.is_installed().await {
                available.push(manager);
            }
        }

        // Sort by priority (descending)
        available.sort_by(|a, b| b.priority().cmp(&a.priority()));
        available.into_iter().next()
    }

    /// Get all available (installed) package managers
    pub async fn get_available(&self) -> Vec<Arc<dyn SystemPackageManager>> {
        let mut available = Vec::new();

        for manager in self.for_current_platform() {
            if manager.is_installed().await {
                available.push(manager);
            }
        }

        // Sort by priority (descending)
        available.sort_by(|a, b| b.priority().cmp(&a.priority()));
        available
    }

    /// Check if any package manager is available
    pub async fn has_available(&self) -> bool {
        for manager in self.for_current_platform() {
            if manager.is_installed().await {
                return true;
            }
        }
        false
    }
}

impl Default for PackageManagerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = PackageManagerRegistry::new();
        assert!(registry.get("choco").is_ok());
        assert!(registry.get("winget").is_ok());
        assert!(registry.get("brew").is_ok());
        assert!(registry.get("apt").is_ok());
    }

    #[test]
    fn test_unknown_manager() {
        let registry = PackageManagerRegistry::new();
        assert!(registry.get("unknown").is_err());
    }

    #[test]
    fn test_for_current_platform() {
        let registry = PackageManagerRegistry::new();
        let managers = registry.for_current_platform();

        #[cfg(windows)]
        {
            assert!(managers.iter().any(|m| m.name() == "choco"));
            assert!(managers.iter().any(|m| m.name() == "winget"));
        }

        #[cfg(target_os = "macos")]
        {
            assert!(managers.iter().any(|m| m.name() == "brew"));
        }

        #[cfg(target_os = "linux")]
        {
            assert!(managers.iter().any(|m| m.name() == "apt"));
            assert!(managers.iter().any(|m| m.name() == "brew"));
        }
    }
}
