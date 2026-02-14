//! Executable Layout Configuration (RFC 0019)
//!
//! This module provides declarative configuration for handling various executable file layouts:
//! - Single binary files (e.g., yasm-1.3.0-win64.exe → bin/yasm.exe)
//! - Archives with nested directories (e.g., ffmpeg-6.0/bin/ffmpeg.exe)
//! - Platform-specific layouts
//!
//! ## Usage
//!
//! ```toml
//! [runtimes.layout]
//! download_type = "binary"
//!
//! [runtimes.layout.binary."windows-x86_64"]
//! source_name = "tool-{version}-win64.exe"
//! target_name = "tool.exe"
//! target_dir = "bin"
//! ```

use crate::{Os, Platform};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Executable layout configuration from provider.toml
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutableLayout {
    /// Download type (binary or archive)
    pub download_type: DownloadType,

    /// Binary-specific configuration
    #[serde(default)]
    pub binary: Option<HashMap<String, BinaryLayout>>,

    /// Archive-specific configuration
    #[serde(default)]
    pub archive: Option<ArchiveLayout>,

    /// MSI-specific configuration (Windows only)
    #[serde(default)]
    pub msi: Option<HashMap<String, MsiLayout>>,

    /// Platform-specific configurations
    #[serde(default)]
    pub windows: Option<PlatformLayout>,
    #[serde(default)]
    pub macos: Option<PlatformLayout>,
    #[serde(default)]
    pub linux: Option<PlatformLayout>,
}

/// Download type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadType {
    /// Single binary file (e.g., yasm.exe)
    Binary,
    /// Archive file (tar.gz, zip, etc.)
    Archive,
    /// MSI installer (Windows only)
    Msi,
}

/// Binary layout configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BinaryLayout {
    /// Source filename template (supports variables: {version}, {name}, etc.)
    pub source_name: String,
    /// Target filename after installation
    pub target_name: String,
    /// Target directory relative to install root
    #[serde(default = "default_bin_dir")]
    pub target_dir: String,
    /// Unix file permissions (e.g., "755")
    #[serde(default)]
    pub target_permissions: Option<String>,
}

/// Archive layout configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArchiveLayout {
    /// Executable paths inside the archive (supports multiple)
    pub executable_paths: Vec<String>,
    /// Prefix to strip when extracting
    #[serde(default)]
    pub strip_prefix: Option<String>,
    /// Unix permissions for extracted files
    #[serde(default)]
    pub permissions: Option<String>,
}

/// MSI layout configuration (Windows only)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MsiLayout {
    /// Download URL template (supports variables: {version}, {name}, etc.)
    pub url_template: String,
    /// Expected executable paths after MSI extraction
    #[serde(default)]
    pub executable_paths: Option<Vec<String>>,
}

/// Platform-specific layout
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlatformLayout {
    /// Executable paths
    pub executable_paths: Vec<String>,
    /// Prefix to strip
    #[serde(default)]
    pub strip_prefix: Option<String>,
    /// Unix permissions
    #[serde(default)]
    pub permissions: Option<String>,
}

/// Context for resolving layout variables
#[derive(Debug, Clone)]
pub struct LayoutContext {
    /// Version string (e.g., "1.3.0")
    pub version: String,
    /// Runtime name (e.g., "yasm")
    pub name: String,
    /// Target platform
    pub platform: Platform,
}

/// Resolved layout after variable interpolation
#[derive(Debug, Clone)]
pub enum ResolvedLayout {
    /// Binary file layout
    Binary {
        /// Original downloaded filename
        source_name: String,
        /// Target filename
        target_name: String,
        /// Target directory
        target_dir: String,
        /// Unix permissions
        permissions: Option<String>,
    },
    /// Archive layout
    Archive {
        /// Possible executable paths
        executable_paths: Vec<String>,
        /// Prefix to strip
        strip_prefix: Option<String>,
        /// Permissions
        permissions: Option<String>,
    },
}

impl ExecutableLayout {
    /// Resolve layout with context variables
    pub fn resolve(&self, ctx: &LayoutContext) -> Result<ResolvedLayout> {
        let vars = build_variables(ctx);

        match self.download_type {
            DownloadType::Binary => self.resolve_binary(&vars, ctx),
            DownloadType::Archive => self.resolve_archive(&vars, ctx),
            DownloadType::Msi => self.resolve_msi(&vars, ctx),
        }
    }

