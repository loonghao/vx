//! Embedded bridge binary registry.
//!
//! On Windows, bridge binaries (e.g., `MSBuild.exe`) are embedded directly into
//! the `vx` binary at compile time. This avoids the need to ship separate executables
//! alongside `vx`.
//!
//! ## How it works
//!
//! 1. `vx-cli/build.rs` locates the compiled bridge binary in the target directory
//!    and generates code that embeds it via `include_bytes!`
//! 2. `vx-cli` calls `register_embedded_bridge()` at startup to register the bytes
//! 3. `deploy_bridge()` checks the registry before reporting "not found"
//! 4. If found in the registry, the bytes are written to the target path

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};

/// Global registry of embedded bridge binaries.
static EMBEDDED_BRIDGES: OnceLock<RwLock<HashMap<String, &'static [u8]>>> = OnceLock::new();

fn registry() -> &'static RwLock<HashMap<String, &'static [u8]>> {
    EMBEDDED_BRIDGES.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Register an embedded bridge binary.
///
/// Call this at program startup to make embedded bridges available to `deploy_bridge()`.
///
/// # Arguments
///
/// * `name` — Bridge name (e.g., "MSBuild")
/// * `data` — Static byte slice of the compiled bridge binary
pub fn register_embedded_bridge(name: &str, data: &'static [u8]) {
    if data.is_empty() {
        return;
    }
    if let Ok(mut map) = registry().write() {
        map.insert(name.to_string(), data);
    }
}

/// Deploy an embedded bridge binary to a target path.
///
/// Returns `Ok(path)` if the bridge was found in the registry and written successfully,
/// or `Err(())` if the bridge is not registered.
pub fn deploy_embedded_bridge(
    name: &str,
    target_path: &Path,
) -> Result<PathBuf, DeployEmbeddedError> {
    let data = {
        let map = registry()
            .read()
            .map_err(|_| DeployEmbeddedError::NotRegistered)?;
        match map.get(name) {
            Some(data) if !data.is_empty() => *data,
            _ => return Err(DeployEmbeddedError::NotRegistered),
        }
    };

    // Ensure parent directory exists
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent).map_err(DeployEmbeddedError::Io)?;
    }

    // Write the embedded binary
    std::fs::write(target_path, data).map_err(DeployEmbeddedError::Io)?;

    // Set executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(target_path, perms).map_err(DeployEmbeddedError::Io)?;
    }

    Ok(target_path.to_path_buf())
}

/// Errors from deploying an embedded bridge.
#[derive(Debug)]
pub enum DeployEmbeddedError {
    /// Bridge not found in the embedded registry.
    NotRegistered,
    /// I/O error writing the bridge binary.
    Io(std::io::Error),
}

impl std::fmt::Display for DeployEmbeddedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotRegistered => write!(f, "bridge not registered in embedded registry"),
            Self::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for DeployEmbeddedError {}
