//! Lock command implementation
//!
//! This module provides the `vx lock` command for generating and managing
//! the `vx.lock` file for reproducible environments.

use anyhow::{Context, Result};
use vx_config::{parse_config, ToolVersion, VxConfig};
use vx_paths::project::{find_vx_config, LOCK_FILE_NAME};
use vx_resolver::{Ecosystem, LockFile, LockedTool, VersionRequest, VersionSolver};
use vx_runtime::{ProviderRegistry, RuntimeContext};

/// Handle the lock command
pub async fn handle(
    registry: &ProviderRegistry,
    ctx: &RuntimeContext,
    update: bool,
    update_tool: Option<&str>,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let config_path =
        find_vx_config(&current_dir).map_err(|e| anyhow::anyhow!("No vx.toml found: {}", e))?;

    let project_root = config_path.parent().unwrap_or(&current_dir);
    let lock_path = project_root.join(LOCK_FILE_NAME);

    // Load vx.toml
    let config = parse_config(&config_path)
        .with_context(|| format!("Failed to load {}", config_path.display()))?;

    // Load existing lock file if present
    let existing_lock = if lock_path.exists() {
        Some(LockFile::load(&lock_path).with_context(|| {
            format!("Failed to load existing lock file: {}", lock_path.display())
        })?)
    } else {
        None
    };

    // Determine which tools to resolve
    let tools_to_resolve = get_tools_to_resolve(&config, &existing_lock, update, update_tool);

    if tools_to_resolve.is_empty() {
        if existing_lock.is_some() {
            println!("✓ Lock file is up to date");
            return Ok(());
        } else {
            println!("No tools configured in vx.toml");
            return Ok(());
        }
    }

    if verbose {
        println!("Resolving versions for {} tools...", tools_to_resolve.len());
    }

    // Create solver and resolve versions
    let solver = VersionSolver::new();
    let mut new_lock = existing_lock.clone().unwrap_or_default();

    for (tool_name, version_str) in &tools_to_resolve {
        if verbose {
            println!("  Resolving {} @ {}...", tool_name, version_str);
        }

        match resolve_tool_version(registry, ctx, &solver, tool_name, version_str).await {
            Ok(locked) => {
                if verbose {
                    println!("    → {} (from {})", locked.version, locked.resolved_from);
                }
                new_lock.lock_tool(tool_name.clone(), locked);
            }
            Err(e) => {
                eprintln!("  ✗ Failed to resolve {}: {}", tool_name, e);
                if !update {
                    // Keep existing lock entry if not updating
                    if let Some(ref existing) = existing_lock {
                        if let Some(existing_tool) = existing.get_tool(tool_name) {
                            new_lock.lock_tool(tool_name.clone(), existing_tool.clone());
                        }
                    }
                }
            }
        }
    }

    // Add dependency information
    add_dependencies(&mut new_lock);

    if dry_run {
        println!("\n--- vx.lock (dry run) ---\n");
        println!("{}", new_lock.to_string()?);
        return Ok(());
    }

    // Save lock file
    new_lock.save(&lock_path)?;

    let action = if existing_lock.is_some() {
        "Updated"
    } else {
        "Created"
    };
    println!(
        "✓ {} {} with {} tools",
        action,
        LOCK_FILE_NAME,
        new_lock.tools.len()
    );

    if verbose {
        for (name, tool) in &new_lock.tools {
            println!(
                "  {} = {} (from {})",
                name, tool.version, tool.resolved_from
            );
        }
    }

    Ok(())
}

/// Get version string from ToolVersion
fn get_version_string(version: &ToolVersion) -> String {
    match version {
        ToolVersion::Simple(s) => s.clone(),
        ToolVersion::Detailed(d) => d.version.clone(),
    }
}

/// Get tools that need to be resolved
fn get_tools_to_resolve(
    config: &VxConfig,
    existing_lock: &Option<LockFile>,
    update: bool,
    update_tool: Option<&str>,
) -> Vec<(String, String)> {
    let mut tools = Vec::new();

    for (name, version) in &config.tools {
        let version_str = get_version_string(version);

        // If updating specific tool, only include that tool
        if let Some(target) = update_tool {
            if name == target {
                tools.push((name.clone(), version_str));
            }
            continue;
        }

        // If updating all, include all tools
        if update {
            tools.push((name.clone(), version_str));
            continue;
        }

        // Otherwise, only include tools not in lock file or with changed version
        match existing_lock {
            Some(lock) => {
                if let Some(locked) = lock.get_tool(name) {
                    if locked.resolved_from != version_str {
                        tools.push((name.clone(), version_str));
                    }
                } else {
                    tools.push((name.clone(), version_str));
                }
            }
            None => {
                tools.push((name.clone(), version_str));
            }
        }
    }

    tools
}

