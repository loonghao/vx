//! Execution Plan - the intermediate representation between pipeline stages
//!
//! The `ExecutionPlan` is the output of the Resolve stage and input to subsequent stages.
//! It captures all resolved information needed to install, prepare, and execute a runtime.

use std::collections::HashMap;
use std::path::PathBuf;

/// Execution plan - the resolved blueprint for running a command.
///
/// Produced by ResolveStage, consumed by EnsureStage → PrepareStage → ExecuteStage.
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// The primary runtime to execute
    pub primary: PlannedRuntime,

    /// Dependency runtimes (topologically sorted, deps first)
    pub dependencies: Vec<PlannedRuntime>,

    /// Extra runtimes injected via `--with` flag
    pub injected: Vec<PlannedRuntime>,

    /// Proxy runtime configuration (RFC 0028)
    pub proxy: Option<ProxyConfig>,

    /// Execution configuration
    pub config: ExecutionConfig,
}

impl ExecutionPlan {
    /// Create a minimal execution plan for a single runtime
    pub fn new(primary: PlannedRuntime, config: ExecutionConfig) -> Self {
        Self {
            primary,
            dependencies: Vec::new(),
            injected: Vec::new(),
            proxy: None,
            config,
        }
    }

    /// Add a dependency runtime
    pub fn with_dependency(mut self, dep: PlannedRuntime) -> Self {
        self.dependencies.push(dep);
        self
    }

    /// Add an injected runtime (--with)
    pub fn with_injected(mut self, runtime: PlannedRuntime) -> Self {
        self.injected.push(runtime);
        self
    }

    /// Set proxy configuration
    pub fn with_proxy(mut self, proxy: ProxyConfig) -> Self {
        self.proxy = Some(proxy);
        self
    }

    /// Iterate over all runtimes that need installation (deps → primary → injected)
    pub fn all_runtimes(&self) -> impl Iterator<Item = &PlannedRuntime> {
        self.dependencies
            .iter()
            .chain(std::iter::once(&self.primary))
            .chain(self.injected.iter())
    }

    /// Iterate mutably over all runtimes
    pub fn all_runtimes_mut(&mut self) -> impl Iterator<Item = &mut PlannedRuntime> {
        self.dependencies
            .iter_mut()
            .chain(std::iter::once(&mut self.primary))
            .chain(self.injected.iter_mut())
    }

    /// Check if any runtime needs installation
    pub fn needs_install(&self) -> bool {
        self.all_runtimes()
            .any(|r| matches!(r.status, InstallStatus::NeedsInstall))
    }

    /// Get all runtimes that need installation
    pub fn runtimes_needing_install(&self) -> Vec<&PlannedRuntime> {
        self.all_runtimes()
            .filter(|r| matches!(r.status, InstallStatus::NeedsInstall))
            .collect()
    }

    /// Check if any runtime is unsupported on the current platform
    pub fn unsupported_runtimes(&self) -> Vec<&PlannedRuntime> {
        self.all_runtimes()
            .filter(|r| matches!(r.status, InstallStatus::PlatformUnsupported { .. }))
            .collect()
    }
}

/// A runtime resolved for execution within the pipeline.
///
/// This is different from `ResolutionResult` — it captures both the resolution
/// outcome and the installation status in a single, stage-friendly structure.
#[derive(Debug, Clone)]
pub struct PlannedRuntime {
    /// Runtime name (e.g., "node", "npm", "go")
    pub name: String,

    /// How the version was resolved
    pub version: VersionResolution,

    /// Current installation status
    pub status: InstallStatus,

    /// Path to the executable (populated after Ensure stage)
    pub executable: Option<PathBuf>,

    /// Installation directory (populated after Ensure stage)
    pub install_dir: Option<PathBuf>,
}

impl PlannedRuntime {
    /// Create a new planned runtime
    pub fn new(name: impl Into<String>, version: VersionResolution) -> Self {
        let status = match &version {
            VersionResolution::Installed { .. } => InstallStatus::Installed,
            VersionResolution::NeedsInstall { .. } => InstallStatus::NeedsInstall,
            _ => InstallStatus::NeedsInstall,
        };

        Self {
            name: name.into(),
            version,
            status,
            executable: None,
            install_dir: None,
        }
    }

    /// Create an already-installed runtime
    pub fn installed(name: impl Into<String>, version: String, executable: PathBuf) -> Self {
        Self {
            name: name.into(),
            version: VersionResolution::Installed {
                version: version.clone(),
                source: VersionSource::VxManaged,
            },
            status: InstallStatus::Installed,
            executable: Some(executable),
            install_dir: None,
        }
    }

