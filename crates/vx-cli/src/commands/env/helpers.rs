//! Helper functions for environment management

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use vx_paths::{find_config_file, link, project_env_dir, LinkStrategy, PathManager};

/// Get the project environment directory if in a project with vx.toml
pub fn get_project_env_dir() -> Option<PathBuf> {
    let current_dir = env::current_dir().ok()?;

    if find_config_file(&current_dir).is_some() {
        Some(project_env_dir(&current_dir))
    } else {
        None
    }
}

/// Set the default global environment
pub fn set_default_env(name: &str) -> Result<()> {
    let path_manager = PathManager::new()?;
    let config_dir = path_manager.config_dir();
    let default_file = config_dir.join("default-env");

    std::fs::create_dir_all(config_dir)?;
    std::fs::write(&default_file, name)?;

    Ok(())
}

/// Get the current default global environment
pub fn get_default_env() -> Result<String> {
    let path_manager = PathManager::new()?;
    let config_dir = path_manager.config_dir();
    let default_file = config_dir.join("default-env");

    if default_file.exists() {
        let name = std::fs::read_to_string(&default_file)?;
        Ok(name.trim().to_string())
    } else {
        Ok("default".to_string())
    }
}

/// List runtimes in an environment directory
pub fn list_env_runtimes(env_dir: &Path) -> Result<Vec<String>> {
    let mut runtimes = Vec::new();

    if !env_dir.exists() {
        return Ok(runtimes);
    }

    for entry in std::fs::read_dir(env_dir)? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            runtimes.push(name.to_string());
        }
    }

    runtimes.sort();
    Ok(runtimes)
}

/// Parse runtime@version string
pub fn parse_runtime_version(s: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = s.splitn(2, '@').collect();
    if parts.len() != 2 {
        anyhow::bail!(
            "Invalid format '{}'. Expected '<runtime>@<version>' (e.g., node@20.0.0)",
            s
        );
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

/// Clone environment contents (symlinks)
pub fn clone_env_contents(source: &Path, target: &Path) -> Result<()> {
    if !source.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());

        if source_path.is_symlink() {
            // Recreate symlink pointing to the same target
            let link_target = std::fs::read_link(&source_path)?;
            link::create_link(&link_target, &target_path, LinkStrategy::SymLink)
                .context("Failed to create symlink")?;
        } else if source_path.is_file() {
            std::fs::copy(&source_path, &target_path)?;
        } else if source_path.is_dir() {
            std::fs::create_dir_all(&target_path)?;
            clone_env_contents(&source_path, &target_path)?;
        }
    }

    Ok(())
}

/// Resolve environment directory for shell command
pub fn resolve_env_for_shell(
    name: Option<&str>,
    global: bool,
    path_manager: &PathManager,
) -> Result<(PathBuf, String)> {
    if global {
        let env_name =
            name.ok_or_else(|| anyhow::anyhow!("Environment name is required with --global"))?;

        if !path_manager.env_exists(env_name) {
            anyhow::bail!(
                "Global environment '{}' does not exist. Create it with 'vx env create --global {}'",
                env_name,
                env_name
            );
        }

        Ok((path_manager.env_dir(env_name), env_name.to_string()))
    } else if let Some(env_name) = name {
        // Check global environment by name
        if path_manager.env_exists(env_name) {
            Ok((path_manager.env_dir(env_name), env_name.to_string()))
        } else {
            anyhow::bail!(
                "Environment '{}' does not exist. Create it with 'vx env create --global {}'",
                env_name,
                env_name
            );
        }
    } else {
        // Use project environment if available
        if let Some(project_env) = get_project_env_dir() {
            if project_env.exists() {
                return Ok((project_env, "project".to_string()));
            }
        }

        // Fall back to default global environment
        let default_env = get_default_env()?;
        let env_dir = path_manager.env_dir(&default_env);

        if !env_dir.exists() {
            anyhow::bail!(
                "No environment found. Create one with 'vx env create' or 'vx env create --global <name>'"
            );
        }

        Ok((env_dir, default_env))
    }
}

/// Build tools map from environment directory symlinks
pub fn build_tools_from_env_dir(
    env_dir: &Path,
    _path_manager: &PathManager,
) -> Result<HashMap<String, String>> {
    let mut tools = HashMap::new();

    if !env_dir.exists() {
        return Ok(tools);
    }

    for entry in std::fs::read_dir(env_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_symlink() && !path.is_dir() {
            continue;
        }

        let tool_name = entry.file_name().to_string_lossy().to_string();

        // Try to extract version from symlink target or directory name
        let version = if path.is_symlink() {
            if let Ok(target) = std::fs::read_link(&path) {
                // Extract version from store path: ~/.vx/store/<tool>/<version>/...
                extract_version_from_store_path(&target, &tool_name)
            } else {
                "latest".to_string()
            }
        } else {
            // For directories, try to detect version
            "latest".to_string()
        };

        tools.insert(tool_name, version);
    }

    Ok(tools)
}

/// Extract version from store path
fn extract_version_from_store_path(store_path: &Path, tool_name: &str) -> String {
    // Store path format: ~/.vx/store/<tool>/<version>/...
    let path_str = store_path.to_string_lossy();

    if let Some(store_idx) = path_str.find("store") {
        let after_store = &path_str[store_idx + 6..]; // Skip "store/"
        let parts: Vec<&str> = after_store.split(['/', '\\']).collect();

        // Expected: [<tool>, <version>, ...]
        if parts.len() >= 2 && parts[0] == tool_name {
            return parts[1].to_string();
        }
    }

    "latest".to_string()
}
