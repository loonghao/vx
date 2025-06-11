//! Package manager trait and related types

use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Trait for package managers
#[async_trait::async_trait]
pub trait PackageManager: Send + Sync {
    /// Get the name of this package manager
    fn name(&self) -> &str;
    
    /// Get the ecosystem this package manager belongs to
    fn ecosystem(&self) -> Ecosystem;
    
    /// Install packages
    async fn install_packages(&self, packages: &[PackageSpec]) -> Result<()>;
    
    /// Remove packages
    async fn remove_packages(&self, packages: &[String]) -> Result<()>;
    
    /// List installed packages
    async fn list_packages(&self) -> Result<Vec<PackageInfo>>;
    
    /// Search for packages
    async fn search_packages(&self, query: &str) -> Result<Vec<PackageInfo>>;
    
    /// Update packages
    async fn update_packages(&self, packages: &[String]) -> Result<()>;
    
    /// Check if this package manager is available on the system
    async fn is_available(&self) -> Result<bool>;
    
    /// Check if this package manager is preferred for the given project
    fn is_preferred_for_project(&self, project_path: &Path) -> bool;
    
    /// Get package manager configuration
    fn get_config(&self) -> PackageManagerConfig;
    
    /// Run a package manager command
    async fn run_command(&self, args: &[String]) -> Result<i32>;
}

/// Package specification for installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSpec {
    pub name: String,
    pub version: Option<String>,
    pub source: Option<String>,
    pub dev_dependency: bool,
}

/// Information about a package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    pub dependencies: Vec<String>,
}

/// Ecosystem classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ecosystem {
    JavaScript,
    Python,
    Rust,
    Go,
    Ruby,
    PHP,
    DotNet,
    Java,
    System(SystemType),
    Scientific,
    VFX,
    Container,
    Other(String),
}

/// System type classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemType {
    Linux(LinuxDistro),
    MacOS,
    Windows,
    Unix,
}

/// Linux distribution types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinuxDistro {
    Ubuntu,
    Debian,
    CentOS,
    RHEL,
    Fedora,
    Arch,
    Alpine,
    Other(String),
}

/// Package manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManagerConfig {
    pub name: String,
    pub version: Option<String>,
    pub executable_path: Option<std::path::PathBuf>,
    pub config_files: Vec<std::path::PathBuf>,
    pub cache_directory: Option<std::path::PathBuf>,
    pub supports_lockfiles: bool,
    pub supports_workspaces: bool,
    pub isolation_level: IsolationLevel,
}

/// Environment isolation level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IsolationLevel {
    /// No isolation, global installation
    Global,
    /// User-level isolation
    User,
    /// Project-level isolation
    Project,
    /// Complete sandboxing
    Sandbox,
}

impl PackageSpec {
    /// Create a new package spec
    pub fn new(name: String) -> Self {
        Self {
            name,
            version: None,
            source: None,
            dev_dependency: false,
        }
    }
    
    /// Set version constraint
    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }
    
    /// Set as dev dependency
    pub fn as_dev_dependency(mut self) -> Self {
        self.dev_dependency = true;
        self
    }
    
    /// Set package source
    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }
}
