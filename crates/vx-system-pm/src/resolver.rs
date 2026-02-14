//! System dependency resolver

use crate::Result;
use crate::dependency::{SystemDepType, SystemDependency};
use crate::detector::PackageManagerDetector;
use crate::registry::PackageManagerRegistry;
use crate::strategy::InstallStrategy;
use tracing::{debug, warn};

/// Dependency resolution result
#[derive(Debug)]
pub struct DependencyResolution {
    /// Dependencies to install (in order)
    pub to_install: Vec<ResolvedDependency>,
    /// Already satisfied dependencies
    pub satisfied: Vec<ResolvedDependency>,
    /// Unresolved dependencies
    pub unresolved: Vec<UnresolvedDependency>,
}

/// A resolved dependency with installation strategy
#[derive(Debug)]
pub struct ResolvedDependency {
    /// The dependency
    pub dep: SystemDependency,
    /// Installation strategy
    pub strategy: InstallStrategy,
    /// Currently installed version (if any)
    pub installed_version: Option<String>,
}

/// An unresolved dependency
#[derive(Debug)]
pub struct UnresolvedDependency {
    /// The dependency
    pub dep: SystemDependency,
    /// Reason for failure
    pub reason: String,
}

/// System dependency resolver
pub struct SystemDependencyResolver {
    /// Package manager registry
    registry: PackageManagerRegistry,
    /// Package manager detector
    detector: PackageManagerDetector,
}

impl SystemDependencyResolver {
    /// Create a new resolver
    pub fn new() -> Self {
        Self {
            registry: PackageManagerRegistry::new(),
            detector: PackageManagerDetector::new(),
        }
    }

    /// Create with custom registry
    pub fn with_registry(registry: PackageManagerRegistry) -> Self {
        Self {
            registry,
            detector: PackageManagerDetector::new(),
        }
    }

