//! Cross-platform shim generation utilities
//!
//! This module provides utilities for creating executable shim scripts that
//! forward commands to other executables. Shims are commonly needed when:
//!
//! - A tool doesn't provide a standalone executable (e.g., `bunx` is `bun x`)
//! - Multiple commands share the same underlying executable with different args
//! - Need to wrap executables with environment setup
//!
//! # Example
//!
//! ```rust,ignore
//! use vx_runtime::{Platform, Shim};
//!
//! // Create a simple shim that forwards `bunx` to `bun x`
//! let shim = Shim::new("bunx", "/path/to/bun")
//!     .with_args(&["x"])
//!     .build();
//!
//! // Create the shim file
//! shim.create("/path/to/shim/dir", &Platform::current())?;
//! ```
//!
//! ## Command shims (RFC 0042)
//!
//! A shim does not have to point at a real binary. Pointing it at the `vx`
//! executable and prepending a runtime name turns it into a **command shim**:
//! a first-class `jq` / `git` entry point that works without typing `vx`.
//!
//! ```rust,ignore
//! use vx_runtime::{Platform, Shim};
//!
//! // `jq --version` -> `vx jq --version`
//! let shim = Shim::new("jq", vx_exe_path).with_args(&["jq"]);
//! let written = shim.create_all(shim_dir, &Platform::current())?;
//! ```
//!
//! [`create_all`](Shim::create_all) writes every variant the platform needs —
//! see [`ShimType::platform_variants`]. Every generated file carries
//! [`VX_SHIM_MARKER`] so vx can tell its own shims from user files and never
//! delete something it did not create.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::debug;

use crate::Platform;

mod generate;

/// Marker written into every vx-generated shim.
///
/// Used to identify files vx owns before overwriting or deleting them, so a
/// hand-written wrapper (for example `~/.local/bin/jq`) is never clobbered.
pub const VX_SHIM_MARKER: &str = "vx-shim";

/// Shim type determines the script format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShimType {
    /// Windows batch script (.cmd)
    Batch,
    /// Windows PowerShell script (.ps1)
    PowerShell,
    /// Unix shell script (no extension, uses shebang)
    Shell,
}

impl ShimType {
    /// Get the file extension for this shim type
    pub fn extension(&self) -> &'static str {
        match self {
            ShimType::Batch => ".cmd",
            ShimType::PowerShell => ".ps1",
            ShimType::Shell => "",
        }
    }

    /// Detect the appropriate shim type for the current platform
    pub fn detect() -> Self {
        if cfg!(windows) {
            ShimType::Batch
        } else {
            ShimType::Shell
        }
    }

    /// Every shim variant worth generating for a platform.
    ///
    /// Windows is not one shell but three depending on where the user types the
    /// command, so a complete alias needs more than one file:
    ///
    /// | Variant | Invoked from | File |
    /// |---|---|---|
    /// | [`Batch`](ShimType::Batch) | cmd.exe, PowerShell (via `PATHEXT`) | `<name>.cmd` |
    /// | [`Shell`](ShimType::Shell) | Git Bash, MSYS2, Cygwin | `<name>` |
    ///
    /// Unix terminals all speak `/bin/sh`, so [`Shell`](ShimType::Shell) alone
    /// is enough there.
    pub fn platform_variants(platform: &Platform) -> Vec<ShimType> {
        if platform.is_windows() {
            vec![ShimType::Batch, ShimType::Shell]
        } else {
            vec![ShimType::Shell]
        }
    }
}

/// A shim script that forwards commands to another executable
#[derive(Debug, Clone)]
pub struct Shim {
    /// Name of the shim (e.g., "bunx")
    pub name: String,

    /// Path to the target executable
    pub target: PathBuf,

    /// Arguments to prepend before user args
    pub args: Vec<String>,

    /// Environment variables to set before execution
    pub env: Vec<(String, String)>,

    /// Working directory (None = inherit from caller)
    pub working_dir: Option<PathBuf>,

    /// Shim type (auto-detected if None)
    pub shim_type: Option<ShimType>,
}

