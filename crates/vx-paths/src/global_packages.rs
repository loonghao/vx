//! Global Package Management (RFC 0025)
//!
//! This module provides data structures and utilities for managing globally
//! installed packages across different package ecosystems (npm, pip, cargo, go, gem).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A globally installed package
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlobalPackage {
    /// Package name
    pub name: String,
    /// Installed version
    pub version: String,
    /// Ecosystem identifier (npm, pip, cargo, go, gem)
    pub ecosystem: String,
    /// Installation timestamp (ISO 8601)
    pub installed_at: String,
    /// Executables provided by this package
    pub executables: Vec<String>,
    /// Runtime dependency (e.g., node@20 for npm packages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_dependency: Option<RuntimeDependency>,
    /// Installation directory
    pub install_dir: PathBuf,
}

impl GlobalPackage {
    /// Create a new GlobalPackage
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        ecosystem: impl Into<String>,
        install_dir: PathBuf,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            ecosystem: ecosystem.into(),
            installed_at: chrono::Utc::now().to_rfc3339(),
            executables: Vec::new(),
            runtime_dependency: None,
            install_dir,
        }
    }

    /// Add an executable to this package
    pub fn with_executable(mut self, exe: impl Into<String>) -> Self {
        self.executables.push(exe.into());
        self
    }

    /// Add multiple executables to this package
    pub fn with_executables(mut self, exes: Vec<String>) -> Self {
        self.executables.extend(exes);
        self
    }

    /// Set the runtime dependency
    pub fn with_runtime_dependency(
        mut self,
        runtime: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        self.runtime_dependency = Some(RuntimeDependency {
            runtime: runtime.into(),
            version: version.into(),
        });
        self
    }

    /// Get the unique key for this package (ecosystem:name)
    pub fn key(&self) -> String {
        format!("{}:{}", self.ecosystem, self.name)
    }
}

/// Runtime dependency for a package
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeDependency {
    /// Runtime name (e.g., "node", "python")
    pub runtime: String,
    /// Required version (e.g., "20", "3.11")
    pub version: String,
}

/// Registry of installed global packages
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageRegistry {
    /// Map of package key (ecosystem:name) to GlobalPackage
    packages: HashMap<String, GlobalPackage>,
    /// Map of executable name to package key
    #[serde(default)]
    executable_index: HashMap<String, String>,
}