    /// Create a runtime that needs installation
    pub fn needs_install(name: impl Into<String>, version: String) -> Self {
        Self {
            name: name.into(),
            version: VersionResolution::NeedsInstall { version },
            status: InstallStatus::NeedsInstall,
            executable: None,
            install_dir: None,
        }
    }

    /// Create a platform-unsupported runtime
    pub fn unsupported(name: impl Into<String>, reason: String) -> Self {
        Self {
            name: name.into(),
            version: VersionResolution::Unresolved,
            status: InstallStatus::PlatformUnsupported { reason },
            executable: None,
            install_dir: None,
        }
    }

    /// Mark this runtime as installed with the given executable path
    pub fn mark_installed(&mut self, executable: PathBuf) {
        self.status = InstallStatus::Installed;
        self.executable = Some(executable);
    }

    /// Get the version string (if resolved)
    pub fn version_string(&self) -> Option<&str> {
        match &self.version {
            VersionResolution::Installed { version, .. } => Some(version),
            VersionResolution::NeedsInstall { version } => Some(version),
            VersionResolution::LatestInstalled { version } => Some(version),
            VersionResolution::LatestRemote { version } => Some(version),
            VersionResolution::Range { resolved, .. } => Some(resolved),
            VersionResolution::SystemAvailable { version, .. } => version.as_deref(),
            VersionResolution::Unresolved => None,
        }
    }

    /// Whether the runtime is ready to execute (installed with a known executable)
    pub fn is_ready(&self) -> bool {
        self.status == InstallStatus::Installed && self.executable.is_some()
    }
}

/// How a version was resolved.
///
/// This captures the *source* and *method* of version resolution,
/// which is important for debugging and caching.
#[derive(Debug, Clone, PartialEq)]
pub enum VersionResolution {
    /// A specific version that is already installed
    Installed {
        version: String,
        source: VersionSource,
    },

    /// A specific version that needs to be installed
    NeedsInstall { version: String },

    /// The latest installed version was selected
    LatestInstalled { version: String },

    /// The latest remote version was selected (requires network)
    LatestRemote { version: String },

    /// A range constraint was resolved to a specific version
    Range { spec: String, resolved: String },

    /// Available via system PATH (not vx-managed)
    SystemAvailable {
        path: PathBuf,
        version: Option<String>,
    },

    /// Version could not be resolved
    Unresolved,
}

/// Where a version was sourced from
#[derive(Debug, Clone, PartialEq)]
pub enum VersionSource {
    /// Explicit command-line argument (e.g., `vx node@20.0.0`)
    Explicit,
    /// Project configuration (vx.toml)
    ProjectConfig,
    /// Legacy config file (.nvmrc, .node-version, etc.)
    LegacyConfig { file: String },
    /// User default (~/.vx/config.toml)
    UserDefault,
    /// Installed latest version
    InstalledLatest,
    /// Remote latest version
    RemoteLatest,
    /// vx-managed installation
    VxManaged,
    /// System PATH
    System,
}

/// Installation status of a runtime
#[derive(Debug, Clone, PartialEq)]
pub enum InstallStatus {
    /// Already installed and ready
    Installed,

    /// Needs to be installed
    NeedsInstall,

    /// Needs a dependency to be installed first
    NeedsDependency { dependency: String },

    /// Not supported on the current platform
    PlatformUnsupported { reason: String },
}

/// Proxy runtime configuration (RFC 0028)
///
/// Some runtimes (like `vite`, `jest`) are not standalone executables
/// but are invoked via a host runtime (like `node`, `npx`).
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// The host runtime that will actually execute (e.g., "npx")
    pub host_runtime: String,

    /// The package to invoke (e.g., "vite")
    pub package: String,

    /// Optional version constraint for the package
    pub version: Option<String>,

    /// Additional arguments to prepend
    pub prepend_args: Vec<String>,
}

/// Execution configuration passed through the pipeline
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Command-line arguments to pass to the runtime
    pub args: Vec<String>,

    /// Working directory for execution
    pub working_dir: Option<PathBuf>,

    /// Additional environment variables
    pub extra_env: HashMap<String, String>,

    /// Whether to inherit vx-managed tools PATH in subprocesses
    pub inherit_vx_path: bool,

    /// Whether to inherit full parent environment
    pub inherit_parent_env: bool,

    /// Whether auto-install is enabled
    pub auto_install: bool,

    /// Whether to show progress during installation
    pub show_progress: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            args: Vec::new(),
            working_dir: None,
            extra_env: HashMap::new(),
            inherit_vx_path: true,
            inherit_parent_env: false,
            auto_install: true,
            show_progress: true,
        }
    }
}

