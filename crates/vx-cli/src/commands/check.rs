//! Check command implementation
//!
//! Checks version constraints and tool availability for the project.
//! This command is part of RFC 0023: Version Range Locking System.
//! RFC 0035: Added unified structured output support.
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
//! # JSON output
//! vx check --json
//!
//! # Quiet mode (exit code only)
//! vx check --quiet
//! ```

use crate::cli::OutputFormat;
use crate::commands::common::{ToolStatus, check_tools_status, rust_toolchain_owner};
use crate::commands::setup::{find_vx_config, parse_vx_config};
use crate::output::{CheckOutput, OutputRenderer, RequirementStatus, RequirementStatusType};
use crate::ui::UI;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use vx_paths::project::LOCK_FILE_NAME;
use vx_resolver::{
    ConflictDetector, LockFile, RUSTUP_MANAGED, Version, VersionRangeConfig, VersionRangeResolver,
    VersionRequest, effective_toolchain_version, executable_for, find_rustup_executable,
    is_rust_toolchain_runtime, versions_conflict,
};
use vx_runtime::ProviderRegistry;

/// Handle the check command
pub async fn handle(
    registry: &ProviderRegistry,
    tool: Option<String>,
    detailed: bool,
    quiet: bool,
    format: OutputFormat,
) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    // Find vx.toml
    let config_path = match find_vx_config(&current_dir) {
        Ok(p) => p,
        Err(_) => {
            let output = CheckOutput {
                project_file: None,
                requirements: vec![],
                all_satisfied: false,
                missing_tools: vec![],
                warnings: vec!["No vx.toml found".to_string()],
                errors: vec!["No vx.toml found in current directory or parents".to_string()],
            };

            if !quiet {
                let renderer = OutputRenderer::new(format);
                if renderer.is_text() {
                    UI::warn("No vx.toml found in current directory or parents");
                    UI::hint("Run 'vx init' to create a project configuration");
                } else {
                    renderer.render(&output)?;
                }
            }
            std::process::exit(1);
        }
    };

    let config = parse_vx_config(&config_path)?;

    if config.tools.is_empty() {
        let output = CheckOutput {
            project_file: Some(config_path.display().to_string()),
            requirements: vec![],
            all_satisfied: true,
            missing_tools: vec![],
            warnings: vec!["No tools configured in vx.toml".to_string()],
            errors: vec![],
        };

        if !quiet {
            let renderer = OutputRenderer::new(format);
            if renderer.is_text() {
                UI::info("No tools configured in vx.toml");
            } else {
                renderer.render(&output)?;
            }
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
    let mut missing_tools = Vec::new();
    let mut requirements = Vec::new();

    // Check tool status (installed/missing)
    let statuses = check_tools_status(&tools_to_check)?;

    // Rust is the one tool vx does not get to own: a `rust-toolchain.toml`, an
    // exported `RUSTUP_TOOLCHAIN`, or `rust = "rustup-managed"` mean rustup decides.
    // `vx check` has to agree with the executor, or it reports a tool as missing that
    // will actually run fine — and, worse, stays silent about a pin nobody honours.
    let rust_owner =
        |name: &str, config_version: &str| rust_toolchain_owner(name, config_version, project_root);

    let pin_mismatch_severity = config
        .check
        .as_ref()
        .map(|check| check.pin_mismatch_severity())
        .unwrap_or_default();

    for (name, config_version, status, path, detected_version) in &statuses {
        let mut tool_ok = true;
        let mut tool_warnings = Vec::new();
        let mut tool_errors = Vec::new();

        let rust_owner = rust_owner(name, config_version);

        // The toolchain that will actually run, for the tools whose version is decided
        // outside vx's store. Resolved once and reused: both the "installed" column and
        // the pin comparison below need it, and each call spawns `rustc --version`.
        //
        // A numeric pin used to be validated only when something *named* a toolchain (a
        // `rust-toolchain.toml`, an exported `RUSTUP_TOOLCHAIN`, ...). With none of those
        // present — the case this bug is about — `owner.channel()` was `None`, every
        // branch was skipped, and a `rust = "1.93.1"` pin was silently ignored while the
        // build ran on stable. Falling back to the resolved toolchain gives the
        // comparison a real value in every case.
        //
        // The fallback is gated on the runtime name rather than on `rust_owner`, because
        // `rust_toolchain_owner` returns `None` precisely when no override names a
        // toolchain: gating on it made this arm unreachable in the one case it exists to
        // cover.
        let effective_version = if is_rust_toolchain_runtime(name) {
            rust_owner
                .as_ref()
                .and_then(|owner| owner.channel())
                .map(str::to_string)
                .or_else(|| effective_toolchain_version(project_root))
        } else {
            None
        };

        // Determine status type
        let (status_type, installed_version) = match status {
            ToolStatus::Installed => {
                // Extract version from path if possible
                let ver = if is_rust_toolchain_runtime(name) {
                    // A Rust "store version" is a rustup bootstrap directory named after
                    // the requested version, not a toolchain: `store/rust/1.0.0/` can hold
                    // a rustup whose `rustc` reports 1.98.0. Trusting the directory name
                    // — or the pin it came from — reports the pin back as the installed
                    // version, which is exactly what made a drifting pin invisible.
                    effective_version.clone()
                } else {
                    None
                }
                .or_else(|| detected_version.clone())
                .or_else(|| path.as_ref().and_then(|p| extract_version_from_path(p)))
                .unwrap_or_else(|| config_version.clone());
                (RequirementStatusType::Installed, Some(ver))
            }
            ToolStatus::SystemFallback => {
                if let Some(owner) = &rust_owner {
                    // rustup owns this toolchain, so "absent from the vx store" is the
                    // expected state — but rustup itself still has to be reachable, or
                    // the tool resolves to nothing at run time.
                    if find_rustup_executable(name).is_none() {
                        tool_errors.push(format!(
                            "{} is owned by rustup ({}) but no '{}' was found on PATH. \
                             Install rustup from https://rustup.rs/.",
                            name,
                            owner.reason(),
                            executable_for(name)
                        ));
                        tool_ok = false;
                    } else {
                        tool_warnings.push(format!(
                            "{} is owned by rustup ({}), not vx",
                            name,
                            owner.reason()
                        ));
                    }
                    // Report the version that will actually run, not just the channel an
                    // override names: `rust = "rustup-managed"` carries no channel, and
                    // an empty "installed" column is what made a drifting pin invisible.
                    (
                        RequirementStatusType::SystemFallback,
                        owner
                            .channel()
                            .map(str::to_string)
                            .or_else(|| effective_version.clone()),
                    )
                } else {
                    tool_warnings.push("Using system fallback version".to_string());
                    // Still report the detected version if available
                    (
                        RequirementStatusType::SystemFallback,
                        detected_version.clone(),
                    )
                }
            }
            ToolStatus::NotInstalled => {
                tool_errors.push(format!("{} is not installed", name));
                missing_tools.push(name.clone());
                tool_ok = false;
                (RequirementStatusType::NotInstalled, None)
            }
        };

        if let Some(actual) = &effective_version
            && versions_conflict(config_version, actual)
        {
            let reason = rust_owner
                .as_ref()
                .map(|owner| owner.reason())
                .unwrap_or_else(|| "the toolchain resolved on this machine".to_string());

            let message = format!(
                "{} is pinned to {} in vx.toml but {} runs {} — the vx.toml pin is ignored. \
                 Remove it, or set rust = \"{}\" to record that rustup owns the toolchain.",
                name, config_version, reason, actual, RUSTUP_MANAGED,
            );

            // Warn by default. Failing here would turn every already-drifting repository
            // red on upgrade, which is a breaking change to a published CLI contract;
            // repositories that want the pin enforced opt in with
            // `[check] toolchain_pin_mismatch = "error"`, or silence it entirely with
            // "ignore" when the drift is known and accepted.
            if pin_mismatch_severity.is_error() {
                tool_errors.push(message);
                tool_ok = false;
            } else if !pin_mismatch_severity.is_ignored() {
                tool_warnings.push(message);
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

        // Determine action
        let action = if !tool_ok {
            Some(format!("vx install {}@{}", name, config_version))
        } else {
            None
        };

        requirements.push(RequirementStatus {
            runtime: name.clone(),
            required: config_version.clone(),
            installed: installed_version.clone(),
            satisfied: tool_ok,
            status: status_type,
            action,
            path: path.as_ref().map(|p| p.display().to_string()),
        });

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

    if let Ok(conflicts) = conflict_detector.detect_conflicts(&tools_with_requests)
        && !conflicts.is_empty()
    {
        all_ok = false;
        for conflict in &conflicts {
            errors.push(format!("Version conflict for {}", conflict.runtime));
        }
    }

    // Build output
    let output = CheckOutput {
        project_file: Some(config_path.display().to_string()),
        requirements,
        all_satisfied: all_ok,
        missing_tools,
        warnings,
        errors,
    };

    // Render output
    if !quiet {
        let renderer = OutputRenderer::new(format);
        if renderer.is_text() {
            // Text mode with optional details
            render_text_output(&output, detailed)?;
        } else {
            renderer.render(&output)?;
        }
    }

    // Exit with appropriate code
    if !all_ok {
        std::process::exit(1);
    }

    Ok(())
}

/// Render text output with optional details
fn render_text_output(output: &CheckOutput, detailed: bool) -> Result<()> {
    if let Some(ref path) = output.project_file {
        println!("Checking project: {}", path);
        println!();
    }

    for req in &output.requirements {
        let status_icon = match req.status {
            RequirementStatusType::Installed => "✓",
            RequirementStatusType::SystemFallback => "⚠",
            RequirementStatusType::NotInstalled => "✗",
            RequirementStatusType::VersionMismatch => "✗",
        };

        let version_info = if let Some(ref ver) = req.installed {
            format!(" ({})", ver)
        } else {
            String::new()
        };

        let path_info = if detailed {
            if let Some(ref path) = req.path {
                format!(" at {}", path)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        println!(
            "{} {} {}{}{}",
            status_icon, req.runtime, req.required, version_info, path_info
        );
    }

    println!();

    if output.all_satisfied && output.warnings.is_empty() {
        UI::success("✓ All version constraints satisfied");
    } else if output.all_satisfied {
        UI::warn(&format!(
            "⚠ All constraints satisfied with {} warning(s)",
            output.warnings.len()
        ));
        if detailed {
            for warn in &output.warnings {
                println!("  - {}", warn);
            }
        } else {
            UI::hint("Run 'vx check --detailed' for more information");
        }
    } else {
        UI::error(&format!(
            "✗ {} error(s) and {} warning(s) found",
            output.errors.len(),
            output.warnings.len()
        ));
        for err in &output.errors {
            println!("  - {}", err);
        }
        if !detailed {
            UI::hint("Run 'vx check --detailed' for more information");
        }
    }

    Ok(())
}

/// Extract version from executable path
fn extract_version_from_path(path: &std::path::Path) -> Option<String> {
    for ancestor in path.ancestors() {
        if let Some(name) = ancestor.file_name().and_then(|n| n.to_str())
            && name.chars().any(|c| c.is_ascii_digit())
            && (name.contains('.') || name.chars().all(|c| c.is_ascii_digit()))
            && !name.contains('-')
        {
            return Some(name.to_string());
        }
    }
    None
}

/// Get version range configuration from provider
fn get_provider_version_range_config(
    _registry: &ProviderRegistry,
    _tool_name: &str,
) -> VersionRangeConfig {
    VersionRangeConfig::default()
}
