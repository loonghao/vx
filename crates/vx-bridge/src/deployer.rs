//! Bridge deployer — copies bridge executables to target locations.
//!
//! Bridge executables are compiled as separate binaries (e.g., `MSBuild.exe`)
//! and shipped alongside the main `vx` binary. When a provider installs a runtime,
//! it can use this module to deploy the appropriate bridge to the runtime's store.
//!
//! ## Bridge binary discovery
//!
//! Bridge binaries are located by searching (in order):
//! 1. Same directory as the current `vx` binary
//! 2. `VX_BRIDGE_DIR` environment variable (if set)
//! 3. `~/.vx/bin/` directory

use std::path::{Path, PathBuf};

/// Deploy a bridge executable to a target directory.
///
/// This finds the bridge binary (e.g., `MSBuild.exe`) that ships alongside `vx`,
/// and copies it to the specified target path.
///
/// # Arguments
///
/// * `bridge_name` — Name of the bridge binary (without `.exe` on non-Windows).
///   On Windows, `.exe` is appended automatically if not present.
/// * `target_path` — Full path where the bridge should be placed, including filename.
///
/// # Returns
///
/// The path to the deployed bridge, or an error if the bridge binary was not found.
///
/// # Example
///
/// ```rust,no_run
/// use vx_bridge::deploy_bridge;
///
/// // Deploy MSBuild.exe to the MSVC store directory
/// let deployed = deploy_bridge(
///     "MSBuild",
///     &std::path::PathBuf::from(r"C:\Users\user\.vx\store\msvc\14.42\MSBuild\Current\Bin\MSBuild.exe"),
/// ).unwrap();
/// ```
pub fn deploy_bridge(bridge_name: &str, target_path: &Path) -> Result<PathBuf, DeployError> {
    // Strategy 1: Try to find bridge binary on the filesystem
    match find_bridge_binary(bridge_name) {
        Ok(source) => {
            // Ensure target parent directory exists
            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| DeployError::CreateDir {
                    path: parent.to_path_buf(),
                    source: e,
                })?;
            }

            // Copy the bridge binary
            std::fs::copy(&source, target_path).map_err(|e| DeployError::Copy {
                from: source.clone(),
                to: target_path.to_path_buf(),
                source: e,
            })?;

            // On Unix, ensure the deployed binary is executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = std::fs::Permissions::from_mode(0o755);
                std::fs::set_permissions(target_path, perms).map_err(|e| {
                    DeployError::Permissions {
                        path: target_path.to_path_buf(),
                        source: e,
                    }
                })?;
            }

            return Ok(target_path.to_path_buf());
        }
        Err(_fs_err) => {
            // Strategy 2: Try to deploy from embedded data (compiled into the vx binary)
            if let Ok(path) = crate::deploy_embedded_bridge(bridge_name, target_path) {
                return Ok(path);
            }
        }
    }

    // Both strategies failed
    Err(DeployError::NotFound {
        name: platform_exe_name(bridge_name),
        searched: get_searched_locations(bridge_name),
    })
}

/// Find the bridge binary by searching known locations.
fn find_bridge_binary(name: &str) -> Result<PathBuf, DeployError> {
    let exe_name = platform_exe_name(name);

    // 1. Same directory as the current executable (vx binary)
    if let Ok(current_exe) = std::env::current_exe()
        && let Some(exe_dir) = current_exe.parent()
    {
        let candidate = exe_dir.join(&exe_name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    // 2. VX_BRIDGE_DIR environment variable
    if let Ok(bridge_dir) = std::env::var("VX_BRIDGE_DIR") {
        let candidate = PathBuf::from(&bridge_dir).join(&exe_name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    // 3. ~/.vx/bin/ directory
    if let Ok(vx_paths) = vx_paths::VxPaths::new() {
        let candidate = vx_paths.base_dir.join("bin").join(&exe_name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(DeployError::NotFound {
        name: exe_name,
        searched: get_searched_locations(name),
    })
}

/// Get the list of locations that were searched, for error messages.
fn get_searched_locations(name: &str) -> Vec<String> {
    let exe_name = platform_exe_name(name);
    let mut locations = Vec::new();

    if let Ok(current_exe) = std::env::current_exe()
        && let Some(exe_dir) = current_exe.parent()
    {
        locations.push(exe_dir.join(&exe_name).display().to_string());
    }

    if let Ok(bridge_dir) = std::env::var("VX_BRIDGE_DIR") {
        locations.push(
            PathBuf::from(&bridge_dir)
                .join(&exe_name)
                .display()
                .to_string(),
        );
    }

    if let Ok(vx_paths) = vx_paths::VxPaths::new() {
        locations.push(
            vx_paths
                .base_dir
                .join("bin")
                .join(&exe_name)
                .display()
                .to_string(),
        );
    }

    locations
}

/// Get the platform-specific executable name.
fn platform_exe_name(name: &str) -> String {
    if cfg!(windows) && !name.ends_with(".exe") {
        format!("{}.exe", name)
    } else {
        name.to_string()
    }
}

/// Errors that can occur during bridge deployment.
#[derive(Debug)]
pub enum DeployError {
    /// Bridge binary not found in any searched location.
    NotFound { name: String, searched: Vec<String> },
    /// Failed to create target directory.
    CreateDir {
        path: PathBuf,
        source: std::io::Error,
    },
    /// Failed to copy bridge binary.
    Copy {
        from: PathBuf,
        to: PathBuf,
        source: std::io::Error,
    },
    /// Failed to set permissions (Unix only).
    #[allow(dead_code)]
    Permissions {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl std::fmt::Display for DeployError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeployError::NotFound { name, searched } => {
                write!(
                    f,
                    "Bridge binary '{}' not found. Searched:\n{}",
                    name,
                    searched
                        .iter()
                        .map(|s| format!("  - {}", s))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
            DeployError::CreateDir { path, source } => {
                write!(
                    f,
                    "Failed to create directory '{}': {}",
                    path.display(),
                    source
                )
            }
            DeployError::Copy { from, to, source } => {
                write!(
                    f,
                    "Failed to copy '{}' to '{}': {}",
                    from.display(),
                    to.display(),
                    source
                )
            }
            DeployError::Permissions { path, source } => {
                write!(
                    f,
                    "Failed to set permissions on '{}': {}",
                    path.display(),
                    source
                )
            }
        }
    }
}

impl std::error::Error for DeployError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DeployError::NotFound { .. } => None,
            DeployError::CreateDir { source, .. } => Some(source),
            DeployError::Copy { source, .. } => Some(source),
            DeployError::Permissions { source, .. } => Some(source),
        }
    }
}