impl Shim {
    /// Create a new shim builder
    ///
    /// # Arguments
    /// * `name` - Name of the shim executable (without extension)
    /// * `target` - Path to the target executable
    pub fn new(name: impl Into<String>, target: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            target: target.into(),
            args: Vec::new(),
            env: Vec::new(),
            working_dir: None,
            shim_type: None,
        }
    }

    /// Add arguments to prepend before user-provided arguments
    ///
    /// # Example
    /// ```rust,ignore
    /// // bunx -> bun x [user args]
    /// Shim::new("bunx", "/path/to/bun")
    ///     .with_args(&["x"]);
    /// ```
    pub fn with_args(mut self, args: &[&str]) -> Self {
        self.args = args.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add environment variable to set before execution
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    /// Add multiple environment variables
    pub fn with_envs(mut self, envs: &[(impl AsRef<str>, impl AsRef<str>)]) -> Self {
        for (k, v) in envs {
            self.env
                .push((k.as_ref().to_string(), v.as_ref().to_string()));
        }
        self
    }

    /// Set working directory for the shim
    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Set the shim type explicitly
    pub fn with_type(mut self, shim_type: ShimType) -> Self {
        self.shim_type = Some(shim_type);
        self
    }

    /// Get the shim file name (with platform-appropriate extension)
    pub fn file_name(&self, platform: &Platform) -> String {
        let shim_type = self.shim_type.unwrap_or_else(|| {
            if platform.is_windows() {
                ShimType::Batch
            } else {
                ShimType::Shell
            }
        });

        format!("{}{}", self.name, shim_type.extension())
    }

    /// Generate the shim script content for the given platform
    pub fn content(&self, platform: &Platform) -> String {
        let shim_type = self.shim_type.unwrap_or_else(|| {
            if platform.is_windows() {
                ShimType::Batch
            } else {
                ShimType::Shell
            }
        });

        self.content_for(shim_type)
    }

    /// Generate the shim script content for one specific shim type
    pub fn content_for(&self, shim_type: ShimType) -> String {
        match shim_type {
            ShimType::Batch => generate::batch(self),
            ShimType::PowerShell => generate::powershell(self),
            ShimType::Shell => generate::shell(self),
        }
    }

    /// Path this shim would occupy in `dir` for one specific shim type
    pub fn path_in(&self, dir: &Path, shim_type: ShimType) -> PathBuf {
        dir.join(format!("{}{}", self.name, shim_type.extension()))
    }

    /// Every path this shim occupies in `dir` on the given platform
    pub fn paths_in(&self, dir: &Path, platform: &Platform) -> Vec<PathBuf> {
        ShimType::platform_variants(platform)
            .into_iter()
            .map(|variant| self.path_in(dir, variant))
            .collect()
    }

    /// Create the shim file in the specified directory
    ///
    /// # Arguments
    /// * `dir` - Directory to create the shim in
    /// * `platform` - Platform for determining shim format
    ///
    /// # Returns
    /// The path to the created shim file
    pub fn create(&self, dir: &Path, platform: &Platform) -> Result<PathBuf> {
        let shim_file = dir.join(self.file_name(platform));
        let content = self.content(platform);

        // Write the file
        self.write(&shim_file, &content)?;

        debug!("Created shim at {}", shim_file.display());
        Ok(shim_file)
    }

    /// Create every platform variant of this shim in `dir`.
    ///
    /// On Windows this writes both `<name>.cmd` (cmd.exe / PowerShell) and an
    /// extension-less `<name>` (Git Bash / MSYS2); on Unix just `<name>`.
    ///
    /// # Returns
    /// The paths of every file written, in [`ShimType::platform_variants`] order.
    pub fn create_all(&self, dir: &Path, platform: &Platform) -> Result<Vec<PathBuf>> {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create shim directory: {}", dir.display()))?;

        let mut written = Vec::new();
        for variant in ShimType::platform_variants(platform) {
            let path = self.path_in(dir, variant);
            self.write(&path, &self.content_for(variant))?;
            debug!("Created {} shim at {}", variant.extension(), path.display());
            written.push(path);
        }

        Ok(written)
    }

    /// Remove every platform variant of this shim from `dir`.
    ///
    /// Only files carrying [`VX_SHIM_MARKER`] are deleted, so a user-written
    /// wrapper that happens to share the name is left untouched.
    ///
    /// # Returns
    /// The paths that were removed.
    pub fn remove_all(&self, dir: &Path, platform: &Platform) -> Result<Vec<PathBuf>> {
        let mut removed = Vec::new();

        for path in self.paths_in(dir, platform) {
            if !Self::is_managed(&path) {
                debug!(
                    "Skipping {} — not created by vx",
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
                continue;
            }

            std::fs::remove_file(&path)
                .with_context(|| format!("Failed to remove shim: {}", path.display()))?;
            removed.push(path);
        }

        Ok(removed)
    }

    /// Whether `path` was generated by vx.
    ///
    /// Reads the file and looks for [`VX_SHIM_MARKER`]. Unreadable or missing
    /// files are reported as unmanaged rather than treated as an error.
    pub fn is_managed(path: &Path) -> bool {
        std::fs::read_to_string(path)
            .map(|content| content.contains(VX_SHIM_MARKER))
            .unwrap_or(false)
    }

    /// Write shim content and apply platform permissions
    fn write(&self, path: &Path, content: &str) -> Result<()> {
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write shim to {}", path.display()))?;

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(path, perms)?;
        }

        Ok(())
    }

    /// Create shim in the same directory as the target executable
    ///
    /// This is useful for creating shims like `bunx` next to `bun`.
    pub fn create_next_to_target(&self, platform: &Platform) -> Result<PathBuf> {
        let dir = self
            .target
            .parent()
            .context("Target has no parent directory")?;
        self.create(dir, platform)
    }
}