/// Resolve a single tool's version
async fn resolve_tool_version(
    registry: &ProviderRegistry,
    ctx: &RuntimeContext,
    solver: &VersionSolver,
    tool_name: &str,
    version_str: &str,
) -> Result<LockedTool> {
    // Find provider for this tool
    let provider = registry
        .get_provider(tool_name)
        .ok_or_else(|| anyhow::anyhow!("Unknown tool: {}", tool_name))?;

    // Get runtime for this tool
    let runtime = provider
        .get_runtime(tool_name)
        .ok_or_else(|| anyhow::anyhow!("No runtime found for: {}", tool_name))?;

    // Fetch available versions
    let versions = runtime.fetch_versions(ctx).await?;

    if versions.is_empty() {
        return Err(anyhow::anyhow!("No versions available for {}", tool_name));
    }

    // Parse version request
    let request = VersionRequest::parse(version_str);

    // Get ecosystem from runtime and convert to vx_resolver::Ecosystem
    let runtime_ecosystem = runtime.ecosystem();
    let ecosystem = match runtime_ecosystem {
        vx_runtime::Ecosystem::NodeJs => Ecosystem::Node,
        vx_runtime::Ecosystem::Python => Ecosystem::Python,
        vx_runtime::Ecosystem::Rust => Ecosystem::Rust,
        vx_runtime::Ecosystem::Go => Ecosystem::Go,
        _ => Ecosystem::Generic,
    };

    // Resolve version
    let resolved = solver
        .resolve(tool_name, &request, &versions, &ecosystem)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Create locked tool entry
    let mut locked = LockedTool::new(resolved.version.to_string(), resolved.source.clone())
        .with_resolved_from(version_str)
        .with_ecosystem(ecosystem);

    // Copy metadata
    for (key, value) in &resolved.metadata {
        locked = locked.with_metadata(key.clone(), value.clone());
    }

    Ok(locked)
}

/// Add dependency relationships to lock file
fn add_dependencies(lock: &mut LockFile) {
    // Common dependency mappings
    let dependencies = [
        ("npm", vec!["node"]),
        ("npx", vec!["node"]),
        ("yarn", vec!["node"]),
        ("pnpm", vec!["node"]),
        ("bun", vec![]),
        ("uvx", vec!["uv"]),
        ("pip", vec!["python"]),
        ("cargo", vec!["rust"]),
        ("rustc", vec!["rust"]),
    ];

    for (tool, deps) in dependencies {
        if lock.is_locked(tool) {
            let deps: Vec<String> = deps
                .iter()
                .filter(|d| lock.is_locked(d))
                .map(|s| s.to_string())
                .collect();
            if !deps.is_empty() {
                lock.add_dependency(tool, deps);
            }
        }
    }
}

/// Handle the check command - verify lock file consistency
pub async fn handle_check(verbose: bool) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let config_path =
        find_vx_config(&current_dir).map_err(|e| anyhow::anyhow!("No vx.toml found: {}", e))?;

    let project_root = config_path.parent().unwrap_or(&current_dir);
    let lock_path = project_root.join(LOCK_FILE_NAME);

    // Load vx.toml
    let config = parse_config(&config_path)
        .with_context(|| format!("Failed to load {}", config_path.display()))?;

    // Check if lock file exists
    if !lock_path.exists() {
        println!("✗ No {} found", LOCK_FILE_NAME);
        println!("  Run 'vx lock' to generate one");
        return Ok(());
    }

    // Load lock file
    let lock = LockFile::load(&lock_path)
        .with_context(|| format!("Failed to load {}", lock_path.display()))?;

    // Build config tools map
    let config_tools: std::collections::HashMap<String, String> = config
        .tools
        .iter()
        .map(|(k, v)| (k.clone(), get_version_string(v)))
        .collect();

    // Check consistency
    let inconsistencies = lock.check_consistency(&config_tools);

    if inconsistencies.is_empty() {
        println!("✓ {} is consistent with vx.toml", LOCK_FILE_NAME);
        if verbose {
            println!("\nLocked tools:");
            for (name, tool) in &lock.tools {
                println!(
                    "  {} = {} (from {})",
                    name, tool.version, tool.resolved_from
                );
            }
        }
        return Ok(());
    }

    println!("✗ {} is inconsistent with vx.toml:", LOCK_FILE_NAME);
    for issue in &inconsistencies {
        println!("  • {}", issue);
    }
    println!("\nRun 'vx lock' to update the lock file");

    // Return error to indicate inconsistency (useful for CI)
    Err(anyhow::anyhow!(
        "Lock file has {} inconsistencies",
        inconsistencies.len()
    ))
}
