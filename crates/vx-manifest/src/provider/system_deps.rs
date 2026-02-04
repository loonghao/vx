//! System dependencies configuration for provider.toml
//!
//! This module defines the schema for system-level dependencies like
//! Windows KB updates, VCRedist, .NET Framework, and system packages.

use serde::{Deserialize, Serialize};

/// System dependency definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemDependencyDef {
    /// Dependency type
    #[serde(rename = "type")]
    pub dep_type: SystemDepTypeDef,

    /// Dependency identifier (e.g., "kb2919355", "vcredist140", "git")
    pub id: String,

    /// Version constraint (optional, semver syntax)
    #[serde(default)]
    pub version: Option<String>,

    /// Reason for this dependency
    #[serde(default)]
    pub reason: Option<String>,

    /// Platform conditions (e.g., ["windows"], ["linux", "macos"])
    #[serde(default)]
    pub platforms: Vec<String>,

    /// Whether this dependency is optional
    #[serde(default)]
    pub optional: bool,
}

/// System dependency types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemDepTypeDef {
    /// Windows KB update (e.g., KB2919355)
    WindowsKb,

    /// Windows Feature (DISM feature)
    WindowsFeature,

    /// Visual C++ Redistributable
    VcRedist,

    /// .NET Framework or .NET Runtime
    DotNet,

    /// System package (installed via package manager)
    Package,

    /// Another vx-managed runtime
    Runtime,
}

/// System dependencies configuration for a runtime
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SystemDepsConfigDef {
    /// Pre-dependencies (must be satisfied before installation)
    #[serde(default)]
    pub pre_depends: Vec<SystemDependencyDef>,

    /// Runtime dependencies
    #[serde(default)]
    pub depends: Vec<SystemDependencyDef>,

    /// Recommended dependencies
    #[serde(default)]
    pub recommends: Vec<SystemDependencyDef>,

    /// Optional/suggested dependencies
    #[serde(default)]
    pub suggests: Vec<SystemDependencyDef>,
}

/// Installation strategy definition
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InstallStrategyDef {
    /// Use a system package manager
    PackageManager {
        /// Package manager name (choco, winget, brew, apt, etc.)
        manager: String,
        /// Package name
        package: String,
        /// Installation parameters (Chocolatey --params)
        #[serde(default)]
        params: Option<String>,
        /// Native installer arguments (Chocolatey --install-arguments)
        #[serde(default)]
        install_args: Option<String>,
        /// Priority (higher = preferred)
        #[serde(default = "default_priority")]
        priority: i32,
    },

    /// Direct download from URL
    DirectDownload {
        /// URL template (supports {version}, {platform}, {arch})
        url: String,
        /// Archive format (tar.gz, zip, etc.)
        #[serde(default)]
        format: Option<String>,
        /// Path to executable within archive
        #[serde(default)]
        executable_path: Option<String>,
        /// Priority
        #[serde(default = "default_priority")]
        priority: i32,
    },

    /// Run an installation script
    Script {
        /// Script URL
        url: String,
        /// Script type
        script_type: ScriptTypeDef,
        /// Script arguments
        #[serde(default)]
        args: Vec<String>,
        /// Priority
        #[serde(default = "default_priority")]
        priority: i32,
    },

    /// Tool is provided by another runtime
    ProvidedBy {
        /// Provider runtime name
        provider: String,
        /// Relative path to the tool within provider's installation
        relative_path: String,
        /// Priority
        #[serde(default = "default_priority")]
        priority: i32,
    },

    /// Bundled command - tool is executed via another runtime (e.g., `dotnet nuget`)
    BundledCommand {
        /// Parent runtime that provides this command
        parent_runtime: String,
        /// Command prefix to use when executing (e.g., ["dotnet", "nuget"])
        command_prefix: Vec<String>,
        /// Supported platforms
        #[serde(default)]
        platforms: Vec<String>,
        /// Priority
        #[serde(default = "default_priority")]
        priority: i32,
    },

    /// Manual installation required (display instructions to user)
    Manual {
        /// Human-readable installation instructions
        #[serde(default)]
        instructions: String,
        /// Priority
        #[serde(default = "default_priority")]
        priority: i32,
    },
}

