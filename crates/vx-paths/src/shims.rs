//! Shim Generation for Global Packages (RFC 0025)
//!
//! This module provides cross-platform shim generation for globally installed packages.
//! Shims are wrapper scripts that delegate to the actual package executables.
//!
//! ## Future Enhancement: shimexe-core Integration
//!
//! This module can be enhanced to use `shimexe-core` (https://github.com/loonghao/shimexe)
//! for advanced shim functionality:
//! - TOML-based shim configuration (.shim.toml files)
//! - Environment variable expansion with ${VAR:default} syntax
//! - HTTP download support for remote tools
//! - Static binary shims (no shell overhead)
//! - Archive extraction support
//!
//! ```toml
//! # Add to Cargo.toml when ready:
//! # shimexe-core = "0.1"
//! ```
//!
//! ## Current Implementation
//!
//! ### Unix Shims
//! Unix shims are shell wrapper scripts with 755 permissions:
//! ```sh
//! #!/bin/sh
//! exec "/path/to/executable" "$@"
//! ```
//!
//! ### Windows Shims
//! Windows shims are .cmd batch files:
//! ```cmd
//! @echo off
//! setlocal
//! "/path/to/executable" %*
//! ```

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Result of shim creation
#[derive(Debug)]
pub struct ShimResult {
    /// Path to the created shim
    pub shim_path: PathBuf,
    /// Whether this was a new creation or update
    pub created: bool,
}

/// Create a shim for an executable
///
/// On Unix, creates a shell wrapper script with executable permissions.
/// On Windows, creates a .cmd batch file.
///
/// # Arguments
/// * `shim_dir` - Directory where the shim should be created
/// * `exe_name` - Name of the executable (without extension)
/// * `target_path` - Full path to the target executable
///
/// # Returns
/// * `ShimResult` containing the path to the created shim
pub fn create_shim(shim_dir: &Path, exe_name: &str, target_path: &Path) -> Result<ShimResult> {
    std::fs::create_dir_all(shim_dir)
        .with_context(|| format!("Failed to create shim directory: {}", shim_dir.display()))?;

    #[cfg(windows)]
    {
        create_windows_shim(shim_dir, exe_name, target_path)
    }

    #[cfg(not(windows))]
    {
        create_unix_shim(shim_dir, exe_name, target_path)
    }
}

/// Create a Windows .cmd shim
#[cfg(windows)]
fn create_windows_shim(shim_dir: &Path, exe_name: &str, target_path: &Path) -> Result<ShimResult> {
    let shim_path = shim_dir.join(format!("{}.cmd", exe_name));
    let created = !shim_path.exists();

    // Use forward slashes in the script for better compatibility
    let target_str = target_path.to_string_lossy();

    // Create batch script content
    let content = format!(
        r#"@echo off
setlocal
"{}" %*
"#,
        target_str
    );

    std::fs::write(&shim_path, content)
        .with_context(|| format!("Failed to write shim: {}", shim_path.display()))?;

    Ok(ShimResult { shim_path, created })
}

/// Create a Unix shell wrapper shim
#[cfg(not(windows))]
fn create_unix_shim(shim_dir: &Path, exe_name: &str, target_path: &Path) -> Result<ShimResult> {
    use std::os::unix::fs::PermissionsExt;

    let shim_path = shim_dir.join(exe_name);
    let created = !shim_path.exists();

    // Create shell script content
    let content = format!(
        r#"#!/bin/sh
exec "{}" "$@"
"#,
        target_path.display()
    );

    std::fs::write(&shim_path, &content)
        .with_context(|| format!("Failed to write shim: {}", shim_path.display()))?;

    // Set executable permissions (755)
    let mut perms = std::fs::metadata(&shim_path)?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&shim_path, perms)
        .with_context(|| format!("Failed to set shim permissions: {}", shim_path.display()))?;

    Ok(ShimResult { shim_path, created })
}