/// Builder for creating multiple shims at once
#[derive(Debug, Default)]
pub struct ShimBuilder {
    /// Directory to create shims in
    dir: Option<PathBuf>,

    /// Platform for shim generation
    platform: Option<Platform>,

    /// Shims to create
    shims: Vec<Shim>,
}

impl ShimBuilder {
    /// Create a new shim builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the output directory for shims
    pub fn dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.dir = Some(dir.into());
        self
    }

    /// Set the platform for shim generation
    pub fn platform(mut self, platform: Platform) -> Self {
        self.platform = Some(platform);
        self
    }

    /// Add a shim to create
    pub fn shim(mut self, shim: Shim) -> Self {
        self.shims.push(shim);
        self
    }

    /// Add a simple forwarding shim
    ///
    /// # Arguments
    /// * `name` - Shim name
    /// * `target` - Target executable path
    /// * `args` - Arguments to prepend
    pub fn forward(
        mut self,
        name: impl Into<String>,
        target: impl Into<PathBuf>,
        args: &[&str],
    ) -> Self {
        self.shims.push(Shim::new(name, target).with_args(args));
        self
    }

    /// Create all shims
    ///
    /// # Returns
    /// List of created shim paths
    pub fn build(self) -> Result<Vec<PathBuf>> {
        let dir = self.dir.context("Output directory not set")?;
        let platform = self.platform.unwrap_or_else(Platform::current);

        // Ensure directory exists
        std::fs::create_dir_all(&dir)?;

        let mut paths = Vec::new();
        for shim in &self.shims {
            let path = shim.create(&dir, &platform)?;
            paths.push(path);
        }

        Ok(paths)
    }
}

/// Convenience function to create a simple forwarding shim
///
/// # Example
/// ```rust,ignore
/// use vx_runtime::create_shim;
///
/// // Create bunx -> bun x
/// create_shim("bunx", "/path/to/bun", &["x"], "/path/to/bin")?;
/// ```
pub fn create_shim(name: &str, target: &Path, args: &[&str], output_dir: &Path) -> Result<PathBuf> {
    let platform = Platform::current();
    Shim::new(name, target)
        .with_args(args)
        .create(output_dir, &platform)
}
