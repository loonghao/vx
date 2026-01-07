//! Info command implementation - shows system information and capabilities
//!
//! Provides AI-friendly capability discovery for vx.
//! Returns structured information about available runtimes, system tools, and features.

use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use vx_paths::{PathManager, PathResolver};
use vx_runtime::{Ecosystem, Platform, ProviderRegistry};

/// Output format for capabilities command
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilitiesFormat {
    Json,
    Text,
}

/// Platform information
#[derive(Debug, Serialize)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
}

impl PlatformInfo {
    pub fn current() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
        }
    }
}

/// Runtime information for capabilities output
#[derive(Debug, Serialize)]
pub struct RuntimeInfo {
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub installed: bool,
    pub ecosystem: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_constraint: Option<String>,
}

/// System tool information
#[derive(Debug, Serialize)]
pub struct SystemToolInfo {
    pub name: String,
    pub category: String,
    pub platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Feature flags
#[derive(Debug, Serialize)]
pub struct Features {
    pub auto_install: bool,
    pub shell_mode: bool,
    pub project_config: bool,
    pub extensions: bool,
    pub virtual_environments: bool,
}

impl Default for Features {
    fn default() -> Self {
        Self {
            auto_install: true,
            shell_mode: true,
            project_config: true,
            extensions: true,
            virtual_environments: true,
        }
    }
}

/// Complete capabilities response
#[derive(Debug, Serialize)]
pub struct Capabilities {
    pub version: String,
    pub platform: PlatformInfo,
    pub runtimes: HashMap<String, RuntimeInfo>,
    pub system_tools: SystemTools,
    pub features: Features,
}

/// System tools categorized by availability
#[derive(Debug, Serialize)]
pub struct SystemTools {
    pub available: Vec<SystemToolInfo>,
    pub unavailable: Vec<SystemToolInfo>,
}

/// Handle the capabilities command
pub async fn handle(registry: &ProviderRegistry, json: bool) -> Result<()> {
    let format = if json {
        CapabilitiesFormat::Json
    } else {
        CapabilitiesFormat::Text
    };

    let capabilities = gather_capabilities(registry).await?;

    match format {
        CapabilitiesFormat::Json => {
            let json_output = serde_json::to_string_pretty(&capabilities)?;
            println!("{}", json_output);
        }
        CapabilitiesFormat::Text => {
            print_text_format(&capabilities);
        }
    }

    Ok(())
}

/// Gather all capabilities information
async fn gather_capabilities(registry: &ProviderRegistry) -> Result<Capabilities> {
    let current_platform = Platform::current();

    // Create path manager and resolver for checking installed tools
    let path_manager = PathManager::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize path manager: {}", e))?;
    let resolver = PathResolver::new(path_manager);

    // Get installed tools
    let installed_tools = resolver
        .get_installed_tools_with_versions()
        .unwrap_or_default();
    let installed_set: std::collections::HashSet<_> =
        installed_tools.iter().map(|(name, _)| name.as_str()).collect();

    // Gather runtime information
    let mut runtimes = HashMap::new();
    let mut system_tools_available = Vec::new();
    let mut system_tools_unavailable = Vec::new();

    for runtime_name in registry.runtime_names() {
        if let Some(runtime) = registry.get_runtime(&runtime_name) {
            let is_supported = runtime.is_platform_supported(&current_platform);
            let ecosystem = runtime.ecosystem();
            let is_installed = installed_set.contains(runtime_name.as_str());

            // Get version if installed
            let version = if is_installed {
                installed_tools
                    .iter()
                    .find(|(name, _)| name == &runtime_name)
                    .and_then(|(_, versions)| versions.first().cloned())
            } else {
                None
            };

            // Get platform constraint description
            let platform_constraint = if !is_supported {
                Some(format!("{} only", get_platform_label(&runtime_name, registry)))
            } else {
                None
            };

            // Categorize as runtime or system tool
            if ecosystem == Ecosystem::System {
                let category = get_tool_category(&runtime_name);
                let tool_info = SystemToolInfo {
                    name: runtime_name.clone(),
                    category,
                    platform: if is_supported {
                        "universal".to_string()
                    } else {
                        get_platform_label(&runtime_name, registry)
                    },
                    path: if is_installed {
                        resolver
                            .find_latest_executable(&runtime_name)
                            .ok()
                            .flatten()
                            .map(|p| p.display().to_string())
                    } else {
                        // Check system PATH
                        which::which(&runtime_name)
                            .ok()
                            .map(|p| p.display().to_string())
                    },
                    reason: if !is_supported {
                        Some(format!("Only available on {}", get_platform_label(&runtime_name, registry)))
                    } else {
                        None
                    },
                };

                if is_supported {
                    system_tools_available.push(tool_info);
                } else {
                    system_tools_unavailable.push(tool_info);
                }
            } else {
                // Regular runtime
                let runtime_info = RuntimeInfo {
                    name: runtime_name.clone(),
                    description: runtime.description().to_string(),
                    version,
                    installed: is_installed,
                    ecosystem: ecosystem.to_string(),
                    commands: get_runtime_commands(&runtime_name, registry),
                    platform_constraint,
                };
                runtimes.insert(runtime_name, runtime_info);
            }
        }
    }

    Ok(Capabilities {
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: PlatformInfo::current(),
        runtimes,
        system_tools: SystemTools {
            available: system_tools_available,
            unavailable: system_tools_unavailable,
        },
        features: Features::default(),
    })
}

/// Get platform label for a runtime
fn get_platform_label(runtime_name: &str, _registry: &ProviderRegistry) -> String {
    // Try to get from manifest
    if let Some(label) = crate::registry::get_runtime_platform_label(runtime_name) {
        return label;
    }

    // Default based on known tools
    match runtime_name {
        "msvc" | "cl" | "nmake" | "msbuild" | "devenv" | "choco" | "rcedit" => "Windows".to_string(),
        "xcodebuild" | "xcrun" | "swift" | "swiftc" | "xcode-select" => "macOS".to_string(),
        "systemctl" | "journalctl" | "apt" | "apt-get" => "Linux".to_string(),
        _ => "universal".to_string(),
    }
}

/// Get tool category based on runtime name
fn get_tool_category(runtime_name: &str) -> String {
    match runtime_name {
        // Build tools
        "cmake" | "make" | "ninja" | "msbuild" | "xcodebuild" => "build",
        // Compilers
        "cl" | "gcc" | "clang" | "swift" | "swiftc" => "compiler",
        // Security
        "openssl" | "gpg" | "codesign" | "signtool" => "security",
        // Network
        "curl" | "wget" | "ssh" | "scp" => "network",
        // System
        "systemctl" | "journalctl" | "launchctl" => "system",
        // Package managers
        "choco" | "brew" | "apt" | "apt-get" | "winget" => "package",
        // Version control
        "git" | "svn" | "hg" => "vcs",
        // Container
        "docker" | "podman" | "kubectl" | "helm" => "container",
        // Cloud
        "aws" | "az" | "gcloud" => "cloud",
        _ => "other",
    }
    .to_string()
}

/// Get commands provided by a runtime
fn get_runtime_commands(runtime_name: &str, registry: &ProviderRegistry) -> Vec<String> {
    // Find all runtimes that are bundled with this one
    let mut commands = vec![runtime_name.to_string()];

    for name in registry.runtime_names() {
        if let Some(runtime) = registry.get_runtime(&name) {
            if let Some(bundled_with) = runtime.metadata().get("bundled_with") {
                if bundled_with == runtime_name && name != runtime_name {
                    commands.push(name);
                }
            }
        }
    }

    // Also add aliases
    if let Some(runtime) = registry.get_runtime(runtime_name) {
        for alias in runtime.aliases() {
            if !commands.contains(&alias.to_string()) {
                commands.push(alias.to_string());
            }
        }
    }

    commands
}

/// Print capabilities in text format
fn print_text_format(capabilities: &Capabilities) {
    use crate::ui::UI;

    UI::info(&format!("vx {} capabilities", capabilities.version));
    println!();

    // Platform
    println!(
        "Platform: {} ({})",
        capabilities.platform.os, capabilities.platform.arch
    );
    println!();

    // Runtimes by ecosystem
    UI::info("Managed Runtimes:");
    let mut by_ecosystem: HashMap<&str, Vec<&RuntimeInfo>> = HashMap::new();
    for runtime in capabilities.runtimes.values() {
        by_ecosystem
            .entry(&runtime.ecosystem)
            .or_default()
            .push(runtime);
    }

    for (ecosystem, runtimes) in by_ecosystem.iter() {
        println!("  {}:", ecosystem);
        for runtime in runtimes {
            let status = if runtime.installed { "✅" } else { "❌" };
            let version_str = runtime
                .version
                .as_ref()
                .map(|v| format!(" ({})", v))
                .unwrap_or_default();
            println!(
                "    {} {}{} - {}",
                status, runtime.name, version_str, runtime.description
            );
        }
    }
    println!();

    // System tools
    if !capabilities.system_tools.available.is_empty() {
        UI::info("System Tools (available):");
        for tool in &capabilities.system_tools.available {
            let path_str = tool
                .path
                .as_ref()
                .map(|p| format!(" @ {}", p))
                .unwrap_or_default();
            println!("    {} [{}]{}", tool.name, tool.category, path_str);
        }
        println!();
    }

    if !capabilities.system_tools.unavailable.is_empty() {
        UI::info("System Tools (unavailable on this platform):");
        for tool in &capabilities.system_tools.unavailable {
            let reason = tool
                .reason
                .as_ref()
                .map(|r| format!(" - {}", r))
                .unwrap_or_default();
            println!("    {} [{}]{}", tool.name, tool.platform, reason);
        }
        println!();
    }

    // Features
    UI::info("Features:");
    println!("    auto_install: {}", capabilities.features.auto_install);
    println!("    shell_mode: {}", capabilities.features.shell_mode);
    println!("    project_config: {}", capabilities.features.project_config);
    println!("    extensions: {}", capabilities.features.extensions);
    println!(
        "    virtual_environments: {}",
        capabilities.features.virtual_environments
    );
}
