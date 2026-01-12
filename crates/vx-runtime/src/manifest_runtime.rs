//! Manifest-Driven Runtime implementation
//!
//! This module provides a Runtime implementation that is driven entirely by
//! provider.toml configuration files. It's designed for system tools that
//! don't require strict version management (git, cmake, curl, etc.).
//!
//! # Design Goals
//!
//! 1. **Zero Rust code for system tools** - All configuration in TOML
//! 2. **User-extensible** - Users can add their own tools via ~/.vx/providers/
//! 3. **System package manager integration** - Leverage choco, winget, brew, apt
//! 4. **Fallback strategies** - Multiple installation methods with priorities

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;

use crate::{Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};

/// Source of a provider
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderSource {
    /// Built-in provider (compiled into vx)
    BuiltIn,
    /// User local provider (~/.vx/providers/)
    UserLocal(PathBuf),
    /// Environment variable specified path
    EnvPath(PathBuf),
}

impl std::fmt::Display for ProviderSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderSource::BuiltIn => write!(f, "built-in"),
            ProviderSource::UserLocal(p) => write!(f, "{}", p.display()),
            ProviderSource::EnvPath(p) => write!(f, "{} (env)", p.display()),
        }
    }
}

/// A runtime driven by manifest configuration (provider.toml)
///
/// This is used for system tools that don't require strict version management.
/// The runtime is entirely configured via TOML, with no Rust code needed.
#[derive(Debug, Clone)]
pub struct ManifestDrivenRuntime {
    /// Runtime name
    pub name: String,
    /// Description
    pub description: String,
    /// Executable name
    pub executable: String,
    /// Aliases
    pub aliases: Vec<String>,
    /// Provider name
    pub provider_name: String,
    /// Provider source
    pub source: ProviderSource,
    /// System installation strategies
    pub install_strategies: Vec<InstallStrategy>,
    /// Tools provided by this runtime
    pub provides: Vec<ProvidedTool>,
    /// Detection configuration
    pub detection: Option<DetectionConfig>,
    /// System dependencies
    pub system_deps: Option<SystemDepsConfig>,
}

/// Installation strategy for system tools
#[derive(Debug, Clone)]
pub enum InstallStrategy {
    /// Use a system package manager
    PackageManager {
        /// Package manager name (choco, winget, brew, apt, etc.)
        manager: String,
        /// Package name
        package: String,
        /// Installation parameters (Chocolatey --params)
        params: Option<String>,
        /// Native installer arguments
        install_args: Option<String>,
        /// Priority (higher = preferred)
        priority: i32,
        /// Platform filter
        platforms: Vec<String>,
    },
    /// Direct download
    DirectDownload {
        /// URL template (supports {version}, {platform}, {arch})
        url: String,
        /// Archive format (zip, tar.gz, etc.)
        format: Option<String>,
        /// Executable path within archive
        executable_path: Option<String>,
        /// Priority
        priority: i32,
        /// Platform filter
        platforms: Vec<String>,
    },
    /// Run a script
    Script {
        /// Script URL
        url: String,
        /// Script type (powershell, bash, cmd)
        script_type: ScriptType,
        /// Script arguments
        args: Vec<String>,
        /// Priority
        priority: i32,
        /// Platform filter
        platforms: Vec<String>,
    },
    /// Provided by another runtime
    ProvidedBy {
        /// Provider runtime name
        provider: String,
        /// Relative path to executable
        relative_path: String,
        /// Priority
        priority: i32,
        /// Platform filter
        platforms: Vec<String>,
    },
}

impl InstallStrategy {
    /// Get the priority of this strategy
    pub fn priority(&self) -> i32 {
        match self {
            InstallStrategy::PackageManager { priority, .. } => *priority,
            InstallStrategy::DirectDownload { priority, .. } => *priority,
            InstallStrategy::Script { priority, .. } => *priority,
            InstallStrategy::ProvidedBy { priority, .. } => *priority,
        }
    }

    /// Check if this strategy matches the current platform
    pub fn matches_platform(&self, platform: &Platform) -> bool {
        let platforms = match self {
            InstallStrategy::PackageManager { platforms, .. } => platforms,
            InstallStrategy::DirectDownload { platforms, .. } => platforms,
            InstallStrategy::Script { platforms, .. } => platforms,
            InstallStrategy::ProvidedBy { platforms, .. } => platforms,
        };

        if platforms.is_empty() {
            return true; // No filter = all platforms
        }

        let current = platform.os_name();
        platforms.iter().any(|p| p.eq_ignore_ascii_case(current))
    }
}

