//! Registry of vx-managed command shims (RFC 0042)
//!
//! [`Shim`] generates the wrapper scripts; this module remembers what was
//! generated so the shims can be listed, refreshed and removed later.
//!
//! Keeping this record matters for two reasons:
//!
//! - **Refresh**: the generated scripts bake in an absolute path to the `vx`
//!   executable. When vx is upgraded or moved, `vx shim sync` rewrites every
//!   shim against the new location.
//! - **Safety**: a shim name often collides with a real binary (`git` being the
//!   obvious one). The registry is what lets `vx shim remove` delete only the
//!   files vx created and leave the system `git` alone.
//!
//! The registry is a plain JSON file inside the vx configuration directory, so
//! it follows `VX_HOME` and can be inspected by hand.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::Platform;
use crate::shim::{Shim, ShimType};

/// File name of the shim registry inside the vx configuration directory
pub const SHIM_REGISTRY_FILE: &str = "command-shims.json";

/// A command shim managed by vx
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandShim {
    /// Name the user types on the command line (e.g. `jq`)
    pub name: String,

    /// Runtime spec forwarded to vx (e.g. `jq`, `git@2.53.0`, `node@22`)
    pub runtime: String,

    /// Absolute path of the `vx` executable the shim invokes
    pub launcher: PathBuf,

    /// Home whose managed runtimes and packages the command uses.
    ///
    /// Older records omitted this field and inherit the caller's home until
    /// explicitly refreshed by `vx shim sync`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vx_home: Option<PathBuf>,

    /// Directories the shim files were written to
    pub dirs: Vec<PathBuf>,

    /// Every file written for this shim
    pub files: Vec<PathBuf>,

    /// Creation time, RFC 3339
    pub created_at: String,
}

impl CommandShim {
    /// Build the `vx <runtime>` forwarding shim for `name`
    ///
    /// This is the piece the user would otherwise write by hand:
    ///
    /// ```sh
    /// #!/bin/sh
    /// exec vx jq "$@"
    /// ```
    pub fn shim(&self) -> Shim {
        let shim = Shim::new(&self.name, &self.launcher).with_args(&[&self.runtime]);
        match &self.vx_home {
            Some(home) => shim.with_env("VX_HOME", home.to_string_lossy()),
            None => shim,
        }
    }

    /// Whether every recorded file still contains this command's wrapper.
    pub fn is_complete(&self) -> bool {
        !self.files.is_empty() && self.files.iter().all(|path| self.owns_file(path))
    }

    /// Whether a recorded file exactly matches this command's generated wrapper.
    ///
    /// Symbolic links, paths outside the recorded directories, edited wrappers,
    /// and wrappers bound to another home are not owned by this record.
    pub fn owns_file(&self, path: &Path) -> bool {
        if self.validate().is_err() || !self.files.iter().any(|file| same_path(file, path)) {
            return false;
        }
        let Ok(metadata) = std::fs::symlink_metadata(path) else {
            return false;
        };
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return false;
        }
        let shim = self.shim();
        let variant = [ShimType::Batch, ShimType::Shell, ShimType::PowerShell]
            .into_iter()
            .find(|variant| {
                self.dirs
                    .iter()
                    .any(|dir| same_path(&shim.path_in(dir, *variant), path))
            });
        let Some(variant) = variant else {
            return false;
        };
        std::fs::read_to_string(path)
            .map(|content| content == shim.content_for(variant))
            .unwrap_or(false)
    }

    /// Remove only recorded files that still belong to this command.
    ///
    /// Files replaced by a user or another vx home are preserved.
    pub fn remove_files(&self) -> Result<Vec<PathBuf>> {
        self.validate()?;
        let mut removed = Vec::new();
        for path in &self.files {
            if self.owns_file(path) {
                std::fs::remove_file(path)
                    .with_context(|| format!("Failed to remove shim: {}", path.display()))?;
                removed.push(path.clone());
            }
        }
        Ok(removed)
    }

    fn validate(&self) -> Result<()> {
        validate_command_shim_name(&self.name)?;
        anyhow::ensure!(
            !self.runtime.is_empty() && !self.runtime.starts_with('-'),
            "A command shim requires a runtime or package request"
        );
        validate_literal(&self.runtime)?;
        validate_absolute_path(&self.launcher, "vx executable")?;
        if let Some(home) = &self.vx_home {
            validate_absolute_path(home, "vx home")?;
        }
        anyhow::ensure!(!self.dirs.is_empty(), "A shim requires an output directory");
        for dir in &self.dirs {
            validate_absolute_path(dir, "shim directory")?;
            validate_output_directory(dir)?;
        }
        Ok(())
    }
}

