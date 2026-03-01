//! Shell path resolution for manifest-driven runtimes (RFC 0038).
//!
//! This module handles finding shell executables provided by a runtime
//! (e.g., `git-bash`, `git-cmd` provided by Git for Windows).

use std::path::{Path, PathBuf};

use tracing::debug;

use crate::{RuntimeContext, platform::Platform};

use super::types::{DetectionConfig, ShellDefinition};

/// Find the path to a shell executable provided by a runtime.
///
/// Search order:
/// 1. vx store directory (managed installation)
/// 2. System paths from detection config
/// 3. Executable in PATH → derive install directory
/// 4. Shell directly in PATH (last resort)
pub fn get_shell_path(
    shell_name: &str,
    shells: &[ShellDefinition],
    executable: &str,
    store_name: &str,
    version: &str,
    detection: Option<&DetectionConfig>,
    ctx: &RuntimeContext,
) -> Option<PathBuf> {
    // Find the shell definition
    let shell_def = shells.iter().find(|s| s.name == shell_name)?;
    let shell_relative = &shell_def.path;

    debug!(
        "Looking for shell '{}' with relative path '{}'",
        shell_name, shell_relative
    );

    // 1. Try vx store directory first
    let platform = Platform::current();
    let base_path = ctx.paths.version_store_dir(store_name, version);
    let install_path = base_path.join(platform.as_str());
    let shell_path = install_path.join(shell_relative);

    debug!("Checking vx store path: {}", shell_path.display());

    if shell_path.exists() {
        debug!("Found shell in vx store: {}", shell_path.display());
        return Some(shell_path);
    }

    // 2. Try system paths from detection config
    if let Some(detection) = detection {
        for sys_path in &detection.system_paths {
            let path = PathBuf::from(sys_path);
            if path.exists() {
                // Found the executable, derive install directory
                if let Some(install_dir) = derive_install_dir_from_executable(&path) {
                    let shell_in_install = install_dir.join(shell_relative);
                    debug!(
                        "Checking system install path: {}",
                        shell_in_install.display()
                    );
                    if shell_in_install.exists() {
                        debug!(
                            "Found shell in system install: {}",
                            shell_in_install.display()
                        );
                        return Some(shell_in_install);
                    }
                }
            }
        }
    }

    // 3. Try to find executable in PATH and derive install directory
    if let Ok(exe_path) = which::which(executable) {
        debug!(
            "Found executable '{}' at: {}",
            executable,
            exe_path.display()
        );
        if let Some(install_dir) = derive_install_dir_from_executable(&exe_path) {
            let shell_in_install = install_dir.join(shell_relative);
            debug!(
                "Checking derived install path: {}",
                shell_in_install.display()
            );
            if shell_in_install.exists() {
                debug!(
                    "Found shell in derived install: {}",
                    shell_in_install.display()
                );
                return Some(shell_in_install);
            }
        }
    }

    // 4. Try to find shell directly in PATH (last resort)
    if let Ok(shell_exe) = which::which(shell_name) {
        debug!("Found shell in PATH: {}", shell_exe.display());
        return Some(shell_exe);
    }

    debug!("Shell '{}' not found", shell_name);
    None
}

/// Derive the installation directory from an executable path.
///
/// This is used to find the root installation directory for system-installed tools.
/// For example, Git for Windows installs git.exe to:
/// - `C:\Program Files\Git\cmd\git.exe` or `C:\Program Files\Git\bin\git.exe`
///
/// The install directory would be `C:\Program Files\Git`.
pub fn derive_install_dir_from_executable(exe_path: &Path) -> Option<PathBuf> {
    // Get the parent directory of the executable
    let parent = exe_path.parent()?;

    // Common patterns for installation directories:
    // - Windows: <install>/bin/..., <install>/cmd/..., <install>/...
    // - Unix: <install>/bin/...

    let parent_name = parent.file_name()?.to_str()?;

    // Check if parent is a common bin directory
    if matches!(parent_name, "bin" | "cmd" | "sbin" | "libexec" | "Scripts") {
        // The install directory is the parent of the bin directory
        return parent.parent().map(|p| p.to_path_buf());
    }

    // On Windows, also check for mingw64/bin pattern (Git for Windows)
    #[cfg(windows)]
    {
        if parent_name == "bin"
            && let Some(grandparent) = parent.parent()
            && let Some(grandparent_name) = grandparent.file_name().and_then(|n| n.to_str())
            && (grandparent_name == "mingw64" || grandparent_name == "mingw32")
        {
            // Git for Windows: <install>/mingw64/bin/
            return grandparent.parent().map(|p| p.to_path_buf());
        }
    }

    // Otherwise, assume the parent is the install directory
    Some(parent.to_path_buf())
}