/// Script type for installation
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptType {
    PowerShell,
    Bash,
    Cmd,
}

/// Tool provided by a runtime
#[derive(Debug, Clone)]
pub struct ProvidedTool {
    /// Tool name
    pub name: String,
    /// Relative path to executable
    pub relative_path: String,
    /// Supported platforms
    pub platforms: Vec<String>,
}

/// Detection configuration for version detection
#[derive(Debug, Clone)]
pub struct DetectionConfig {
    /// Command to run (e.g., "{executable} --version")
    pub command: String,
    /// Regex pattern to extract version
    pub pattern: String,
    /// System paths to search
    pub system_paths: Vec<String>,
    /// Environment variable hints
    pub env_hints: Vec<String>,
}

/// System dependencies configuration
#[derive(Debug, Clone, Default)]
pub struct SystemDepsConfig {
    /// Pre-installation dependencies
    pub pre_depends: Vec<SystemDependency>,
    /// Runtime dependencies
    pub depends: Vec<SystemDependency>,
    /// Recommended dependencies
    pub recommends: Vec<SystemDependency>,
    /// Optional dependencies
    pub suggests: Vec<SystemDependency>,
}

/// A system-level dependency
#[derive(Debug, Clone)]
pub struct SystemDependency {
    /// Dependency type
    pub dep_type: SystemDepType,
    /// Dependency identifier
    pub id: String,
    /// Version constraint
    pub version: Option<String>,
    /// Reason for dependency
    pub reason: Option<String>,
    /// Platform filter
    pub platforms: Vec<String>,
    /// Whether this is optional
    pub optional: bool,
}

/// Type of system dependency
#[derive(Debug, Clone, PartialEq)]
pub enum SystemDepType {
    /// Windows KB update
    WindowsKb,
    /// Windows Feature (DISM)
    WindowsFeature,
    /// Visual C++ Redistributable
    VcRedist,
    /// .NET Framework / Runtime
    DotNet,
    /// System package
    Package,
    /// Another vx runtime
    Runtime,
}

impl ManifestDrivenRuntime {
    /// Create a new manifest-driven runtime
    pub fn new(
        name: impl Into<String>,
        provider_name: impl Into<String>,
        source: ProviderSource,
    ) -> Self {
        let name = name.into();
        Self {
            executable: name.clone(),
            name,
            description: String::new(),
            aliases: Vec::new(),
            provider_name: provider_name.into(),
            source,
            install_strategies: Vec::new(),
            provides: Vec::new(),
            detection: None,
            system_deps: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set executable name
    pub fn with_executable(mut self, executable: impl Into<String>) -> Self {
        self.executable = executable.into();
        self
    }

    /// Add an alias
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    /// Add an installation strategy
    pub fn with_strategy(mut self, strategy: InstallStrategy) -> Self {
        self.install_strategies.push(strategy);
        self
    }

    /// Set detection configuration
    pub fn with_detection(mut self, detection: DetectionConfig) -> Self {
        self.detection = Some(detection);
        self
    }

    /// Select the best available installation strategy
    pub async fn select_best_strategy(&self, platform: &Platform) -> Option<&InstallStrategy> {
        let mut candidates: Vec<_> = self
            .install_strategies
            .iter()
            .filter(|s| s.matches_platform(platform))
            .collect();

        // Sort by priority (descending)
        candidates.sort_by(|a, b| b.priority().cmp(&a.priority()));

        // Return the first available strategy
        for strategy in candidates {
            if self.is_strategy_available(strategy).await {
                return Some(strategy);
            }
        }

        None
    }

    /// Check if a strategy is available on the current system
    async fn is_strategy_available(&self, strategy: &InstallStrategy) -> bool {
        match strategy {
            InstallStrategy::PackageManager { manager, .. } => {
                // Check if the package manager is installed
                is_package_manager_available(manager).await
            }
            InstallStrategy::DirectDownload { .. } => true,
            InstallStrategy::Script { .. } => true,
            InstallStrategy::ProvidedBy { provider, .. } => {
                // Check if the provider runtime is installed
                which::which(provider).is_ok()
            }
        }
    }

    /// Detect the installed version using the detection configuration
    pub async fn detect_version(&self) -> Result<Option<String>> {
        let detection = match &self.detection {
            Some(d) => d,
            None => return Ok(None),
        };

        // Find the executable
        let executable_path = match which::which(&self.executable) {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        // Build the command
        let command = detection.command.replace("{executable}", &self.executable);
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(None);
        }

        // Execute the command
        let output = tokio::process::Command::new(&executable_path)
            .args(&parts[1..])
            .output()
            .await?;

        if !output.status.success() {
            return Ok(None);
        }

        // Parse the output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);

        // Extract version using regex
        let re = regex::Regex::new(&detection.pattern)?;
        if let Some(captures) = re.captures(&combined) {
            if let Some(version) = captures.get(1) {
                return Ok(Some(version.as_str().to_string()));
            }
        }

        Ok(None)
    }
}

#[async_trait]
impl Runtime for ManifestDrivenRuntime {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        if self.description.is_empty() {
            "System tool"
        } else {
            &self.description
        }
    }

    fn aliases(&self) -> &[&str] {
        // This is a limitation - we can't return borrowed slices from owned Vec
        // In practice, this method might need to be redesigned
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("provider".to_string(), self.provider_name.clone());
        meta.insert("source".to_string(), self.source.to_string());
        meta.insert("manifest_driven".to_string(), "true".to_string());
        meta
    }

    /// For manifest-driven runtimes, we return "system" as the only version
    /// since we don't manage versions - we use whatever the system package manager installs
    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        Ok(vec![VersionInfo {
            version: "system".to_string(),
            released_at: None,
            prerelease: false,
            lts: true,
            download_url: None,
            checksum: None,
            metadata: HashMap::new(),
        }])
    }

