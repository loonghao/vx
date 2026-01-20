//! Post-install Normalizer (RFC 0022)
//!
//! This module provides functionality to normalize the installation directory
//! structure after extracting/downloading a runtime, ensuring a consistent
//! layout across all runtimes.

use anyhow::{Context, Result};
use glob::glob;
use std::path::{Path, PathBuf};
use tracing::{debug, trace, warn};
use vx_manifest::{
    AliasNormalize, DirectoryNormalize, EffectiveNormalizeConfig, ExecutableNormalize,
    NormalizeAction, NormalizeConfig,
};

/// Context for variable expansion in normalize rules
#[derive(Debug, Clone)]
pub struct NormalizeContext {
    /// Runtime version
    pub version: String,
    /// Runtime name
    pub name: String,
}

impl NormalizeContext {
    /// Create a new normalize context
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
        }
    }

    /// Expand variables in a template string
    /// Supported variables: {version}, {name}, {ext}
    pub fn expand(&self, template: &str) -> String {
        let ext = if cfg!(windows) { ".exe" } else { "" };

        template
            .replace("{version}", &self.version)
            .replace("{name}", &self.name)
            .replace("{ext}", ext)
    }
}

/// Result of normalization
#[derive(Debug, Default)]
pub struct NormalizeResult {
    /// Executables that were normalized
    pub executables_normalized: Vec<String>,
    /// Directories that were normalized
    pub directories_normalized: Vec<String>,
    /// Aliases that were created
    pub aliases_created: Vec<String>,
    /// Errors encountered (non-fatal)
    pub warnings: Vec<String>,
}

impl NormalizeResult {
    /// Check if any normalization was performed
    pub fn has_changes(&self) -> bool {
        !self.executables_normalized.is_empty()
            || !self.directories_normalized.is_empty()
            || !self.aliases_created.is_empty()
    }