/// Validate a command name before publishing or removing global wrappers.
pub fn validate_command_shim_name(name: &str) -> Result<()> {
    let stem = name.split('.').next().unwrap_or("").to_ascii_lowercase();
    let reserved = matches!(stem.as_str(), "vx" | "con" | "prn" | "aux" | "nul")
        || (stem.len() == 4
            && (stem.starts_with("com") || stem.starts_with("lpt"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'));
    anyhow::ensure!(
        !name.is_empty()
            && !name.starts_with('.')
            && !name.ends_with('.')
            && name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.' | b'+'))
            && !reserved,
        "Invalid or reserved shim name: {name:?}"
    );
    Ok(())
}

fn validate_literal(value: &str) -> Result<()> {
    anyhow::ensure!(
        !value.chars().any(|c| c.is_control() || c == '"'),
        "Shim requests and paths cannot contain control characters or double quotes"
    );
    Ok(())
}

fn validate_absolute_path(path: &Path, label: &str) -> Result<()> {
    anyhow::ensure!(path.is_absolute(), "The {label} path must be absolute");
    anyhow::ensure!(
        !path
            .components()
            .any(|component| component == std::path::Component::ParentDir),
        "The {label} path cannot contain parent traversal"
    );
    validate_literal(&path.to_string_lossy())
}

fn validate_output_directory(dir: &Path) -> Result<()> {
    match std::fs::symlink_metadata(dir) {
        Ok(metadata) => anyhow::ensure!(
            metadata.is_dir() && !metadata.file_type().is_symlink(),
            "Shim output directory must be a directory, not a symbolic link: {}",
            dir.display()
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if let Some(parent) = dir.parent() {
                validate_output_directory(parent)?;
            }
        }
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

fn same_name(left: &str, right: &str) -> bool {
    if cfg!(windows) {
        left.eq_ignore_ascii_case(right)
    } else {
        left == right
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    #[cfg(windows)]
    {
        let normalize = |path: &Path| {
            let text = path.to_string_lossy().replace('\\', "/");
            text.strip_prefix("//?/")
                .unwrap_or(&text)
                .trim_end_matches('/')
                .to_ascii_lowercase()
        };
        normalize(left) == normalize(right)
    }
    #[cfg(not(windows))]
    {
        left == right
    }
}

/// Persistent set of [`CommandShim`]s
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ShimRegistry {
    /// Registered command shims, kept sorted by name
    pub shims: Vec<CommandShim>,
}

impl ShimRegistry {
    /// Default registry location for a vx home
    pub fn default_path(config_dir: &Path) -> PathBuf {
        config_dir.join(SHIM_REGISTRY_FILE)
    }

    /// Load a registry, treating a missing file as empty
    pub fn load_or_default(path: &Path) -> Self {
        match Self::load(path) {
            Ok(registry) => registry,
            Err(error) => {
                debug!(
                    "No usable shim registry at {} ({}), starting empty",
                    path.display(),
                    error
                );
                Self::default()
            }
        }
    }

    /// Load a registry from disk
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read shim registry: {}", path.display()))?;

        let mut registry: Self = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse shim registry: {}", path.display()))?;
        for (index, entry) in registry.shims.iter().enumerate() {
            validate_command_shim_name(&entry.name)?;
            anyhow::ensure!(
                !registry.shims[..index]
                    .iter()
                    .any(|previous| same_name(&previous.name, &entry.name)),
                "Duplicate command shim name in registry: {}",
                entry.name
            );
        }
        registry.shims.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(registry)
    }

    /// Persist the registry, creating parent directories as needed
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        let content = serde_json::to_string_pretty(self)
            .with_context(|| format!("Failed to serialize shim registry: {}", path.display()))?;
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write shim registry: {}", path.display()))?;

        Ok(())
    }

    /// Look up a shim by name
    pub fn get(&self, name: &str) -> Option<&CommandShim> {
        self.shims.iter().find(|shim| same_name(&shim.name, name))
    }

    /// Insert or replace a shim, keeping the list sorted by name
    pub fn upsert(&mut self, shim: CommandShim) {
        self.shims
            .retain(|existing| !same_name(&existing.name, &shim.name));
        self.shims.push(shim);
        self.shims.sort_by(|a, b| a.name.cmp(&b.name));
    }

    /// Remove a shim by name
    pub fn remove(&mut self, name: &str) -> Option<CommandShim> {
        let position = self
            .shims
            .iter()
            .position(|shim| same_name(&shim.name, name))?;
        Some(self.shims.remove(position))
    }

    /// Whether any shim is registered under `name`
    pub fn contains(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// Names of all registered shims, sorted
    pub fn names(&self) -> Vec<&str> {
        self.shims.iter().map(|shim| shim.name.as_str()).collect()
    }

    /// Whether the registry holds no shims
    pub fn is_empty(&self) -> bool {
        self.shims.is_empty()
    }

    /// Number of registered shims
    pub fn len(&self) -> usize {
        self.shims.len()
    }
}

/// Create a command shim in every target directory and record it
///
/// Writes all platform variants (see [`Shim::create_all`]) into each directory
/// in `dirs` and returns the resulting [`CommandShim`] entry. Callers persist
/// the entry themselves by adding it to a [`ShimRegistry`].
pub fn create_command_shim(
    name: &str,
    runtime: &str,
    launcher: &Path,
    dirs: &[PathBuf],
    platform: &Platform,
) -> Result<CommandShim> {
    let entry = CommandShim {
        name: name.to_string(),
        runtime: runtime.to_string(),
        launcher: launcher.to_path_buf(),
        vx_home: None,
        dirs: dirs.to_vec(),
        files: Vec::new(),
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    };
    write_command_shim(entry, platform, None)
}

/// Create or refresh a command bound to one vx home.
///
/// Every destination is checked before any file is written. Existing files must
/// exactly match either the desired wrapper or `previous`, including its home.
/// A legacy record without a home can be migrated only if its contents match.
pub fn create_command_shim_in_home(
    name: &str,
    runtime: &str,
    launcher: &Path,
    vx_home: &Path,
    dirs: &[PathBuf],
    platform: &Platform,
    previous: Option<&CommandShim>,
) -> Result<CommandShim> {
    let entry = CommandShim {
        name: name.to_string(),
        runtime: runtime.to_string(),
        launcher: launcher.to_path_buf(),
        vx_home: Some(vx_home.to_path_buf()),
        dirs: dirs.to_vec(),
        files: Vec::new(),
        created_at: previous
            .map(|entry| entry.created_at.clone())
            .unwrap_or_else(|| {
                chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            }),
    };
    write_command_shim(entry, platform, previous)
}

fn write_command_shim(
    mut entry: CommandShim,
    platform: &Platform,
    previous: Option<&CommandShim>,
) -> Result<CommandShim> {
    entry.validate()?;
    if let Some(previous) = previous {
        anyhow::ensure!(
            same_name(&previous.name, &entry.name)
                && previous.vx_home.as_ref().is_none_or(|home| {
                    entry
                        .vx_home
                        .as_ref()
                        .is_some_and(|new_home| same_path(home, new_home))
                }),
            "Cannot replace a command shim belonging to another name or vx home"
        );
        // Retain recorded spelling so a case-only update does not look like a
        // move to callers cleaning up the previous file list on Windows.
        entry.name.clone_from(&previous.name);
    }
    let shim = entry.shim();
    entry.files = entry
        .dirs
        .iter()
        .flat_map(|dir| shim.paths_in(dir, platform))
        .collect();
    for path in &entry.files {
        anyhow::ensure!(
            !same_path(path, &entry.launcher),
            "A shim cannot replace the vx executable"
        );
        match std::fs::symlink_metadata(path) {
            Ok(_) => anyhow::ensure!(
                entry.owns_file(path) || previous.is_some_and(|previous| previous.owns_file(path)),
                "Refusing to replace a file not owned by this command shim: {}",
                path.display()
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    for dir in &entry.dirs {
        shim.create_all(dir, platform)?;
    }
    Ok(entry)
}