    fn resolve_binary(
        &self,
        vars: &HashMap<String, String>,
        ctx: &LayoutContext,
    ) -> Result<ResolvedLayout> {
        let binary_configs = self
            .binary
            .as_ref()
            .ok_or_else(|| anyhow!("Missing binary configuration for download_type = 'binary'"))?;

        // Try to find platform-specific config
        let platform_key = format!("{}-{}", ctx.platform.os, ctx.platform.arch);
        let config = binary_configs
            .get(&platform_key)
            .or_else(|| binary_configs.get(&ctx.platform.os.to_string()))
            .or_else(|| binary_configs.values().next())
            .ok_or_else(|| {
                anyhow!(
                    "No binary configuration found for platform: {}",
                    platform_key
                )
            })?;

        Ok(ResolvedLayout::Binary {
            source_name: interpolate(&config.source_name, vars),
            target_name: interpolate(&config.target_name, vars),
            target_dir: interpolate(&config.target_dir, vars),
            permissions: config.target_permissions.clone(),
        })
    }

    fn resolve_archive(
        &self,
        vars: &HashMap<String, String>,
        ctx: &LayoutContext,
    ) -> Result<ResolvedLayout> {
        // Get platform-specific layout or fallback to default archive config
        let layout = self.get_platform_layout(&ctx.platform.os)?;

        Ok(ResolvedLayout::Archive {
            executable_paths: layout
                .executable_paths
                .iter()
                .map(|p| interpolate(p, vars))
                .collect(),
            strip_prefix: layout.strip_prefix.as_ref().map(|p| interpolate(p, vars)),
            permissions: layout.permissions.clone(),
        })
    }

    fn get_platform_layout(&self, os: &Os) -> Result<PlatformLayout> {
        let platform_layout = match os {
            Os::Windows => self.windows.as_ref(),
            Os::MacOS => self.macos.as_ref(),
            Os::Linux => self.linux.as_ref(),
            _ => None,
        };

        // If platform-specific layout exists, use it
        if let Some(layout) = platform_layout {
            return Ok(layout.clone());
        }

        // Otherwise, use default archive config
        self.archive
            .as_ref()
            .map(|a| PlatformLayout {
                executable_paths: a.executable_paths.clone(),
                strip_prefix: a.strip_prefix.clone(),
                permissions: a.permissions.clone(),
            })
            .ok_or_else(|| anyhow!("No layout configuration found for OS: {:?}", os))
    }

    fn resolve_msi(
        &self,
        vars: &HashMap<String, String>,
        ctx: &LayoutContext,
    ) -> Result<ResolvedLayout> {
        // First try to get MSI-specific configuration
        if let Some(msi_configs) = &self.msi {
            let platform_key = format!("{}-{}", ctx.platform.os, ctx.platform.arch);

            if let Some(config) = msi_configs
                .get(&platform_key)
                .or_else(|| msi_configs.get(&ctx.platform.os.to_string()))
                .or_else(|| msi_configs.values().next())
            {
                // Use executable_paths from MSI config if available
                if let Some(exe_paths) = &config.executable_paths {
                    return Ok(ResolvedLayout::Archive {
                        executable_paths: exe_paths.iter().map(|p| interpolate(p, vars)).collect(),
                        strip_prefix: None,
                        permissions: None,
                    });
                }
            }
        }

        // Fallback: try platform-specific layout or archive config
        // MSI extraction typically places files in a specific structure
        // Try to use windows layout if available
        if let Some(windows_layout) = &self.windows {
            return Ok(ResolvedLayout::Archive {
                executable_paths: windows_layout
                    .executable_paths
                    .iter()
                    .map(|p| interpolate(p, vars))
                    .collect(),
                strip_prefix: windows_layout
                    .strip_prefix
                    .as_ref()
                    .map(|p| interpolate(p, vars)),
                permissions: windows_layout.permissions.clone(),
            });
        }

        // If no specific config, use a sensible default for MSI
        // MSI typically extracts to Program Files structure
        let name = &ctx.name;
        Ok(ResolvedLayout::Archive {
            executable_paths: vec![
                format!("{}.exe", name),
                format!("bin/{}.exe", name),
                format!("Amazon/AWSCLIV2/{}.exe", name), // AWS CLI specific
                "**/*.exe".to_string(),                  // Glob pattern fallback
            ],
            strip_prefix: None,
            permissions: None,
        })
    }
}

impl ResolvedLayout {
    /// Get the expected executable path after installation
    pub fn executable_path(&self, install_root: &Path) -> PathBuf {
        match self {
            ResolvedLayout::Binary {
                target_name,
                target_dir,
                ..
            } => install_root.join(target_dir).join(target_name),
            ResolvedLayout::Archive {
                executable_paths, ..
            } => {
                // Return the first path that exists
                if let Some(found) = executable_paths
                    .iter()
                    .map(|p| install_root.join(p))
                    .find(|p| p.exists())
                {
                    return found;
                }

                // No path exists (e.g., called before installation or with empty root).
                // Use platform-aware fallback: on non-Windows, prefer paths without .exe
                let platform = Platform::current();
                if platform.os != Os::Windows
                    && let Some(non_exe) = executable_paths
                        .iter()
                        .find(|p| !p.ends_with(".exe") && !p.ends_with(".cmd"))
                {
                    return install_root.join(non_exe);
                }

                // Final fallback: first path
                install_root.join(&executable_paths[0])
            }
        }
    }

