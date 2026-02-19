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

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::debug;

use crate::Platform;

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

        match shim_type {
            ShimType::Batch => self.generate_batch(),
            ShimType::PowerShell => self.generate_powershell(),
            ShimType::Shell => self.generate_shell(),
        }
    }

    /// Generate Windows batch script content
    fn generate_batch(&self) -> String {
        let mut lines = vec!["@echo off".to_string(), "setlocal".to_string()];

        // Add environment variables
        for (key, value) in &self.env {
            lines.push(format!("set {}={}", key, value));
        }

        // Add working directory change if specified
        if let Some(ref dir) = self.working_dir {
            lines.push(format!("cd /d \"{}\"", dir.display()));
        }

        // Build the command
        let target_str = self.target.display().to_string();
        let args_str = if self.args.is_empty() {
            String::new()
        } else {
            format!(" {} ", self.args.join(" "))
        };

        lines.push(format!("\"{}\"{}%*", target_str, args_str));

        lines.join("\r\n")
    }

    /// Generate PowerShell script content
    fn generate_powershell(&self) -> String {
        let mut lines = vec!["#!/usr/bin/env pwsh".to_string()];

        // Add environment variables
        for (key, value) in &self.env {
            lines.push(format!("$env:{} = '{}'", key, value));
        }

        // Add working directory change if specified
        if let Some(ref dir) = self.working_dir {
            lines.push(format!("Set-Location -Path \"{}\"", dir.display()));
        }

        // Build the command
        let mut cmd_parts = vec![format!("\"{}\"", self.target.display())];
        cmd_parts.extend(self.args.iter().cloned());
        cmd_parts.push("$args".to_string());

        lines.push(format!("& {}", cmd_parts.join(" ")));

        lines.join("\n")
    }

    /// Generate Unix shell script content
    fn generate_shell(&self) -> String {
        let mut lines = vec!["#!/bin/sh".to_string()];

        // Add environment variables
        for (key, value) in &self.env {
            lines.push(format!("{}='{}'", key, value));
            lines.push(format!("export {}", key));
        }

        // Add working directory change if specified
        if let Some(ref dir) = self.working_dir {
            lines.push(format!("cd \"{}\"", dir.display()));
        }

        // Build the command
        let mut cmd_parts = vec![format!("\"{}\"", self.target.display())];
        cmd_parts.extend(self.args.iter().cloned());
        cmd_parts.push("\"$@\"".to_string());

        lines.push(format!("exec {}", cmd_parts.join(" ")));

        lines.join("\n")
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
        std::fs::write(&shim_file, &content)
            .with_context(|| format!("Failed to write shim to {}", shim_file.display()))?;

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&shim_file)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&shim_file, perms)?;
        }

        debug!("Created shim at {}", shim_file.display());
        Ok(shim_file)
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_shim_file_name_windows() {
        let platform = Platform::new(crate::Os::Windows, crate::Arch::X86_64);
        let shim = Shim::new("bunx", "/path/to/bun");

        assert_eq!(shim.file_name(&platform), "bunx.cmd");
    }

    #[test]
    fn test_shim_file_name_unix() {
        let platform = Platform::new(crate::Os::Linux, crate::Arch::X86_64);
        let shim = Shim::new("bunx", "/path/to/bun");

        assert_eq!(shim.file_name(&platform), "bunx");
    }

    #[test]
    fn test_batch_content() {
        let shim = Shim::new("bunx", "/path/to/bun.exe").with_args(&["x"]);

        let content = shim.generate_batch();

        assert!(content.contains("@echo off"));
        assert!(content.contains("/path/to/bun.exe"));
        assert!(content.contains(" x %*"));
    }

    #[test]
    fn test_shell_content() {
        let shim = Shim::new("bunx", "/path/to/bun").with_args(&["x"]);

        let content = shim.generate_shell();

        assert!(content.contains("#!/bin/sh"));
        assert!(content.contains("exec"));
        assert!(content.contains("/path/to/bun"));
        assert!(content.contains("\" x \"$@\""));
    }

    #[test]
    fn test_shim_with_env() {
        let shim = Shim::new("test", "/path/to/exe")
            .with_env("FOO", "bar")
            .with_env("BAZ", "qux");

        let shell_content = shim.generate_shell();

        assert!(shell_content.contains("FOO='bar'"));
        assert!(shell_content.contains("export FOO"));
        assert!(shell_content.contains("BAZ='qux'"));
        assert!(shell_content.contains("export BAZ"));

        let batch_content = shim.generate_batch();

        assert!(batch_content.contains("set FOO=bar"));
        assert!(batch_content.contains("set BAZ=qux"));
    }

    #[test]
    fn test_shim_with_working_dir() {
        let shim = Shim::new("test", "/path/to/exe").with_working_dir("/working/dir");

        let shell_content = shim.generate_shell();
        assert!(shell_content.contains("cd \"/working/dir\""));

        let batch_content = shim.generate_batch();
        assert!(batch_content.contains("cd /d \"/working/dir\""));
    }

    #[test]
    fn test_create_shim_file() {
        let temp = TempDir::new().unwrap();
        let platform = Platform::current();

        let shim = Shim::new("test-shim", temp.path().join("target")).with_args(&["arg1"]);

        let path = shim.create(temp.path(), &platform).unwrap();

        assert!(path.exists());

        let content = std::fs::read_to_string(&path).unwrap();

        #[cfg(windows)]
        assert!(content.contains("@echo off"));

        #[cfg(not(windows))]
        {
            assert!(content.contains("#!/bin/sh"));

            // Check executable permission
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::metadata(&path).unwrap().permissions();
            assert!(perms.mode() & 0o111 != 0);
        }
    }

    #[test]
    fn test_shim_builder() {
        let temp = TempDir::new().unwrap();

        let paths = ShimBuilder::new()
            .dir(temp.path())
            .forward("shim1", temp.path().join("exe1"), &["arg1"])
            .forward("shim2", temp.path().join("exe2"), &["arg2", "arg3"])
            .build()
            .unwrap();

        assert_eq!(paths.len(), 2);

        #[cfg(windows)]
        {
            assert!(paths[0].ends_with("shim1.cmd"));
            assert!(paths[1].ends_with("shim2.cmd"));
        }

        #[cfg(not(windows))]
        {
            assert!(paths[0].ends_with("shim1"));
            assert!(paths[1].ends_with("shim2"));
        }
    }
}
