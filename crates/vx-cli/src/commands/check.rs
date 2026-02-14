//! Check command implementation
//!
//! Checks version constraints and tool availability for the project.
//! This command is part of RFC 0023: Version Range Locking System.
//!
//! ## Features
//!
//! - Verify all tools meet version constraints
//! - Detect version conflicts between tools
//! - Check if tools are installed
//! - Validate against provider version ranges
//!
//! ## Usage
//!
//! ```bash
//! # Check all tools in project
//! vx check
//!
//! # Check specific tool
//! vx check node
//!
//! # Detailed output
//! vx check --detailed
//!
//! # Quiet mode (exit code only)
//! vx check --quiet
//! ```

use crate::commands::common::{ToolStatus, check_tools_status};
use crate::commands::setup::{find_vx_config, parse_vx_config};
use crate::ui::UI;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use vx_paths::project::LOCK_FILE_NAME;
use vx_resolver::{
    ConflictDetector, LockFile, Version, VersionRangeConfig, VersionRangeResolver, VersionRequest,
};
use vx_runtime::ProviderRegistry;

/// Handle the check command
pub async fn handle(
    registry: &ProviderRegistry,
    tool: Option<String>,
    detailed: bool,
    quiet: bool,
) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    // Find vx.toml
    let config_path = match find_vx_config(&current_dir) {
        Ok(p) => p,
        Err(_) => {
            if !quiet {
                UI::warn("No vx.toml found in current directory or parents");
                UI::hint("Run 'vx init' to create a project configuration");
            }
            std::process::exit(1);
        }
    };

    let config = parse_vx_config(&config_path)?;

    if config.tools.is_empty() {
        if !quiet {
            UI::info("No tools configured in vx.toml");
        }
        return Ok(());
    }

    // Get tools to check
    let tools_to_check: HashMap<String, String> = if let Some(ref name) = tool {
        if let Some(version) = config.tools.get(name) {
            let mut map = HashMap::new();
            map.insert(name.clone(), version.clone());
            map
        } else {
            if !quiet {
                UI::error(&format!("Tool '{}' not found in vx.toml", name));
            }
            std::process::exit(1);
        }
    } else {
        config.tools.clone()
    };

    // Check lock file
    let project_root = config_path.parent().unwrap_or(&current_dir);
    let lock_path = project_root.join(LOCK_FILE_NAME);
    let lockfile = if lock_path.exists() {
        LockFile::load(&lock_path).ok()
    } else {
        None
    };

    // Check results
    let mut all_ok = true;
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    // Check tool status (installed/missing)
    let statuses = check_tools_status(&tools_to_check)?;

    if !quiet && detailed {
        println!("Checking project tools...\n");
    }

    for (name, config_version, status, path, _) in &statuses {
        let mut tool_ok = true;
        let mut tool_warnings = Vec::new();
        let mut tool_errors = Vec::new();

        // Check installation status
        match status {
            ToolStatus::Installed => {
                if detailed && !quiet {
                    println!(
                        "✓ {} {} (installed at {})",
                        name,
                        config_version,
                        path.as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_default()
                    );
                }
            }
            ToolStatus::SystemFallback => {
                if detailed && !quiet {
                    println!("⚠ {} {} (using system version)", name, config_version);
                }
                tool_warnings.push("Using system fallback version".to_string());
            }
            ToolStatus::NotInstalled => {
                tool_errors.push(format!("{} is not installed", name));
                tool_ok = false;
            }
        }

        // Check lock file consistency
        if let Some(ref lock) = lockfile {
            if let Some(locked) = lock.get_tool(name) {
                if &locked.resolved_from != config_version {
                    tool_warnings.push(format!(
                        "Version mismatch: vx.toml has '{}', lock file has '{}'",
                        config_version, locked.resolved_from
                    ));
                }

                // Check version range bounds if provider config is available
                if let Some(version) = Version::parse(&locked.version) {
                    let range_config = get_provider_version_range_config(registry, name);
                    if !range_config.is_empty() {
                        let bounds_result =
                            VersionRangeResolver::check_bounds(&version, &range_config);
                        if !bounds_result.is_ok() {
                            tool_errors.extend(bounds_result.errors.clone());
                            tool_ok = false;
                        }
                        if bounds_result.has_warnings() {
                            tool_warnings.extend(bounds_result.warnings.clone());
                        }
                    }
                }
            } else {
                tool_warnings.push(format!("{} is not in lock file", name));
            }
        }

        // Show tool-specific results in detailed mode
        if detailed && !quiet {
            for warn in &tool_warnings {
                println!("  ⚠ {}", warn);
            }
            for err in &tool_errors {
                println!("  ✗ {}", err);
            }
        }

        if !tool_ok {
            all_ok = false;
        }
        warnings.extend(tool_warnings);
        errors.extend(tool_errors);
    }

    // Check for version conflicts
    let conflict_detector = ConflictDetector::with_defaults();
    let tools_with_requests: Vec<(String, VersionRequest)> = tools_to_check
        .iter()
        .map(|(name, version)| (name.clone(), VersionRequest::parse(version)))
        .collect();

    if let Ok(conflicts) = conflict_detector.detect_conflicts(&tools_with_requests) {
        if !conflicts.is_empty() {
            all_ok = false;
            for conflict in &conflicts {
                errors.push(format!("Version conflict for {}", conflict.runtime));
                if detailed && !quiet {
                    println!("\n{}", conflict);
                }
            }
        }
    }

    // Summary output
    if !quiet {
        println!();
        if all_ok && warnings.is_empty() {
            UI::success("✓ All version constraints satisfied");
        } else if all_ok && !warnings.is_empty() {
            UI::warn(&format!(
                "⚠ All constraints satisfied with {} warning(s)",
                warnings.len()
            ));
            if !detailed {
                UI::hint("Run 'vx check --detailed' for more information");
            }
        } else {
            UI::error(&format!(
                "✗ {} error(s) and {} warning(s) found",
                errors.len(),
                warnings.len()
            ));
            if !detailed {
                for err in &errors {
                    println!("  - {}", err);
                }
                UI::hint("Run 'vx check --detailed' for more information");
            }
        }
    }

    // Exit with appropriate code
    if !all_ok {
        std::process::exit(1);
    }

    Ok(())
}

/// Get version range configuration from provider
///
/// Note: In the current implementation, version range configs are typically
/// loaded dynamically from provider manifests during resolution. This function
/// provides a simplified approach for the check command.
fn get_provider_version_range_config(
    _registry: &ProviderRegistry,
    _tool_name: &str,
) -> VersionRangeConfig {
    // Version range configuration is currently obtained dynamically
    // from provider manifests. For basic checks, we return a default config.
    // Future enhancement: Read from provider manifest files directly.
    VersionRangeConfig::default()
}
