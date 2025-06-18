//! Shimexe integration for vx tool manager
//!
//! This module provides integration between vx-core and shimexe-core,
//! enabling seamless tool version switching through modern shim technology.

use crate::{Result, VxEnvironment};
use anyhow::Context;
use std::fs;
use std::path::{Path, PathBuf};

/// Manages shim creation and tool version switching for vx using shimexe-core
pub struct VxShimexeManager {
    /// Path to the shim directory
    shim_dir: PathBuf,
    /// Environment instance for configuration
    #[allow(dead_code)]
    environment: VxEnvironment,
}

impl VxShimexeManager {
    /// Create a new VxShimexeManager
    pub fn new(environment: VxEnvironment) -> Result<Self> {
        let shim_dir = environment.shim_dir()?;

        // Ensure shim directory exists
        fs::create_dir_all(&shim_dir)
            .with_context(|| format!("Failed to create shim directory: {}", shim_dir.display()))?;

        Ok(Self {
            shim_dir,
            environment,
        })
    }

    /// Create a shim for a tool version
    pub fn create_tool_shim(
        &self,
        tool_name: &str,
        tool_path: &Path,
        version: &str,
        args: Option<&str>,
    ) -> Result<PathBuf> {
        // Create shim configuration file
        let config_content = self.create_shim_config_content(tool_name, tool_path, args)?;
        let config_path = self.shim_dir.join(format!("{}.shim", tool_name));

        // Write the configuration file
        fs::write(&config_path, config_content)
            .with_context(|| format!("Failed to write shim config for {}", tool_name))?;

        // Create the actual shim executable using shimexe-core
        let shim_path = self.create_shim_executable(tool_name, &config_path)?;

        // Store metadata about this shim
        self.store_shim_metadata(tool_name, version, &shim_path)?;

        Ok(shim_path)
    }

    /// List all managed shims
    pub fn list_shims(&self) -> Result<Vec<String>> {
        let mut shims = Vec::new();

        if !self.shim_dir.exists() {
            return Ok(shims);
        }

        for entry in fs::read_dir(&self.shim_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(extension) = path.extension() {
                if extension == "shim" {
                    if let Some(stem) = path.file_stem() {
                        shims.push(stem.to_string_lossy().to_string());
                    }
                }
            }
        }

        shims.sort();
        Ok(shims)
    }

    /// Remove a shim
    pub fn remove_shim(&self, tool_name: &str) -> Result<()> {
        // Remove shim configuration file
        let config_path = self.shim_dir.join(format!("{}.shim", tool_name));
        if config_path.exists() {
            fs::remove_file(&config_path)
                .with_context(|| format!("Failed to remove shim config for {}", tool_name))?;
        }

        // Remove shim executable
        let shim_path = if cfg!(windows) {
            self.shim_dir.join(format!("{}.exe", tool_name))
        } else {
            self.shim_dir.join(tool_name)
        };

        if shim_path.exists() {
            fs::remove_file(&shim_path)
                .with_context(|| format!("Failed to remove shim executable for {}", tool_name))?;
        }

        // Remove metadata
        self.remove_shim_metadata(tool_name)?;

        Ok(())
    }

    /// Create the actual shim executable
    fn create_shim_executable(&self, tool_name: &str, config_path: &Path) -> Result<PathBuf> {
        // For now, we'll create a simple wrapper script that uses shimexe-core
        // In a real implementation, we would use shimexe-core's binary creation capabilities
        let shim_path = if cfg!(windows) {
            self.shim_dir.join(format!("{}.exe", tool_name))
        } else {
            self.shim_dir.join(tool_name)
        };

        // Create a simple wrapper that calls shimexe-core
        let wrapper_content = if cfg!(windows) {
            format!(
                r#"@echo off
shimexe-core execute "{}"
"#,
                config_path.display()
            )
        } else {
            format!(
                r#"#!/bin/bash
shimexe-core execute "{}"
"#,
                config_path.display()
            )
        };

        fs::write(&shim_path, wrapper_content)
            .with_context(|| format!("Failed to create shim executable for {}", tool_name))?;

        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&shim_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&shim_path, perms)?;
        }

        Ok(shim_path)
    }

    /// Create shim configuration content for shimexe-core
    fn create_shim_config_content(
        &self,
        tool_name: &str,
        tool_path: &Path,
        args: Option<&str>,
    ) -> Result<String> {
        let mut config = format!(
            r#"name = "{}"

[shim]
name = "{}"
path = "{}"
"#,
            tool_name,
            tool_name,
            tool_path.display()
        );

        if let Some(args_str) = args {
            config.push_str(&format!("args = [\"{}\"]\n", args_str));
        }

        Ok(config)
    }

    /// Store metadata about a shim
    fn store_shim_metadata(&self, tool_name: &str, version: &str, shim_path: &Path) -> Result<()> {
        let metadata_dir = self.shim_dir.join(".vx-metadata");
        fs::create_dir_all(&metadata_dir)?;

        let metadata_file = metadata_dir.join(format!("{}.toml", tool_name));
        let metadata_content = format!(
            r#"tool_name = "{}"
version = "{}"
shim_path = "{}"
created_at = "{}"
"#,
            tool_name,
            version,
            shim_path.display(),
            chrono::Utc::now().to_rfc3339()
        );

        fs::write(&metadata_file, metadata_content)
            .with_context(|| format!("Failed to write metadata for {}", tool_name))?;

        Ok(())
    }

    /// Remove metadata for a shim
    fn remove_shim_metadata(&self, tool_name: &str) -> Result<()> {
        let metadata_dir = self.shim_dir.join(".vx-metadata");
        let metadata_file = metadata_dir.join(format!("{}.toml", tool_name));

        if metadata_file.exists() {
            fs::remove_file(&metadata_file)
                .with_context(|| format!("Failed to remove metadata for {}", tool_name))?;
        }

        Ok(())
    }
}

/// Test shimexe-core API to understand its capabilities
pub fn test_shimexe_api() -> Result<()> {
    println!("Testing shimexe-core API...");

    // Try to create a basic shim configuration
    let temp_dir = std::env::temp_dir().join("vx-shimexe-test");
    std::fs::create_dir_all(&temp_dir)?;

    // Let's explore what shimexe-core provides
    // First, let's try to use ShimConfig
    use shimexe_core::ShimConfig;

    // Try to create a config from a file path
    let config_path = temp_dir.join("test.shim");
    std::fs::write(
        &config_path,
        r#"name = "test-shim"

[shim]
name = "echo"
path = "echo"
"#,
    )?;

    match ShimConfig::from_file(&config_path) {
        Ok(config) => {
            println!("Successfully created ShimConfig: {:?}", config);
        }
        Err(e) => {
            println!("Failed to create ShimConfig: {}", e);
        }
    }

    println!("Shimexe-core API test completed");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shimexe_core_integration() -> Result<()> {
        test_shimexe_api()
    }
}