    /// Get all possible executable paths
    pub fn all_executable_paths(&self, install_root: &Path) -> Vec<PathBuf> {
        match self {
            ResolvedLayout::Binary {
                target_name,
                target_dir,
                ..
            } => vec![install_root.join(target_dir).join(target_name)],
            ResolvedLayout::Archive {
                executable_paths, ..
            } => executable_paths
                .iter()
                .map(|p| install_root.join(p))
                .collect(),
        }
    }

    /// Check if any expected executable exists
    pub fn verify(&self, install_root: &Path) -> Result<PathBuf> {
        let paths = self.all_executable_paths(install_root);

        for path in &paths {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        Err(anyhow!(
            "Executable not found at any expected path:\n{}",
            paths
                .iter()
                .map(|p| format!("  - {}", p.display()))
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }
}

/// Build variable map for interpolation
fn build_variables(ctx: &LayoutContext) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    vars.insert("version".to_string(), ctx.version.clone());
    vars.insert("name".to_string(), ctx.name.clone());
    vars.insert("platform".to_string(), ctx.platform.os.to_string());
    vars.insert("arch".to_string(), ctx.platform.arch.to_string());
    vars.insert("os".to_string(), ctx.platform.os.to_string());
    // Rust target triple (e.g., x86_64-unknown-linux-gnu)
    vars.insert(
        "target_triple".to_string(),
        ctx.platform.rust_target_triple().to_string(),
    );
    vars.insert(
        "ext".to_string(),
        if ctx.platform.os == Os::Windows {
            ".exe".to_string()
        } else {
            String::new()
        },
    );
    vars
}

/// Interpolate variables in a template string
fn interpolate(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
}

fn default_bin_dir() -> String {
    "bin".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Arch;

    fn test_context() -> LayoutContext {
        LayoutContext {
            version: "1.3.0".to_string(),
            name: "yasm".to_string(),
            platform: Platform::new(Os::Windows, Arch::X86_64),
        }
    }

    #[test]
    fn test_interpolate_variables() {
        let vars = build_variables(&test_context());

        assert_eq!(
            interpolate("yasm-{version}-win64.exe", &vars),
            "yasm-1.3.0-win64.exe"
        );
        assert_eq!(interpolate("{name}{ext}", &vars), "yasm.exe");
    }

    #[test]
    fn test_resolve_binary_layout() {
        let mut binary_map = HashMap::new();
        binary_map.insert(
            "windows-x86_64".to_string(),
            BinaryLayout {
                source_name: "yasm-{version}-win64.exe".to_string(),
                target_name: "yasm.exe".to_string(),
                target_dir: "bin".to_string(),
                target_permissions: None,
            },
        );

        let layout = ExecutableLayout {
            download_type: DownloadType::Binary,
            binary: Some(binary_map),
            archive: None,
            msi: None,
            windows: None,
            macos: None,
            linux: None,
        };

        let ctx = test_context();
        let resolved = layout.resolve(&ctx).unwrap();

        match resolved {
            ResolvedLayout::Binary {
                source_name,
                target_name,
                target_dir,
                ..
            } => {
                assert_eq!(source_name, "yasm-1.3.0-win64.exe");
                assert_eq!(target_name, "yasm.exe");
                assert_eq!(target_dir, "bin");
            }
            _ => panic!("Expected Binary layout"),
        }
    }

    #[test]
    fn test_resolve_archive_layout() {
        let layout = ExecutableLayout {
            download_type: DownloadType::Archive,
            binary: None,
            archive: Some(ArchiveLayout {
                executable_paths: vec!["bin/{name}.exe".to_string()],
                strip_prefix: Some("{name}-{version}".to_string()),
                permissions: None,
            }),
            msi: None,
            windows: None,
            macos: None,
            linux: None,
        };

        let ctx = test_context();
        let resolved = layout.resolve(&ctx).unwrap();

        match resolved {
            ResolvedLayout::Archive {
                executable_paths,
                strip_prefix,
                ..
            } => {
                assert_eq!(executable_paths, vec!["bin/yasm.exe"]);
                assert_eq!(strip_prefix, Some("yasm-1.3.0".to_string()));
            }
            _ => panic!("Expected Archive layout"),
        }
    }
}