/// Remove a shim for an executable
///
/// # Arguments
/// * `shim_dir` - Directory containing the shim
/// * `exe_name` - Name of the executable (without extension)
pub fn remove_shim(shim_dir: &Path, exe_name: &str) -> Result<bool> {
    #[cfg(windows)]
    let shim_path = shim_dir.join(format!("{}.cmd", exe_name));

    #[cfg(not(windows))]
    let shim_path = shim_dir.join(exe_name);

    if shim_path.exists() {
        std::fs::remove_file(&shim_path)
            .with_context(|| format!("Failed to remove shim: {}", shim_path.display()))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Get the shim path for an executable name
pub fn get_shim_path(shim_dir: &Path, exe_name: &str) -> PathBuf {
    #[cfg(windows)]
    {
        shim_dir.join(format!("{}.cmd", exe_name))
    }

    #[cfg(not(windows))]
    {
        shim_dir.join(exe_name)
    }
}

/// Check if a shim exists for an executable
pub fn shim_exists(shim_dir: &Path, exe_name: &str) -> bool {
    get_shim_path(shim_dir, exe_name).exists()
}

/// List all shims in a directory
pub fn list_shims(shim_dir: &Path) -> Result<Vec<String>> {
    if !shim_dir.exists() {
        return Ok(Vec::new());
    }

    let mut shims = Vec::new();

    for entry in std::fs::read_dir(shim_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

        #[cfg(windows)]
        {
            if let Some(name) = file_name.strip_suffix(".cmd") {
                shims.push(name.to_string());
            }
        }

        #[cfg(not(windows))]
        {
            // On Unix, shims don't have extensions
            if !file_name.contains('.') {
                shims.push(file_name.to_string());
            }
        }
    }

    shims.sort();
    Ok(shims)
}

/// Update all shims from a package registry
///
/// This function synchronizes the shims directory with the package registry,
/// creating new shims for packages that need them and removing stale shims.
pub fn sync_shims_from_registry(
    shim_dir: &Path,
    packages: &[(String, std::path::PathBuf)], // (exe_name, target_path)
) -> Result<SyncResult> {
    let mut created = 0;
    let mut removed = 0;
    let mut errors = Vec::new();

    // Get existing shims
    let existing = list_shims(shim_dir)?;
    let expected: std::collections::HashSet<_> = packages.iter().map(|(n, _)| n.clone()).collect();

    // Remove stale shims
    for shim_name in &existing {
        if !expected.contains(shim_name) {
            match remove_shim(shim_dir, shim_name) {
                Ok(true) => removed += 1,
                Ok(false) => {}
                Err(e) => errors.push(format!("Failed to remove {}: {}", shim_name, e)),
            }
        }
    }

    // Create/update shims
    for (exe_name, target_path) in packages {
        match create_shim(shim_dir, exe_name, target_path) {
            Ok(result) => {
                if result.created {
                    created += 1;
                }
            }
            Err(e) => errors.push(format!("Failed to create {}: {}", exe_name, e)),
        }
    }

    Ok(SyncResult {
        created,
        removed,
        errors,
    })
}

/// Result of syncing shims with package registry
#[derive(Debug, Default)]
pub struct SyncResult {
    /// Number of shims created
    pub created: usize,
    /// Number of shims removed
    pub removed: usize,
    /// Errors encountered during sync
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_shim_path() {
        let dir = Path::new("/tmp/shims");

        #[cfg(windows)]
        assert_eq!(get_shim_path(dir, "tsc"), Path::new("/tmp/shims/tsc.cmd"));

        #[cfg(not(windows))]
        assert_eq!(get_shim_path(dir, "tsc"), Path::new("/tmp/shims/tsc"));
    }

    #[test]
    fn test_create_and_remove_shim() {
        let temp = tempdir().unwrap();
        let shim_dir = temp.path().join("shims");
        let target = temp.path().join("bin").join("tool");

        // Create fake target
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(&target, "#!/bin/sh\necho hello").unwrap();

        // Create shim
        let result = create_shim(&shim_dir, "tool", &target).unwrap();
        assert!(result.created);
        assert!(result.shim_path.exists());

        // Check shim exists
        assert!(shim_exists(&shim_dir, "tool"));

        // List shims
        let shims = list_shims(&shim_dir).unwrap();
        assert_eq!(shims, vec!["tool"]);

        // Remove shim
        let removed = remove_shim(&shim_dir, "tool").unwrap();
        assert!(removed);
        assert!(!shim_exists(&shim_dir, "tool"));
    }

    #[test]
    fn test_sync_shims() {
        let temp = tempdir().unwrap();
        let shim_dir = temp.path().join("shims");
        let bin_dir = temp.path().join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();

        // Create fake executables
        let tool1 = bin_dir.join("tool1");
        let tool2 = bin_dir.join("tool2");
        std::fs::write(&tool1, "tool1").unwrap();
        std::fs::write(&tool2, "tool2").unwrap();

        // Sync shims
        let packages = vec![
            ("tool1".to_string(), tool1.clone()),
            ("tool2".to_string(), tool2.clone()),
        ];
        let result = sync_shims_from_registry(&shim_dir, &packages).unwrap();
        assert_eq!(result.created, 2);
        assert_eq!(result.removed, 0);

        // Now sync with only tool1
        let packages = vec![("tool1".to_string(), tool1)];
        let result = sync_shims_from_registry(&shim_dir, &packages).unwrap();
        assert_eq!(result.removed, 1);

        let shims = list_shims(&shim_dir).unwrap();
        assert_eq!(shims, vec!["tool1"]);
    }
}