    /// Resolve dependencies
    pub async fn resolve(&mut self, deps: &[SystemDependency]) -> Result<DependencyResolution> {
        let mut to_install = Vec::new();
        let mut satisfied = Vec::new();
        let mut unresolved = Vec::new();

        for dep in deps {
            // Check platform condition
            if !dep.matches_current_platform() {
                debug!("Skipping {} - not for current platform", dep.id);
                continue;
            }

            // Check if already installed
            match self.check_installed(dep).await {
                Ok(InstallStatus::Installed(version)) => {
                    debug!("{} is already installed (version: {})", dep.id, version);
                    satisfied.push(ResolvedDependency {
                        dep: dep.clone(),
                        strategy: InstallStrategy::default(),
                        installed_version: Some(version),
                    });
                }
                Ok(InstallStatus::NotInstalled) => {
                    // Find installation strategy
                    match self.find_strategy(dep).await {
                        Some(strategy) => {
                            debug!("Will install {} using {:?}", dep.id, strategy);
                            to_install.push(ResolvedDependency {
                                dep: dep.clone(),
                                strategy,
                                installed_version: None,
                            });
                        }
                        None if !dep.optional => {
                            warn!(
                                "No installation strategy for required dependency: {}",
                                dep.id
                            );
                            unresolved.push(UnresolvedDependency {
                                dep: dep.clone(),
                                reason: "No installation strategy available".to_string(),
                            });
                        }
                        None => {
                            debug!("Skipping optional dependency: {}", dep.id);
                        }
                    }
                }
                Ok(InstallStatus::VersionMismatch {
                    installed,
                    required,
                }) => {
                    warn!(
                        "{} version mismatch: installed={}, required={}",
                        dep.id, installed, required
                    );
                    match self.find_strategy(dep).await {
                        Some(strategy) => {
                            to_install.push(ResolvedDependency {
                                dep: dep.clone(),
                                strategy,
                                installed_version: Some(installed),
                            });
                        }
                        None => {
                            unresolved.push(UnresolvedDependency {
                                dep: dep.clone(),
                                reason: format!(
                                    "Version {} doesn't satisfy {}, no upgrade strategy",
                                    installed, required
                                ),
                            });
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to check {} status: {}", dep.id, e);
                    if !dep.optional {
                        unresolved.push(UnresolvedDependency {
                            dep: dep.clone(),
                            reason: format!("Check failed: {}", e),
                        });
                    }
                }
            }
        }

        // Sort by dependency type priority
        self.sort_by_priority(&mut to_install);

        Ok(DependencyResolution {
            to_install,
            satisfied,
            unresolved,
        })
    }

    /// Check if a dependency is installed
    async fn check_installed(&self, dep: &SystemDependency) -> Result<InstallStatus> {
        match dep.dep_type {
            SystemDepType::WindowsKb => self.check_kb_installed(&dep.id).await,
            SystemDepType::VcRedist => self.check_vcredist_installed(&dep.id, &dep.version).await,
            SystemDepType::DotNet => self.check_dotnet_installed(&dep.id, &dep.version).await,
            SystemDepType::WindowsFeature => self.check_feature_installed(&dep.id).await,
            SystemDepType::Package => self.check_package_installed(&dep.id, &dep.version).await,
            SystemDepType::Runtime => {
                // Runtime dependencies are handled by vx-resolver
                Ok(InstallStatus::NotInstalled)
            }
        }
    }

    /// Check if a Windows KB update is installed
    #[cfg(windows)]
    async fn check_kb_installed(&self, kb_id: &str) -> Result<InstallStatus> {
        use std::process::Command;

        let kb_upper = kb_id.to_uppercase();
        let kb_query = if kb_upper.starts_with("KB") {
            kb_upper.clone()
        } else {
            format!("KB{}", kb_upper)
        };

        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "Get-HotFix -Id {} -ErrorAction SilentlyContinue | Select-Object -ExpandProperty HotFixID",
                    kb_query
                ),
            ])
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.trim().to_uppercase().contains(&kb_upper) {
                return Ok(InstallStatus::Installed(kb_query));
            }
        }

        Ok(InstallStatus::NotInstalled)
    }

    #[cfg(not(windows))]
    async fn check_kb_installed(&self, _kb_id: &str) -> Result<InstallStatus> {
        // KB updates are Windows-only
        Ok(InstallStatus::NotInstalled)
    }

    /// Check if VCRedist is installed
    #[cfg(windows)]
    async fn check_vcredist_installed(
        &self,
        _id: &str,
        version: &Option<String>,
    ) -> Result<InstallStatus> {
        use winreg::RegKey;
        use winreg::enums::*;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

        // Check both 32-bit and 64-bit uninstall keys
        let uninstall_paths = [
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
            r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ];

        for path in &uninstall_paths {
            if let Ok(uninstall_key) = hklm.open_subkey(path) {
                for key_name in uninstall_key.enum_keys().filter_map(|k| k.ok()) {
                    if let Ok(subkey) = uninstall_key.open_subkey(&key_name) {
                        let display_name: std::result::Result<String, _> =
                            subkey.get_value("DisplayName");
                        if let Ok(name) = display_name
                            && name.contains("Visual C++")
                            && name.contains("Redistributable")
                        {
                            // Check version if specified
                            if let Some(required) = version {
                                let installed_version: std::result::Result<String, _> =
                                    subkey.get_value("DisplayVersion");
                                if let Ok(ver) = installed_version
                                    && self.version_satisfies(&ver, required)
                                {
                                    return Ok(InstallStatus::Installed(ver));
                                }
                            } else {
                                return Ok(InstallStatus::Installed("installed".to_string()));
                            }
                        }
                    }
                }
            }
        }

        Ok(InstallStatus::NotInstalled)
    }

    #[cfg(not(windows))]
    async fn check_vcredist_installed(
        &self,
        _id: &str,
        _version: &Option<String>,
    ) -> Result<InstallStatus> {
        // VCRedist is Windows-only
        Ok(InstallStatus::NotInstalled)
    }

    /// Check if .NET is installed
    async fn check_dotnet_installed(
        &self,
        id: &str,
        version: &Option<String>,
    ) -> Result<InstallStatus> {
        use std::process::Command;

        // Try dotnet --list-runtimes
        let output = Command::new("dotnet").args(["--list-runtimes"]).output();

        if let Ok(output) = output
            && output.status.success()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse runtime list
            for line in stdout.lines() {
                if line.contains(id) {
                    // Extract version from line like "Microsoft.NETCore.App 8.0.0 [path]"
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let ver = parts[1];
                        if let Some(required) = version {
                            if self.version_satisfies(ver, required) {
                                return Ok(InstallStatus::Installed(ver.to_string()));
                            }
                        } else {
                            return Ok(InstallStatus::Installed(ver.to_string()));
                        }
                    }
                }
            }
        }

        Ok(InstallStatus::NotInstalled)
    }

    /// Check if a Windows feature is installed
    #[cfg(windows)]
    async fn check_feature_installed(&self, feature: &str) -> Result<InstallStatus> {
        use std::process::Command;

        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "(Get-WindowsOptionalFeature -Online -FeatureName {}).State",
                    feature
                ),
            ])
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.trim() == "Enabled" {
                return Ok(InstallStatus::Installed(feature.to_string()));
            }
        }

        Ok(InstallStatus::NotInstalled)
    }

    #[cfg(not(windows))]
    async fn check_feature_installed(&self, _feature: &str) -> Result<InstallStatus> {
        Ok(InstallStatus::NotInstalled)
    }

    /// Check if a package is installed
    async fn check_package_installed(
        &self,
        package: &str,
        version: &Option<String>,
    ) -> Result<InstallStatus> {
        // Try to find the package using available package managers
        for manager in self.registry.get_available().await {
            if let Ok(true) = manager.is_package_installed(package).await {
                if let Ok(Some(ver)) = manager.get_installed_version(package).await {
                    if let Some(required) = version {
                        if self.version_satisfies(&ver, required) {
                            return Ok(InstallStatus::Installed(ver));
                        } else {
                            return Ok(InstallStatus::VersionMismatch {
                                installed: ver,
                                required: required.clone(),
                            });
                        }
                    }
                    return Ok(InstallStatus::Installed(ver));
                }
                return Ok(InstallStatus::Installed("unknown".to_string()));
            }
        }

        // Also check if the command exists in PATH
        if which::which(package).is_ok() {
            return Ok(InstallStatus::Installed("system".to_string()));
        }

        Ok(InstallStatus::NotInstalled)
    }

    /// Find an installation strategy for a dependency
    async fn find_strategy(&mut self, dep: &SystemDependency) -> Option<InstallStrategy> {
        match dep.dep_type {
            SystemDepType::WindowsKb => {
                // KB updates via Chocolatey
                if self.detector.is_available("choco").await {
                    Some(InstallStrategy::PackageManager {
                        manager: "choco".to_string(),
                        package: dep.id.clone(),
                        params: None,
                        install_args: None,
                        priority: 80,
                    })
                } else {
                    None
                }
            }
            SystemDepType::VcRedist => {
                // VCRedist via winget or Chocolatey
                if self.detector.is_available("winget").await {
                    Some(InstallStrategy::PackageManager {
                        manager: "winget".to_string(),
                        package: "Microsoft.VCRedist.2015+.x64".to_string(),
                        params: None,
                        install_args: None,
                        priority: 90,
                    })
                } else if self.detector.is_available("choco").await {
                    Some(InstallStrategy::PackageManager {
                        manager: "choco".to_string(),
                        package: "vcredist140".to_string(),
                        params: None,
                        install_args: None,
                        priority: 80,
                    })
                } else {
                    // Direct download fallback
                    Some(InstallStrategy::DirectDownload {
                        url: "https://aka.ms/vs/17/release/vc_redist.x64.exe".to_string(),
                        format: None,
                        executable_path: None,
                        priority: 50,
                    })
                }
            }
            SystemDepType::DotNet => {
                // .NET via winget or direct download
                if self.detector.is_available("winget").await {
                    Some(InstallStrategy::PackageManager {
                        manager: "winget".to_string(),
                        package: format!("Microsoft.DotNet.Runtime.{}", dep.id),
                        params: None,
                        install_args: None,
                        priority: 90,
                    })
                } else {
                    Some(InstallStrategy::Script {
                        url: "https://dot.net/v1/dotnet-install.ps1".to_string(),
                        script_type: crate::strategy::ScriptType::PowerShell,
                        args: vec!["-Runtime".to_string(), dep.id.clone()],
                        priority: 70,
                    })
                }
            }
            SystemDepType::WindowsFeature => {
                // Windows features via DISM
                #[cfg(windows)]
                {
                    Some(InstallStrategy::Script {
                        url: String::new(), // Inline script
                        script_type: crate::strategy::ScriptType::PowerShell,
                        args: vec![format!(
                            "Enable-WindowsOptionalFeature -Online -FeatureName {} -All -NoRestart",
                            dep.id
                        )],
                        priority: 80,
                    })
                }
                #[cfg(not(windows))]
                {
                    None
                }
            }
            SystemDepType::Package => {
                // Use the best available package manager
                self.detector
                    .get_preferred()
                    .await
                    .map(|pm| InstallStrategy::PackageManager {
                        manager: pm,
                        package: dep.id.clone(),
                        params: None,
                        install_args: None,
                        priority: 70,
                    })
            }
            SystemDepType::Runtime => {
                // Runtime dependencies are handled by vx-resolver
                None
            }
        }
    }

    /// Sort dependencies by installation priority
    fn sort_by_priority(&self, deps: &mut [ResolvedDependency]) {
        deps.sort_by(|a, b| {
            // Install order: KB updates > Features > VCRedist > DotNet > Packages
            let priority_a = self.dep_type_priority(&a.dep.dep_type);
            let priority_b = self.dep_type_priority(&b.dep.dep_type);
            priority_b.cmp(&priority_a)
        });
    }

    fn dep_type_priority(&self, dep_type: &SystemDepType) -> i32 {
        match dep_type {
            SystemDepType::WindowsKb => 100,
            SystemDepType::WindowsFeature => 90,
            SystemDepType::VcRedist => 80,
            SystemDepType::DotNet => 70,
            SystemDepType::Package => 50,
            SystemDepType::Runtime => 40,
        }
    }

    /// Check if a version satisfies a constraint
    fn version_satisfies(&self, version: &str, constraint: &str) -> bool {
        // Simple version comparison
        // TODO: Use proper semver parsing
        if constraint.starts_with(">=") {
            let required = constraint.trim_start_matches(">=").trim();
            version >= required
        } else if constraint.starts_with('>') {
            let required = constraint.trim_start_matches('>').trim();
            version > required
        } else if constraint.starts_with("<=") {
            let required = constraint.trim_start_matches("<=").trim();
            version <= required
        } else if constraint.starts_with('<') {
            let required = constraint.trim_start_matches('<').trim();
            version < required
        } else if constraint.starts_with('=') {
            let required = constraint.trim_start_matches('=').trim();
            version == required
        } else {
            // Exact match or any
            constraint == "*" || version.starts_with(constraint)
        }
    }
}

impl Default for SystemDependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Installation status
#[derive(Debug)]
enum InstallStatus {
    /// Installed with version
    Installed(String),
    /// Not installed
    NotInstalled,
    /// Version mismatch
    VersionMismatch { installed: String, required: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_satisfies() {
        let resolver = SystemDependencyResolver::new();

        assert!(resolver.version_satisfies("14.0.0", ">=14.0"));
        assert!(resolver.version_satisfies("15.0.0", ">=14.0"));
        assert!(!resolver.version_satisfies("13.0.0", ">=14.0"));

        assert!(resolver.version_satisfies("1.0.0", "*"));
        assert!(resolver.version_satisfies("14.0.0", "14.0"));
    }

    #[tokio::test]
    async fn test_resolver_creation() {
        let _resolver = SystemDependencyResolver::new();
        // Just ensure it creates without panicking
    }
}
