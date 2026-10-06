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
use crate::shim::Shim;

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
        Shim::new(&self.name, &self.launcher).with_args(&[&self.runtime])
    }

    /// Whether every recorded file still exists
    pub fn is_complete(&self) -> bool {
        !self.files.is_empty() && self.files.iter().all(|path| path.exists())
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

        if content.trim().is_empty() {
            return Ok(Self::default());
        }

        let mut registry: Self = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse shim registry: {}", path.display()))?;
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
        self.shims.iter().find(|shim| shim.name == name)
    }

    /// Insert or replace a shim, keeping the list sorted by name
    pub fn upsert(&mut self, shim: CommandShim) {
        self.shims.retain(|existing| existing.name != shim.name);
        self.shims.push(shim);
        self.shims.sort_by(|a, b| a.name.cmp(&b.name));
    }

    /// Remove a shim by name
    pub fn remove(&mut self, name: &str) -> Option<CommandShim> {
        let position = self.shims.iter().position(|shim| shim.name == name)?;
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
    let shim = Shim::new(name, launcher).with_args(&[runtime]);

    let mut files = Vec::new();
    for dir in dirs {
        files.extend(shim.create_all(dir, platform)?);
    }

    Ok(CommandShim {
        name: name.to_string(),
        runtime: runtime.to_string(),
        launcher: launcher.to_path_buf(),
        dirs: dirs.to_vec(),
        files,
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    })
}