impl PackageRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            executable_index: HashMap::new(),
        }
    }

    /// Load registry from a file
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read registry file: {}", path.display()))?;

        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse registry file: {}", path.display()))
    }

    /// Load registry from file or create new if doesn't exist
    pub fn load_or_create(path: &Path) -> Result<Self> {
        Self::load(path)
    }

    /// Get all packages as an iterator
    pub fn all_packages(&self) -> impl Iterator<Item = &GlobalPackage> {
        self.packages.values()
    }

    /// Save registry to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self).context("Failed to serialize registry")?;

        std::fs::write(path, content)
            .with_context(|| format!("Failed to write registry file: {}", path.display()))
    }

    /// Register a new package
    pub fn register(&mut self, package: GlobalPackage) {
        let key = package.key();

        // Update executable index
        for exe in &package.executables {
            self.executable_index.insert(exe.clone(), key.clone());
        }

        self.packages.insert(key, package);
    }

    /// Unregister a package
    pub fn unregister(&mut self, ecosystem: &str, name: &str) -> Option<GlobalPackage> {
        let key = format!("{}:{}", ecosystem, name);

        if let Some(package) = self.packages.remove(&key) {
            // Remove from executable index
            for exe in &package.executables {
                self.executable_index.remove(exe);
            }
            Some(package)
        } else {
            None
        }
    }

    /// Get a package by ecosystem and name
    pub fn get(&self, ecosystem: &str, name: &str) -> Option<&GlobalPackage> {
        let key = format!("{}:{}", ecosystem, name);
        self.packages.get(&key)
    }

    /// Find a package by executable name
    pub fn find_by_executable(&self, exe_name: &str) -> Option<&GlobalPackage> {
        self.executable_index
            .get(exe_name)
            .and_then(|key| self.packages.get(key))
    }

    /// List all packages
    pub fn list(&self) -> impl Iterator<Item = &GlobalPackage> {
        self.packages.values()
    }

    /// List packages by ecosystem
    pub fn list_by_ecosystem(&self, ecosystem: &str) -> impl Iterator<Item = &GlobalPackage> {
        let ecosystem = ecosystem.to_lowercase();
        self.packages
            .values()
            .filter(move |pkg| pkg.ecosystem.to_lowercase() == ecosystem)
    }

    /// Check if a package is registered
    pub fn contains(&self, ecosystem: &str, name: &str) -> bool {
        let key = format!("{}:{}", ecosystem, name);
        self.packages.contains_key(&key)
    }

    /// Get the number of registered packages
    pub fn len(&self) -> usize {
        self.packages.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    /// Rebuild the executable index from packages
    pub fn rebuild_index(&mut self) {
        self.executable_index.clear();
        for (key, package) in &self.packages {
            for exe in &package.executables {
                self.executable_index.insert(exe.clone(), key.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_package_creation() {
        let pkg = GlobalPackage::new("typescript", "5.3.3", "npm", PathBuf::from("/tmp/pkg"))
            .with_executable("tsc")
            .with_executable("tsserver");

        assert_eq!(pkg.name, "typescript");
        assert_eq!(pkg.version, "5.3.3");
        assert_eq!(pkg.ecosystem, "npm");
        assert_eq!(pkg.executables, vec!["tsc", "tsserver"]);
        assert_eq!(pkg.key(), "npm:typescript");
    }

    #[test]
    fn test_global_package_with_runtime() {
        let pkg = GlobalPackage::new("typescript", "5.3.3", "npm", PathBuf::from("/tmp/pkg"))
            .with_runtime_dependency("node", "20");

        assert!(pkg.runtime_dependency.is_some());
        let dep = pkg.runtime_dependency.unwrap();
        assert_eq!(dep.runtime, "node");
        assert_eq!(dep.version, "20");
    }

    #[test]
    fn test_package_registry() {
        let mut registry = PackageRegistry::new();

        let pkg = GlobalPackage::new("typescript", "5.3.3", "npm", PathBuf::from("/tmp/pkg"))
            .with_executable("tsc")
            .with_executable("tsserver");

        registry.register(pkg);

        assert_eq!(registry.len(), 1);
        assert!(registry.contains("npm", "typescript"));
        assert!(registry.get("npm", "typescript").is_some());
    }

    #[test]
    fn test_find_by_executable() {
        let mut registry = PackageRegistry::new();

        let pkg = GlobalPackage::new("typescript", "5.3.3", "npm", PathBuf::from("/tmp/pkg"))
            .with_executable("tsc")
            .with_executable("tsserver");

        registry.register(pkg);

        let found = registry.find_by_executable("tsc");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "typescript");

        let found = registry.find_by_executable("tsserver");
        assert!(found.is_some());

        let not_found = registry.find_by_executable("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_list_by_ecosystem() {
        let mut registry = PackageRegistry::new();

        registry.register(GlobalPackage::new(
            "typescript",
            "5.3.3",
            "npm",
            PathBuf::from("/tmp/ts"),
        ));
        registry.register(GlobalPackage::new(
            "eslint",
            "8.56.0",
            "npm",
            PathBuf::from("/tmp/eslint"),
        ));
        registry.register(GlobalPackage::new(
            "black",
            "24.1.0",
            "pip",
            PathBuf::from("/tmp/black"),
        ));

        let npm_packages: Vec<_> = registry.list_by_ecosystem("npm").collect();
        assert_eq!(npm_packages.len(), 2);

        let pip_packages: Vec<_> = registry.list_by_ecosystem("pip").collect();
        assert_eq!(pip_packages.len(), 1);
    }

    #[test]
    fn test_unregister() {
        let mut registry = PackageRegistry::new();

        let pkg = GlobalPackage::new("typescript", "5.3.3", "npm", PathBuf::from("/tmp/pkg"))
            .with_executable("tsc");

        registry.register(pkg);
        assert!(registry.find_by_executable("tsc").is_some());

        let removed = registry.unregister("npm", "typescript");
        assert!(removed.is_some());
        assert!(registry.find_by_executable("tsc").is_none());
        assert!(!registry.contains("npm", "typescript"));
    }
}