    /// Get a summary string
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if !self.executables_normalized.is_empty() {
            parts.push(format!("{} executables", self.executables_normalized.len()));
        }
        if !self.directories_normalized.is_empty() {
            parts.push(format!("{} directories", self.directories_normalized.len()));
        }
        if !self.aliases_created.is_empty() {
            parts.push(format!("{} aliases", self.aliases_created.len()));
        }
        if parts.is_empty() {
            "no changes".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// Normalizer for post-install processing
pub struct Normalizer;

impl Normalizer {
    /// Apply normalization to an installed runtime
    ///
    /// # Arguments
    /// * `install_path` - The root installation directory
    /// * `config` - The normalization configuration
    /// * `context` - Context for variable expansion
    pub fn normalize(
        install_path: &Path,
        config: &NormalizeConfig,
        context: &NormalizeContext,
    ) -> Result<NormalizeResult> {
        let effective = config.get_effective_config();

        if !effective.enabled {
            debug!("Normalization disabled for {}", context.name);
            return Ok(NormalizeResult::default());
        }

        if !effective.has_rules() {
            trace!("No normalization rules for {}", context.name);
            return Ok(NormalizeResult::default());
        }

        debug!(
            "Normalizing {} with {} executable rules, {} directory rules, {} aliases",
            context.name,
            effective.executables.len(),
            effective.directories.len(),
            effective.aliases.len()
        );

        Self::apply_config(install_path, &effective, context)
    }

    /// Apply effective configuration
    fn apply_config(
        install_path: &Path,
        effective: &EffectiveNormalizeConfig,
        context: &NormalizeContext,
    ) -> Result<NormalizeResult> {
        let mut result = NormalizeResult::default();

        // Ensure bin directory exists
        let bin_dir = install_path.join("bin");
        if !bin_dir.exists() {
            std::fs::create_dir_all(&bin_dir).with_context(|| {
                format!("Failed to create bin directory: {}", bin_dir.display())
            })?;
        }

        // Process executables
        for rule in &effective.executables {
            match Self::process_executable(install_path, &bin_dir, rule, context) {
                Ok(Some(target)) => {
                    result.executables_normalized.push(target);
                }
                Ok(None) => {
                    trace!("No match for executable pattern: {}", rule.source);
                }
                Err(e) => {
                    let msg = format!("Failed to normalize executable {}: {}", rule.source, e);
                    warn!("{}", msg);
                    result.warnings.push(msg);
                }
            }
        }

        // Process directories
        for rule in &effective.directories {
            match Self::process_directory(install_path, rule, context) {
                Ok(Some(target)) => {
                    result.directories_normalized.push(target);
                }
                Ok(None) => {
                    trace!("No match for directory pattern: {}", rule.source);
                }
                Err(e) => {
                    let msg = format!("Failed to normalize directory {}: {}", rule.source, e);
                    warn!("{}", msg);
                    result.warnings.push(msg);
                }
            }
        }

        // Create aliases
        for alias in &effective.aliases {
            match Self::create_alias(&bin_dir, alias) {
                Ok(true) => {
                    result.aliases_created.push(alias.name.clone());
                }
                Ok(false) => {
                    trace!("Alias target not found: {} -> {}", alias.name, alias.target);
                }
                Err(e) => {
                    let msg = format!("Failed to create alias {}: {}", alias.name, e);
                    warn!("{}", msg);
                    result.warnings.push(msg);
                }
            }
        }

        Ok(result)
    }

    /// Process a single executable rule
    fn process_executable(
        install_path: &Path,
        bin_dir: &Path,
        rule: &ExecutableNormalize,
        context: &NormalizeContext,
    ) -> Result<Option<String>> {
        let source_pattern = context.expand(&rule.source);
        let target_name = context.expand(&rule.target);
        let target_path = bin_dir.join(&target_name);

        // Skip if target already exists
        if target_path.exists() {
            trace!("Target already exists: {}", target_path.display());
            return Ok(Some(target_name));
        }

        // Find matching source files
        let full_pattern = install_path.join(&source_pattern);
        let pattern_str = full_pattern.to_string_lossy();

        debug!("Looking for pattern: {}", pattern_str);

        for entry in glob(&pattern_str)? {
            let source_path = entry?;

            if source_path.is_file() {
                debug!(
                    "Normalizing executable: {} -> {}",
                    source_path.display(),
                    target_path.display()
                );

                Self::apply_action(&source_path, &target_path, &rule.action)?;

                // Set permissions on Unix
                #[cfg(unix)]
                if let Some(perms) = &rule.permissions {
                    Self::set_permissions(&target_path, perms)?;
                }

                return Ok(Some(target_name));
            }
        }

        Ok(None)
    }

    /// Process a single directory rule
    fn process_directory(
        install_path: &Path,
        rule: &DirectoryNormalize,
        context: &NormalizeContext,
    ) -> Result<Option<String>> {
        let source_pattern = context.expand(&rule.source);
        let target_name = context.expand(&rule.target);
        let target_path = install_path.join(&target_name);

        // Skip if target already exists
        if target_path.exists() {
            trace!("Target already exists: {}", target_path.display());
            return Ok(Some(target_name));
        }

        // Find matching source directories
        let full_pattern = install_path.join(&source_pattern);
        let pattern_str = full_pattern.to_string_lossy();

        for entry in glob(&pattern_str)? {
            let source_path = entry?;

            if source_path.is_dir() {
                debug!(
                    "Normalizing directory: {} -> {}",
                    source_path.display(),
                    target_path.display()
                );

                // Ensure parent directory exists
                if let Some(parent) = target_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                Self::apply_action(&source_path, &target_path, &rule.action)?;
                return Ok(Some(target_name));
            }
        }

        Ok(None)
    }

    /// Create an alias (symlink) for an executable
    fn create_alias(bin_dir: &Path, alias: &AliasNormalize) -> Result<bool> {
        let target_path = bin_dir.join(&alias.target);
        let alias_path = bin_dir.join(&alias.name);

        // Add extension on Windows
        #[cfg(windows)]
        let (target_path, alias_path) = {
            let target = if !target_path.exists() && !alias.target.ends_with(".exe") {
                bin_dir.join(format!("{}.exe", alias.target))
            } else {
                target_path
            };
            let alias = if !alias.name.ends_with(".exe") {
                bin_dir.join(format!("{}.exe", alias.name))
            } else {
                alias_path
            };
            (target, alias)
        };

        if !target_path.exists() {
            return Ok(false);
        }

        if alias_path.exists() {
            trace!("Alias already exists: {}", alias_path.display());
            return Ok(true);
        }

        debug!(
            "Creating alias: {} -> {}",
            alias_path.display(),
            target_path.display()
        );

        Self::create_link(&target_path, &alias_path)?;
        Ok(true)
    }

    /// Apply an action (link, copy, move, hardlink) to a file or directory
    fn apply_action(source: &Path, target: &Path, action: &NormalizeAction) -> Result<()> {
        match action {
            NormalizeAction::Link => Self::create_link(source, target),
            NormalizeAction::HardLink => Self::create_hardlink(source, target),
            NormalizeAction::Copy => Self::copy_recursive(source, target),
            NormalizeAction::Move => std::fs::rename(source, target).with_context(|| {
                format!(
                    "Failed to move {} to {}",
                    source.display(),
                    target.display()
                )
            }),
        }
    }

    /// Create a symbolic link
    fn create_link(source: &Path, target: &Path) -> Result<()> {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(source, target).with_context(|| {
                format!(
                    "Failed to create symlink {} -> {}",
                    target.display(),
                    source.display()
                )
            })
        }

        #[cfg(windows)]
        {
            // Try symlink first, fall back to hard link or copy
            let result = if source.is_dir() {
                std::os::windows::fs::symlink_dir(source, target)
            } else {
                std::os::windows::fs::symlink_file(source, target)
            };

            match result {
                Ok(()) => Ok(()),
                Err(e) => {
                    // Symlink may fail on Windows without developer mode
                    // Fall back to hard link for files, or copy for directories
                    debug!(
                        "Symlink failed ({}), trying fallback for {}",
                        e,
                        source.display()
                    );

                    if source.is_file() {
                        Self::create_hardlink(source, target)
                    } else {
                        Self::copy_recursive(source, target)
                    }
                }
            }
        }
    }

    /// Create a hard link
    fn create_hardlink(source: &Path, target: &Path) -> Result<()> {
        if source.is_file() {
            std::fs::hard_link(source, target).with_context(|| {
                format!(
                    "Failed to create hard link {} -> {}",
                    target.display(),
                    source.display()
                )
            })
        } else {
            // Hard links don't work for directories, use symlink or copy
            Self::create_link(source, target)
        }
    }

    /// Copy a file or directory recursively
    fn copy_recursive(source: &Path, target: &Path) -> Result<()> {
        if source.is_dir() {
            Self::copy_dir_all(source, target)
        } else {
            std::fs::copy(source, target).map(|_| ()).with_context(|| {
                format!(
                    "Failed to copy {} to {}",
                    source.display(),
                    target.display()
                )
            })
        }
    }

    /// Copy a directory recursively
    fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
        std::fs::create_dir_all(dst)?;

        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if ty.is_dir() {
                Self::copy_dir_all(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }

    /// Set file permissions on Unix
    #[cfg(unix)]
    fn set_permissions(path: &Path, mode: &str) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let mode = u32::from_str_radix(mode, 8)
            .with_context(|| format!("Invalid permission mode: {}", mode))?;

        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
            .with_context(|| format!("Failed to set permissions on {}", path.display()))?;

        Ok(())
    }
}

/// Find files matching a glob pattern
pub fn find_matching_files(base_path: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let full_pattern = base_path.join(pattern);
    let pattern_str = full_pattern.to_string_lossy();

    let mut results = Vec::new();
    for entry in glob(&pattern_str)? {
        results.push(entry?);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_normalize_context_expand() {
        let ctx = NormalizeContext::new("magick", "7.1.1");

        assert_eq!(ctx.expand("{name}-{version}"), "magick-7.1.1");

        #[cfg(windows)]
        assert_eq!(ctx.expand("{name}{ext}"), "magick.exe");

        #[cfg(not(windows))]
        assert_eq!(ctx.expand("{name}{ext}"), "magick");
    }

    #[test]
    fn test_normalize_result_summary() {
        let mut result = NormalizeResult::default();
        assert_eq!(result.summary(), "no changes");

        result.executables_normalized.push("tool".to_string());
        assert_eq!(result.summary(), "1 executables");

        result.aliases_created.push("alias".to_string());
        assert_eq!(result.summary(), "1 executables, 1 aliases");
    }

    #[test]
    fn test_normalize_creates_bin_dir() {
        let temp = TempDir::new().unwrap();
        let install_path = temp.path();

        let mut config = NormalizeConfig::default();
        config.enabled = true;
        // Add a rule to trigger bin directory creation
        config.executables.push(ExecutableNormalize {
            source: "nonexistent".to_string(),
            target: "nonexistent".to_string(),
            action: NormalizeAction::Link,
            permissions: None,
            platforms: None,
        });

        let ctx = NormalizeContext::new("test", "1.0.0");

        let result = Normalizer::normalize(install_path, &config, &ctx).unwrap();

        // bin directory should be created when there are rules (even if none match)
        assert!(install_path.join("bin").exists());
        assert!(!result.has_changes()); // No source files matched
    }

    #[test]
    fn test_normalize_executable_link() {
        let temp = TempDir::new().unwrap();
        let install_path = temp.path();

        // Create source file
        let source_dir = install_path.join("nested");
        std::fs::create_dir_all(&source_dir).unwrap();
        std::fs::write(source_dir.join("tool.exe"), "binary").unwrap();

        let mut config = NormalizeConfig::default();
        config.enabled = true;
        config.executables.push(ExecutableNormalize {
            source: "nested/tool.exe".to_string(),
            target: "tool.exe".to_string(),
            action: NormalizeAction::Copy, // Use copy for test portability
            permissions: None,
            platforms: None,
        });

        let ctx = NormalizeContext::new("tool", "1.0.0");

        let result = Normalizer::normalize(install_path, &config, &ctx).unwrap();

        assert!(result
            .executables_normalized
            .contains(&"tool.exe".to_string()));
        assert!(install_path.join("bin/tool.exe").exists());
    }
}
