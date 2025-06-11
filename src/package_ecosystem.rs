// Universal package management ecosystem abstraction
// Supports multiple package managers across different languages and platforms

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Universal package manager trait that can represent any package management system
pub trait UniversalPackageManager: Send + Sync {
    /// Get the name of this package manager
    fn name(&self) -> &str;

    /// Get the ecosystem this package manager belongs to
    fn ecosystem(&self) -> Ecosystem;

    /// Install packages
    fn install_packages(&self, packages: &[PackageSpec]) -> Result<()>;

    /// Remove packages
    fn remove_packages(&self, packages: &[String]) -> Result<()>;

    /// List installed packages
    fn list_packages(&self) -> Result<Vec<PackageInfo>>;

    /// Search for packages
    fn search_packages(&self, query: &str) -> Result<Vec<PackageInfo>>;

    /// Update packages
    fn update_packages(&self, packages: &[String]) -> Result<()>;

    /// Check if this package manager is available on the system
    fn is_available(&self) -> bool;

    /// Get package manager specific configuration
    fn get_config(&self) -> PackageManagerConfig;

    /// Check if this package manager is preferred for the given project
    fn is_preferred_for_project(&self, project_path: &PathBuf) -> bool;
}

/// Ecosystem classification for different package management domains
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ecosystem {
    /// JavaScript/Node.js ecosystem
    JavaScript,
    /// Python ecosystem
    Python,
    /// Rust ecosystem
    Rust,
    /// Go ecosystem
    Go,
    /// Ruby ecosystem
    Ruby,
    /// PHP ecosystem
    PHP,
    /// .NET ecosystem
    DotNet,
    /// System-level package managers
    System(SystemType),
    /// Scientific computing (HPC, research)
    Scientific,
    /// VFX/Animation industry
    VFX,
    /// Container/virtualization
    Container,
    /// Generic/other
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinuxDistro {
    Ubuntu,
    Debian,
    CentOS,
    RHEL,
    Fedora,
    Arch,
    SUSE,
    Alpine,
    Other(String),
}

/// Package specification for installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSpec {
    pub name: String,
    pub version: Option<String>,
    pub source: PackageSource,
    pub install_options: InstallOptions,
    pub dependencies: Vec<String>,
}

/// Package source specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageSource {
    /// Default registry (npm, PyPI, crates.io, etc.)
    Registry,
    /// Custom registry with URL
    CustomRegistry(String),
    /// Git repository
    Git(GitSource),
    /// Local file system
    Local(PathBuf),
    /// Direct URL download
    Url(String),
    /// System package repository
    SystemRepo(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSource {
    pub url: String,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub commit: Option<String>,
}

/// Installation options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstallOptions {
    pub dev_dependency: bool,
    pub global: bool,
    pub optional: bool,
    pub exact_version: bool,
    pub save_to_lockfile: bool,
    pub custom_flags: Vec<String>,
}

/// Package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    pub keywords: Vec<String>,
    pub dependencies: Vec<String>,
    pub is_dev_dependency: bool,
    pub is_optional: bool,
    pub install_size: Option<u64>,
}

/// Package manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManagerConfig {
    pub name: String,
    pub version: Option<String>,
    pub executable_path: Option<PathBuf>,
    pub config_files: Vec<PathBuf>,
    pub cache_directory: Option<PathBuf>,
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

/// Universal package ecosystem registry
pub struct PackageEcosystemRegistry {
    managers: HashMap<Ecosystem, Vec<Box<dyn UniversalPackageManager>>>,
    project_detector: ProjectDetector,
}

impl PackageEcosystemRegistry {
    pub fn new() -> Self {
        Self {
            managers: HashMap::new(),
            project_detector: ProjectDetector::new(),
        }
    }

    /// Register a package manager for an ecosystem
    pub fn register_manager(&mut self, manager: Box<dyn UniversalPackageManager>) {
        let ecosystem = manager.ecosystem();
        self.managers
            .entry(ecosystem)
            .or_insert_with(Vec::new)
            .push(manager);
    }

    /// Get all package managers for an ecosystem
    pub fn get_managers(
        &self,
        ecosystem: &Ecosystem,
    ) -> Option<&Vec<Box<dyn UniversalPackageManager>>> {
        self.managers.get(ecosystem)
    }

    /// Get the preferred package manager for a project
    pub fn get_preferred_manager(
        &self,
        project_path: &PathBuf,
    ) -> Option<&dyn UniversalPackageManager> {
        let project_type = self.project_detector.detect_project_type(project_path);

        for ecosystem in project_type.ecosystems() {
            if let Some(managers) = self.get_managers(&ecosystem) {
                for manager in managers {
                    if manager.is_preferred_for_project(project_path) {
                        return Some(manager.as_ref());
                    }
                }
                // Return first available manager if no preference found
                if let Some(manager) = managers.first() {
                    if manager.is_available() {
                        return Some(manager.as_ref());
                    }
                }
            }
        }

        None
    }

    /// Get a specific package manager by name
    pub fn get_manager_by_name(&self, name: &str) -> Option<&dyn UniversalPackageManager> {
        for managers in self.managers.values() {
            for manager in managers {
                if manager.name() == name {
                    return Some(manager.as_ref());
                }
            }
        }
        None
    }