impl ExecutionConfig {
    /// Create config from command-line arguments
    pub fn with_args(args: Vec<String>) -> Self {
        Self {
            args,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_plan_new() {
        let primary = PlannedRuntime::installed(
            "node",
            "20.0.0".to_string(),
            PathBuf::from("/usr/local/bin/node"),
        );
        let config = ExecutionConfig::with_args(vec!["--version".to_string()]);

        let plan = ExecutionPlan::new(primary, config);

        assert_eq!(plan.primary.name, "node");
        assert!(plan.dependencies.is_empty());
        assert!(plan.injected.is_empty());
        assert!(plan.proxy.is_none());
        assert!(!plan.needs_install());
    }

    #[test]
    fn test_execution_plan_with_dependencies() {
        let primary = PlannedRuntime::needs_install("npm", "10.0.0".to_string());
        let node_dep = PlannedRuntime::installed(
            "node",
            "20.0.0".to_string(),
            PathBuf::from("/usr/local/bin/node"),
        );

        let plan =
            ExecutionPlan::new(primary, ExecutionConfig::default()).with_dependency(node_dep);

        assert_eq!(plan.dependencies.len(), 1);
        assert_eq!(plan.dependencies[0].name, "node");
        assert!(plan.needs_install());
        assert_eq!(plan.runtimes_needing_install().len(), 1);
        assert_eq!(plan.runtimes_needing_install()[0].name, "npm");
    }

    #[test]
    fn test_planned_runtime_installed() {
        let rt = PlannedRuntime::installed(
            "node",
            "20.0.0".to_string(),
            PathBuf::from("/usr/local/bin/node"),
        );

        assert_eq!(rt.name, "node");
        assert_eq!(rt.version_string(), Some("20.0.0"));
        assert!(rt.is_ready());
        assert_eq!(rt.status, InstallStatus::Installed);
    }

    #[test]
    fn test_planned_runtime_needs_install() {
        let rt = PlannedRuntime::needs_install("go", "1.21.0".to_string());

        assert_eq!(rt.name, "go");
        assert_eq!(rt.version_string(), Some("1.21.0"));
        assert!(!rt.is_ready());
        assert_eq!(rt.status, InstallStatus::NeedsInstall);
    }

    #[test]
    fn test_planned_runtime_unsupported() {
        let rt = PlannedRuntime::unsupported("msvc", "Windows only".to_string());

        assert_eq!(rt.name, "msvc");
        assert!(rt.version_string().is_none());
        assert!(!rt.is_ready());
        assert!(matches!(
            rt.status,
            InstallStatus::PlatformUnsupported { .. }
        ));
    }

    #[test]
    fn test_planned_runtime_mark_installed() {
        let mut rt = PlannedRuntime::needs_install("node", "20.0.0".to_string());
        assert!(!rt.is_ready());

        rt.mark_installed(PathBuf::from("/home/user/.vx/store/node/20.0.0/bin/node"));
        assert!(rt.is_ready());
        assert_eq!(rt.status, InstallStatus::Installed);
    }

    #[test]
    fn test_version_resolution_variants() {
        let installed = VersionResolution::Installed {
            version: "20.0.0".to_string(),
            source: VersionSource::Explicit,
        };
        assert_ne!(installed, VersionResolution::Unresolved);

        let range = VersionResolution::Range {
            spec: "^20.0.0".to_string(),
            resolved: "20.5.1".to_string(),
        };
        assert_ne!(range, installed);
    }

    #[test]
    fn test_execution_plan_unsupported_runtimes() {
        let primary = PlannedRuntime::installed(
            "node",
            "20.0.0".to_string(),
            PathBuf::from("/usr/local/bin/node"),
        );
        let unsupported = PlannedRuntime::unsupported("msvc", "Windows only".to_string());

        let plan =
            ExecutionPlan::new(primary, ExecutionConfig::default()).with_injected(unsupported);

        assert_eq!(plan.unsupported_runtimes().len(), 1);
        assert_eq!(plan.unsupported_runtimes()[0].name, "msvc");
    }

    #[test]
    fn test_all_runtimes_iterator() {
        let primary = PlannedRuntime::installed(
            "npm",
            "10.0.0".to_string(),
            PathBuf::from("/usr/local/bin/npm"),
        );
        let dep = PlannedRuntime::installed(
            "node",
            "20.0.0".to_string(),
            PathBuf::from("/usr/local/bin/node"),
        );
        let injected = PlannedRuntime::installed(
            "yarn",
            "4.0.0".to_string(),
            PathBuf::from("/usr/local/bin/yarn"),
        );

        let plan = ExecutionPlan::new(primary, ExecutionConfig::default())
            .with_dependency(dep)
            .with_injected(injected);

        let names: Vec<&str> = plan.all_runtimes().map(|r| r.name.as_str()).collect();
        // Order: deps → primary → injected
        assert_eq!(names, vec!["node", "npm", "yarn"]);
    }
}
