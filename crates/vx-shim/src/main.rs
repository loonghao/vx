//! VX Shim - Cross-platform executable shim for vx tool manager
//!
//! This is a lightweight executable that acts as a proxy to other executables,
//! similar to scoop-better-shimexe but written in Rust for cross-platform support.

use anyhow::{Context, Result};
use std::env;
use std::path::{Path, PathBuf};
use std::process;

mod config;
mod executor;
mod platform;
mod shim;

use config::ShimConfig;
use executor::Executor;

fn main() {
    let result = run();

    match result {
        Ok(exit_code) => process::exit(exit_code),
        Err(e) => {
            eprintln!("vx-shim error: {}", e);
            process::exit(1);
        }
    }
}

fn run() -> Result<i32> {
    // Get the current executable path
    let current_exe = env::current_exe().context("Failed to get current executable path")?;

    // Find the corresponding .shim file
    let shim_file = find_shim_file(&current_exe)?;

    // Load shim configuration
    let config = ShimConfig::load(&shim_file)
        .with_context(|| format!("Failed to load shim config from {}", shim_file.display()))?;

    // Get command line arguments (excluding the program name)
    let args: Vec<String> = env::args().skip(1).collect();

    // Create executor and run the target program
    let executor = Executor::new(config);
    executor
        .execute(&args)
        .context("Failed to execute target program")
}

fn find_shim_file(exe_path: &Path) -> Result<PathBuf> {
    // Replace .exe extension with .shim (or add .shim if no extension)
    let mut shim_path = exe_path.to_path_buf();

    if let Some(extension) = shim_path.extension() {
        if extension == "exe" {
            shim_path.set_extension("shim");
        } else {
            // Add .shim to the existing extension
            let mut new_extension = extension.to_os_string();
            new_extension.push(".shim");
            shim_path.set_extension(new_extension);
        }
    } else {
        // No extension, just add .shim
        let mut new_name = shim_path
            .file_name()
            .context("Invalid executable path")?
            .to_os_string();
        new_name.push(".shim");
        shim_path.set_file_name(new_name);
    }

    if !shim_path.exists() {
        anyhow::bail!("Shim file not found: {}", shim_path.display());
    }

    Ok(shim_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_shim_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Test with .exe extension
        let exe_path = temp_path.join("test.exe");
        let shim_path = temp_path.join("test.shim");

        // Create the shim file
        fs::write(&shim_path, "path = /bin/echo\n").unwrap();

        let result = find_shim_file(&exe_path).unwrap();
        assert_eq!(result, shim_path);
    }

    #[test]
    fn test_find_shim_file_no_extension() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Test without extension
        let exe_path = temp_path.join("test");
        let shim_path = temp_path.join("test.shim");

        // Create the shim file
        fs::write(&shim_path, "path = /bin/echo\n").unwrap();

        let result = find_shim_file(&exe_path).unwrap();
        assert_eq!(result, shim_path);
    }

    #[test]
    fn test_find_shim_file_missing() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let exe_path = temp_path.join("test.exe");

        let result = find_shim_file(&exe_path);
        assert!(result.is_err());
    }
}