fn default_priority() -> i32 {
    50
}

/// Script types for installation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptTypeDef {
    /// PowerShell script (.ps1)
    PowerShell,
    /// Bash script (.sh)
    Bash,
    /// Windows batch script (.cmd, .bat)
    Cmd,
}

/// System installation configuration for a runtime
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SystemInstallConfigDef {
    /// Installation strategies (ordered by priority)
    #[serde(default)]
    pub strategies: Vec<InstallStrategyDef>,

    /// Tools provided by this runtime
    #[serde(default)]
    pub provides: Vec<ProvidedToolDef>,
}

/// A tool provided by another runtime
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProvidedToolDef {
    /// Tool name
    pub name: String,

    /// Relative path to the tool
    pub relative_path: String,

    /// Supported platforms
    #[serde(default)]
    pub platforms: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_dependency_deserialize() {
        let toml = r#"
            type = "vc_redist"
            id = "vcredist140"
            version = ">=14.0"
            reason = "Required for C++ runtime"
            platforms = ["windows"]
        "#;

        let dep: SystemDependencyDef = toml::from_str(toml).unwrap();
        assert_eq!(dep.dep_type, SystemDepTypeDef::VcRedist);
        assert_eq!(dep.id, "vcredist140");
        assert_eq!(dep.version, Some(">=14.0".to_string()));
    }

    #[test]
    fn test_install_strategy_deserialize() {
        let toml = r#"
            type = "package_manager"
            manager = "choco"
            package = "git"
            params = "/GitAndUnixToolsOnPath"
            priority = 80
        "#;

        let strategy: InstallStrategyDef = toml::from_str(toml).unwrap();
        match strategy {
            InstallStrategyDef::PackageManager {
                manager,
                package,
                params,
                priority,
                ..
            } => {
                assert_eq!(manager, "choco");
                assert_eq!(package, "git");
                assert_eq!(params, Some("/GitAndUnixToolsOnPath".to_string()));
                assert_eq!(priority, 80);
            }
            _ => panic!("Expected PackageManager strategy"),
        }
    }

    #[test]
    fn test_system_deps_config_deserialize() {
        let toml = r#"
            [[pre_depends]]
            type = "windows_kb"
            id = "kb2919355"
            platforms = ["windows"]

            [[depends]]
            type = "vc_redist"
            id = "vcredist140"
            version = ">=14.0"
        "#;

        let config: SystemDepsConfigDef = toml::from_str(toml).unwrap();
        assert_eq!(config.pre_depends.len(), 1);
        assert_eq!(config.depends.len(), 1);
        assert_eq!(config.pre_depends[0].dep_type, SystemDepTypeDef::WindowsKb);
        assert_eq!(config.depends[0].dep_type, SystemDepTypeDef::VcRedist);
    }

    #[test]
    fn test_bundled_command_strategy_deserialize() {
        let toml = r#"
            type = "bundled_command"
            parent_runtime = "dotnet"
            command_prefix = ["dotnet", "nuget"]
            platforms = ["macos", "linux"]
            priority = 100
        "#;

        let strategy: InstallStrategyDef = toml::from_str(toml).unwrap();
        match strategy {
            InstallStrategyDef::BundledCommand {
                parent_runtime,
                command_prefix,
                platforms,
                priority,
            } => {
                assert_eq!(parent_runtime, "dotnet");
                assert_eq!(command_prefix, vec!["dotnet", "nuget"]);
                assert_eq!(platforms, vec!["macos", "linux"]);
                assert_eq!(priority, 100);
            }
            _ => panic!("Expected BundledCommand strategy"),
        }
    }
}