    /// List all available ecosystems
    pub fn list_ecosystems(&self) -> Vec<Ecosystem> {
        self.managers.keys().cloned().collect()
    }
}

/// Project type detection
pub struct ProjectDetector {
    detectors: Vec<Box<dyn ProjectTypeDetector>>,
}

impl ProjectDetector {
    pub fn new() -> Self {
        Self {
            detectors: vec![
                Box::new(JavaScriptDetector),
                Box::new(PythonDetector),
                Box::new(RustDetector),
                Box::new(GoDetector),
                // Add more detectors as needed
            ],
        }
    }

    pub fn detect_project_type(&self, project_path: &PathBuf) -> ProjectType {
        let mut detected_types = Vec::new();

        for detector in &self.detectors {
            if let Some(project_type) = detector.detect(project_path) {
                detected_types.push(project_type);
            }
        }

        match detected_types.len() {
            0 => ProjectType::Unknown,
            1 => detected_types.into_iter().next().unwrap(),
            _ => ProjectType::Mixed(detected_types),
        }
    }
}

/// Project type classification
#[derive(Debug, Clone)]
pub enum ProjectType {
    JavaScript(JavaScriptProjectType),
    Python(PythonProjectType),
    Rust,
    Go,
    Ruby,
    PHP,
    DotNet,
    VFX(VFXProjectType),
    Scientific(ScientificProjectType),
    Mixed(Vec<ProjectType>),
    Unknown,
}

impl ProjectType {
    /// Get the ecosystems associated with this project type
    pub fn ecosystems(&self) -> Vec<Ecosystem> {
        match self {
            ProjectType::JavaScript(_) => vec![Ecosystem::JavaScript],
            ProjectType::Python(_) => vec![Ecosystem::Python],
            ProjectType::Rust => vec![Ecosystem::Rust],
            ProjectType::Go => vec![Ecosystem::Go],
            ProjectType::Ruby => vec![Ecosystem::Ruby],
            ProjectType::PHP => vec![Ecosystem::PHP],
            ProjectType::DotNet => vec![Ecosystem::DotNet],
            ProjectType::VFX(_) => vec![Ecosystem::VFX],
            ProjectType::Scientific(_) => vec![Ecosystem::Scientific],
            ProjectType::Mixed(types) => types.iter().flat_map(|t| t.ecosystems()).collect(),
            ProjectType::Unknown => vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub enum JavaScriptProjectType {
    Node,
    Deno,
    Bun,
}

#[derive(Debug, Clone)]
pub enum PythonProjectType {
    Standard,
    Conda,
    Poetry,
    Pipenv,
}

#[derive(Debug, Clone)]
pub enum VFXProjectType {
    Maya,
    Houdini,
    Nuke,
    Blender,
}

#[derive(Debug, Clone)]
pub enum ScientificProjectType {
    HPC,
    Research,
    DataScience,
}

/// Project type detector trait
pub trait ProjectTypeDetector {
    fn detect(&self, project_path: &PathBuf) -> Option<ProjectType>;
}

/// JavaScript project detector
pub struct JavaScriptDetector;

impl ProjectTypeDetector for JavaScriptDetector {
    fn detect(&self, project_path: &PathBuf) -> Option<ProjectType> {
        if project_path.join("package.json").exists() {
            Some(ProjectType::JavaScript(JavaScriptProjectType::Node))
        } else if project_path.join("deno.json").exists()
            || project_path.join("deno.jsonc").exists()
        {
            Some(ProjectType::JavaScript(JavaScriptProjectType::Deno))
        } else {
            None
        }
    }
}

/// Python project detector
pub struct PythonDetector;

impl ProjectTypeDetector for PythonDetector {
    fn detect(&self, project_path: &PathBuf) -> Option<ProjectType> {
        if project_path.join("pyproject.toml").exists() {
            Some(ProjectType::Python(PythonProjectType::Poetry))
        } else if project_path.join("Pipfile").exists() {
            Some(ProjectType::Python(PythonProjectType::Pipenv))
        } else if project_path.join("environment.yml").exists() {
            Some(ProjectType::Python(PythonProjectType::Conda))
        } else if project_path.join("requirements.txt").exists()
            || project_path.join("setup.py").exists()
        {
            Some(ProjectType::Python(PythonProjectType::Standard))
        } else {
            None
        }
    }
}

/// Rust project detector
pub struct RustDetector;

impl ProjectTypeDetector for RustDetector {
    fn detect(&self, project_path: &PathBuf) -> Option<ProjectType> {
        if project_path.join("Cargo.toml").exists() {
            Some(ProjectType::Rust)
        } else {
            None
        }
    }
}

/// Go project detector
pub struct GoDetector;

impl ProjectTypeDetector for GoDetector {
    fn detect(&self, project_path: &PathBuf) -> Option<ProjectType> {
        if project_path.join("go.mod").exists() {
            Some(ProjectType::Go)
        } else {
            None
        }
    }
}

impl Default for PackageEcosystemRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ProjectDetector {
    fn default() -> Self {
        Self::new()
    }
}