    /// Check if the tool is installed on the system
    async fn is_installed(&self, _version: &str, _ctx: &RuntimeContext) -> Result<bool> {
        Ok(which::which(&self.executable).is_ok())
    }

    /// Get installed versions
    async fn installed_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<String>> {
        if which::which(&self.executable).is_ok() {
            // Try to detect the version
            if let Ok(Some(version)) = self.detect_version().await {
                return Ok(vec![version]);
            }
            Ok(vec!["system".to_string()])
        } else {
            Ok(vec![])
        }
    }

    /// Get download URL (if direct download is available)
    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        for strategy in &self.install_strategies {
            if let InstallStrategy::DirectDownload { url, platforms, .. } = strategy {
                if platforms.is_empty()
                    || platforms
                        .iter()
                        .any(|p| p.eq_ignore_ascii_case(platform.os_name()))
                {
                    // Substitute version in URL
                    let url = url.replace("{version}", version);
                    return Ok(Some(url));
                }
            }
        }
        Ok(None)
    }
}

/// Check if a package manager is available on the system
async fn is_package_manager_available(manager: &str) -> bool {
    match manager {
        "choco" | "chocolatey" => which::which("choco").is_ok(),
        "winget" => which::which("winget").is_ok(),
        "scoop" => which::which("scoop").is_ok(),
        "brew" | "homebrew" => which::which("brew").is_ok(),
        "apt" | "apt-get" => which::which("apt").is_ok() || which::which("apt-get").is_ok(),
        "yum" => which::which("yum").is_ok(),
        "dnf" => which::which("dnf").is_ok(),
        "pacman" => which::which("pacman").is_ok(),
        "zypper" => which::which("zypper").is_ok(),
        "apk" => which::which("apk").is_ok(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_strategy_priority() {
        let strategy = InstallStrategy::PackageManager {
            manager: "choco".to_string(),
            package: "git".to_string(),
            params: None,
            install_args: None,
            priority: 80,
            platforms: vec!["windows".to_string()],
        };

        assert_eq!(strategy.priority(), 80);
    }

    #[test]
    fn test_install_strategy_platform_filter() {
        let strategy = InstallStrategy::PackageManager {
            manager: "brew".to_string(),
            package: "git".to_string(),
            params: None,
            install_args: None,
            priority: 90,
            platforms: vec!["macos".to_string(), "linux".to_string()],
        };

        let macos = Platform::new(crate::Os::MacOS, crate::Arch::Aarch64);
        let windows = Platform::new(crate::Os::Windows, crate::Arch::X86_64);

        assert!(strategy.matches_platform(&macos));
        assert!(!strategy.matches_platform(&windows));
    }

    #[test]
    fn test_manifest_runtime_builder() {
        let runtime = ManifestDrivenRuntime::new("fd", "mytools", ProviderSource::BuiltIn)
            .with_description("A simple, fast alternative to find")
            .with_executable("fd")
            .with_alias("fd-find")
            .with_strategy(InstallStrategy::PackageManager {
                manager: "brew".to_string(),
                package: "fd".to_string(),
                params: None,
                install_args: None,
                priority: 90,
                platforms: vec![],
            });

        assert_eq!(runtime.name(), "fd");
        assert_eq!(runtime.description(), "A simple, fast alternative to find");
        assert_eq!(runtime.install_strategies.len(), 1);
    }
}
