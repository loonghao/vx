//! Lock command implementation
//!
//! This module provides the `vx lock` command for generating and managing
//! the `vx.lock` file for reproducible environments.

use anyhow::{Context, Result};
use std::collections::{BTreeMap, HashSet};
use vx_config::{parse_config, ToolVersion, VxConfig};
use vx_paths::project::{find_vx_config, LOCK_FILE_NAME};
use vx_resolver::{
    Ecosystem, LockFile, LockedTool, ResolvedVersion, Version, VersionRequest, VersionSolver,
};
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
    let mut resolved_tools: HashSet<String> = HashSet::new();
    let mut failed_tools: Vec<(String, String)> = Vec::new();

    // Resolve all tools and their dependencies recursively
    for (tool_name, version_str) in &tools_to_resolve {
        let success = resolve_tool_with_dependencies(
            registry,
            ctx,
            &solver,
            tool_name,
            version_str,
            &mut new_lock,
            &mut resolved_tools,
            &existing_lock,
            update,
            verbose,
        )
        .await;

        if !success {
            failed_tools.push((tool_name.clone(), version_str.clone()));
        }
    }

    // Add dependency relationships to lock file
    add_dependencies(&mut new_lock, registry);

    if dry_run {
        println!("\n--- vx.lock (dry run) ---\n");
        println!("{}", new_lock.to_string()?);
        if !failed_tools.is_empty() {
            return Err(anyhow::anyhow!(
                "Failed to resolve {} tool(s): {}",
                failed_tools.len(),
                failed_tools
                    .iter()
                    .map(|(name, _)| name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        return Ok(());
    }

    // If any tools failed to resolve, don't save the lock file
    if !failed_tools.is_empty() {
        println!("\n✗ Failed to resolve {} tool(s):", failed_tools.len());
        for (name, version) in &failed_tools {
            println!("  - {}@{}", name, version);
        }
        println!("\n💡 Fix the tool configuration in vx.toml before generating the lock file");
        return Err(anyhow::anyhow!(
            "Cannot generate lock file: {} tool(s) failed to resolve",
            failed_tools.len()
        ));
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

/// Recursively resolve a tool and all its dependencies
///
/// This function:
/// 1. Resolves the tool's version
/// 2. Gets its dependencies from the runtime
/// 3. Recursively resolves each dependency
/// 4. Locks all resolved tools
///
/// Returns `true` if the tool was resolved successfully, `false` otherwise.
#[allow(clippy::too_many_arguments)]
async fn resolve_tool_with_dependencies(
    registry: &ProviderRegistry,
    ctx: &RuntimeContext,
    solver: &VersionSolver,
    tool_name: &str,
    version_str: &str,
    lock: &mut LockFile,
    resolved: &mut HashSet<String>,
    existing_lock: &Option<LockFile>,
    update: bool,
    verbose: bool,
) -> bool {
    // Avoid circular dependencies
    if resolved.contains(tool_name) {
        return true; // Already resolved
    }
    resolved.insert(tool_name.to_string());

    if verbose {
        println!("  Resolving {} @ {}...", tool_name, version_str);
    }

    // Resolve the tool's version
    match resolve_tool_version(registry, ctx, solver, tool_name, version_str, verbose).await {
        Ok(locked) => {
            if verbose {
                println!("    → {} (from {})", locked.version, locked.resolved_from);
            }
            lock.lock_tool(tool_name.to_string(), locked);

            // Get and resolve dependencies
            if let Some(provider) = registry.get_provider(tool_name) {
                if let Some(runtime) = provider.get_runtime(tool_name) {
                    let deps = runtime.dependencies();
                    if verbose && !deps.is_empty() {
                        println!("    Found {} dependencies for {}", deps.len(), tool_name);
                    }
                    for dep in deps {
                        if !resolved.contains(&dep.name) {
                            // Use the dependency's version constraint, or "latest" if not specified
                            let dep_version = dep
                                .min_version
                                .as_ref()
                                .map(|v| format!(">={}", v))
                                .unwrap_or_else(|| "latest".to_string());

                            if verbose {
                                println!(
                                    "    └─ Dependency: {} (required by {})",
                                    dep.name, tool_name
                                );
                            }

                            // Recursively resolve the dependency
                            // Note: Dependency resolution failures don't fail the parent tool
                            Box::pin(resolve_tool_with_dependencies(
                                registry,
                                ctx,
                                solver,
                                &dep.name,
                                &dep_version,
                                lock,
                                resolved,
                                existing_lock,
                                update,
                                verbose,
                            ))
                            .await;
                        }
                    }
                }
            }
            true
        }
        Err(e) => {
            eprintln!("  ✗ Failed to resolve {}: {}", tool_name, e);
            if !update {
                // Keep existing lock entry if not updating
                if let Some(ref existing) = existing_lock {
                    if let Some(existing_tool) = existing.get_tool(tool_name) {
                        lock.lock_tool(tool_name.to_string(), existing_tool.clone());
                        return true; // Kept existing, so not a failure
                    }
                }
            }
            false
        }
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
    verbose: bool,
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

    // Get ecosystem from runtime and convert to vx_resolver::Ecosystem
    let runtime_ecosystem = runtime.ecosystem();
    let ecosystem = match runtime_ecosystem {
        vx_runtime::Ecosystem::NodeJs => Ecosystem::Node,
        vx_runtime::Ecosystem::Python => Ecosystem::Python,
        vx_runtime::Ecosystem::Rust => Ecosystem::Rust,
        vx_runtime::Ecosystem::Go => Ecosystem::Go,
        _ => Ecosystem::Generic,
    };

    // Check if this tool supports passthrough versions (e.g., Rust with rustup)
    // Passthrough means the version manager (like rustup) handles version validation,
    // so we accept any user-specified version without needing it in available versions.
    let is_passthrough = versions.iter().any(|v| {
        v.metadata
            .get("passthrough")
            .map(|s| s == "true")
            .unwrap_or(false)
    });

    // Parse version request
    let request = VersionRequest::parse(version_str);

    // For passthrough tools, check if the requested version matches a known channel
    // or use it directly as a version number
    let resolved = if is_passthrough {
        // Check if version matches a channel (stable, beta, nightly)
        if let Some(channel_version) = versions.iter().find(|v| v.version == version_str) {
            // Use the channel version directly
            ResolvedVersion {
                version: Version::parse(&channel_version.version)
                    .unwrap_or_else(|| Version::new(0, 0, 0)),
                source: ecosystem.to_string(),
                metadata: channel_version.metadata.clone(),
                resolved_from: version_str.to_string(),
            }
        } else {
            // Use user-specified version directly (e.g., "1.83.0")
            // This allows exact version numbers like from rust-version in Cargo.toml
            if verbose {
                println!("    ℹ Using passthrough version: {}", version_str);
            }
            ResolvedVersion {
                version: Version::parse(version_str).unwrap_or_else(|| Version::new(0, 0, 0)),
                source: ecosystem.to_string(),
                metadata: std::collections::HashMap::new(),
                resolved_from: version_str.to_string(),
            }
        }
    } else {
        // Normal resolution through version solver
        solver
            .resolve(tool_name, &request, &versions, &ecosystem)
            .map_err(|e| anyhow::anyhow!("{}", e))?
    };

    // Get download URL for the current platform
    let current_platform = vx_runtime::Platform::current();
    let download_url = if let Ok(Some(url)) = runtime
        .download_url(&resolved.version.to_string(), &current_platform)
        .await
    {
        Some(url)
    } else {
        if verbose {
            eprintln!("    ⚠ Warning: No download URL available for {}", tool_name);
        }
        None
    };

    // Create locked tool entry
    let mut locked = LockedTool::new(resolved.version.to_string(), resolved.source.clone())
        .with_resolved_from(version_str)
        .with_ecosystem(ecosystem);

    // Add download URL if available
    if let Some(url) = download_url {
        locked = locked.with_download_url(url);
    }

    // Copy metadata
    for (key, value) in &resolved.metadata {
        locked = locked.with_metadata(key.clone(), value.clone());
    }

    Ok(locked)
}

/// Add dependency relationships to lock file
///
/// This function dynamically retrieves dependencies from the ProviderRegistry
/// instead of using hardcoded mappings. This ensures that:
/// 1. New tools with dependencies are automatically handled
/// 2. Dependencies declared in provider manifests are respected
/// 3. The lock file accurately reflects the dependency graph
fn add_dependencies(lock: &mut LockFile, registry: &ProviderRegistry) {
    // Collect all locked tool names first to avoid borrow issues
    let locked_tools: Vec<String> = lock.tool_names().iter().map(|s| s.to_string()).collect();

    for tool_name in &locked_tools {
        // Get the runtime from registry to access its dependencies
        if let Some(provider) = registry.get_provider(tool_name) {
            if let Some(runtime) = provider.get_runtime(tool_name) {
                // Get dependencies from the runtime
                let deps: Vec<String> = runtime
                    .dependencies()
                    .iter()
                    .map(|d| d.name.clone())
                    .filter(|dep_name| {
                        // Only include dependencies that are also locked
                        // or that we need to auto-lock
                        lock.is_locked(dep_name)
                    })
                    .collect();

                if !deps.is_empty() {
                    lock.add_dependency(tool_name.clone(), deps);
                }
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

    // Build config tools map (use BTreeMap for deterministic ordering)
    let config_tools: BTreeMap<String, String> = config
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
