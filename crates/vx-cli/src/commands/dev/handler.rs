//! Dev command handler

use super::Args;
use super::export::handle_export;
use super::info::handle_info;
use super::shell::spawn_dev_shell;
use super::tools::get_registry;
use crate::commands::common::load_config_view_cwd;
use crate::commands::setup::ConfigView;
use crate::ui::UI;
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::process::Command;
use vx_env::{RuntimeSpec, ToolEnvironment};
use vx_starlark::handle::global_registry;
use vx_starlark::provider::{EnvOp, apply_env_ops};

/// Handle dev command with Args
pub async fn handle(args: &Args) -> Result<()> {
    // Use common configuration loading
    let (config_path, mut config) = load_config_view_cwd()?;

    // Override isolation mode if --inherit-system is specified
    if args.inherit_system {
        config.isolation = false;
    }

    // Merge CLI passenv with config passenv
    if !args.passenv.is_empty() {
        config.passenv.extend(args.passenv.clone());
    }

    if config.tools.is_empty() {
        UI::warn("No tools configured in vx.toml");
        UI::hint("Run 'vx init' to initialize the project configuration");
        return Ok(());
    }

    // Handle --export mode
    if args.export {
        return handle_export(&config, args.format.clone()).await;
    }

    // Handle --info mode
    if args.info {
        return handle_info(&config).await;
    }

    // Check and install missing tools if needed
    if !args.no_install {
        let auto_install = config
            .settings
            .get("auto_install")
            .map(|v| v == "true")
            .unwrap_or(true);

        if auto_install {
            // Reuse sync's tool installation logic to avoid duplication
            let (registry, _) = get_registry()?;
            crate::commands::sync::handle(
                &registry,
                false, // check: false - we want to install
                false, // force: false
                false, // dry_run: false
                args.verbose,
                false, // no_parallel: false - dev prefers parallel
            )
            .await?;
        }
    }

    // Build the environment
    let env_vars = build_dev_environment(&config, args.verbose).await?;

    // Execute command or spawn shell
    if let Some(cmd) = &args.command {
        execute_command_in_env(cmd, &env_vars)?;
    } else {
        spawn_dev_shell(
            args.shell.clone(),
            &env_vars,
            &config,
            Some(config_path.clone()),
        )?;
    }

    Ok(())
}

/// Build environment variables for the dev shell.
///
/// Implements rez-style dependency resolution:
///
/// **Phase 1 — BFS dependency tree expansion**
///   Start with tools declared in `vx.toml`, then BFS-expand their `deps()`
///   to build a complete dependency tree. Each node is visited at most once
///   (cycle/duplicate detection via `visited` set). The result is
///   `resolved_order`: a list of `(tool_name, version)` pairs with deps
///   appearing *before* their dependents (topological order).
///
/// **Phase 2 — Collect EnvOps in resolution order**
///   For each tool in `resolved_order`, call `environment_ops(version)` to
///   get the list of [`EnvOp`]s that describe how to set up the environment
///   (PATH prepends, variable sets, etc.). Deps' ops come first so their
///   PATH entries take lower precedence than direct tools.
///
/// **Phase 3 — Apply non-PATH EnvOps**
///   Apply all collected EnvOps. PATH-like vars are handled by `vx-env`'s
///   `ToolEnvironment` (Phase 4), so we only apply non-PATH vars here
///   (e.g. GOROOT, JAVA_HOME, GIT_EXEC_PATH).
///
/// **Phase 4 — Build final environment via vx-env**
///   `ToolEnvironment` handles PATH construction with proper isolation,
///   passenv filtering, and bin-dir resolution for vx-managed tools.
async fn build_dev_environment(
    config: &ConfigView,
    verbose: bool,
) -> Result<HashMap<String, String>> {
    // Merge env from vx.toml with setenv from settings
    let mut env_vars = config.env.clone();
    env_vars.extend(config.setenv.clone());

    // ── Phase 1: BFS dependency tree resolution (rez-style) ─────────────────
    //
    // Ordered list of (tool_name, resolved_version) — deps appear before
    // their dependents (topological order).
    let mut resolved_order: Vec<(String, String)> = Vec::new();
    // Set of tool names already visited (cycle / duplicate detection)
    let mut visited: HashSet<String> = HashSet::new();
    // BFS queue: (tool_name, version_req, is_direct)
    let mut queue: VecDeque<(String, String, bool)> = VecDeque::new();

    // Seed the queue with tools declared in vx.toml
    for (tool_name, version) in &config.tools {
        queue.push_back((tool_name.clone(), version.clone(), true));
    }

    let reg = global_registry().await;

    while let Some((tool_name, version, is_direct)) = queue.pop_front() {
        if visited.contains(&tool_name) {
            continue;
        }
        visited.insert(tool_name.clone());

        // Look up the ProviderHandle for this tool
        if let Some(handle) = reg.get(&tool_name) {
            // Resolve the version (e.g. "latest" → actual installed version)
            let resolved_version = handle
                .resolve_installed_version(&version)
                .unwrap_or_else(|_| version.clone());

            // Fetch deps from provider.star::deps()
            match handle.deps(&resolved_version).await {
                Ok(deps) => {
                    for dep in &deps {
                        if dep.optional {
                            // Optional deps: only include if already installed
                            if let Some(dep_handle) = reg.get(&dep.runtime) {
                                if dep_handle.is_installed(&dep.version_req)
                                    || !dep_handle.installed_versions().is_empty()
                                {
                                    if !visited.contains(&dep.runtime) {
                                        if verbose {
                                            UI::info(&format!(
                                                "  [deps] {} → {} (optional, installed)",
                                                tool_name, dep.runtime
                                            ));
                                        }
                                        queue.push_back((
                                            dep.runtime.clone(),
                                            dep.version_req.clone(),
                                            false,
                                        ));
                                    }
                                } else if verbose {
                                    UI::info(&format!(
                                        "  [deps] {} → {} (optional, skipped — not installed)",
                                        tool_name, dep.runtime
                                    ));
                                }
                            }
                        } else {
                            // Required deps: always include
                            if !visited.contains(&dep.runtime) {
                                if verbose {
                                    UI::info(&format!(
                                        "  [deps] {} → {} (required)",
                                        tool_name, dep.runtime
                                    ));
                                }
                                queue.push_back((
                                    dep.runtime.clone(),
                                    dep.version_req.clone(),
                                    false,
                                ));
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::debug!(
                        tool = %tool_name,
                        error = %e,
                        "Failed to fetch deps, skipping"
                    );
                }
            }

            resolved_order.push((tool_name.clone(), resolved_version));
        } else if is_direct {
            // Direct tool not found in registry — still add to order so
            // vx-env can warn about it
            resolved_order.push((tool_name.clone(), version.clone()));
        } else {
            // Transitive dep not found — check system PATH as fallback
            if let Ok(path) = which::which(&tool_name) {
                if verbose {
                    UI::info(&format!(
                        "  [deps] {} found on system PATH: {}",
                        tool_name,
                        path.display()
                    ));
                }
                // Add the system bin dir to PATH via EnvOp
                if let Some(bin_dir) = path.parent() {
                    let bin_dir_str = bin_dir.to_string_lossy().to_string();
                    env_vars
                        .entry("_VX_SYSTEM_PATHS".to_string())
                        .and_modify(|v| {
                            let sep = if cfg!(windows) { ";" } else { ":" };
                            v.push_str(sep);
                            v.push_str(&bin_dir_str);
                        })
                        .or_insert(bin_dir_str.clone());
                }
            } else if verbose {
                UI::warn(&format!(
                    "  [deps] required dependency '{}' not found in vx registry or system PATH",
                    tool_name
                ));
            }
        }
    }

    if verbose {
        let names: Vec<String> = resolved_order
            .iter()
            .map(|(n, v)| format!("{}@{}", n, v))
            .collect();
        UI::info(&format!(
            "  [env] resolved order (including deps): {}",
            names.join(", ")
        ));
    }

    // ── Phase 2: Collect EnvOps in resolution order ──────────────────────────
    //
    // Deps come first, so their PATH entries appear after direct tools in PATH
    // (lower precedence). This matches rez semantics.
    let mut all_env_ops: Vec<EnvOp> = Vec::new();

    for (tool_name, version) in &resolved_order {
        if let Some(handle) = reg.get(tool_name) {
            match handle.environment_ops(version).await {
                Ok(ops) => {
                    if verbose && !ops.is_empty() {
                        UI::info(&format!(
                            "  [env] {}@{}: {} op(s)",
                            tool_name,
                            version,
                            ops.len()
                        ));
                    }
                    all_env_ops.extend(ops);
                }
                Err(e) => {
                    tracing::debug!(
                        tool = %tool_name,
                        version = %version,
                        error = %e,
                        "Failed to fetch environment ops, skipping"
                    );
                }
            }
        }
    }

    // ── Phase 3: Apply non-PATH EnvOps ───────────────────────────────────────
    //
    // EnvOps from provider.star::environment() are applied here.
    // PATH-like vars are handled by vx-env's ToolEnvironment (Phase 4),
    // so we only apply non-PATH vars (e.g. GOROOT, JAVA_HOME, GIT_EXEC_PATH).
    let starlark_env = apply_env_ops(&all_env_ops, None);
    for (key, value) in starlark_env {
        if key != "PATH" {
            env_vars.insert(key, value);
        }
    }

    // ── Phase 4: Build final environment via vx-env ──────────────────────────
    //
    // vx-env's ToolEnvironment handles PATH construction with proper isolation,
    // passenv filtering, and bin-dir resolution for vx-managed tools.
    // We also inject system-PATH deps discovered in Phase 1.
    let (registry, context) = get_registry()?;

    // Build RuntimeSpecs for all resolved tools (direct + transitive deps)
    let mut tool_specs = Vec::new();
    for (tool_name, version) in &resolved_order {
        // Find the runtime for this tool to get bin directories
        let (bin_dirs, resolved_bin_dir) =
            if let Some(provider) = registry.providers().iter().find(|p| p.supports(tool_name)) {
                if let Some(runtime) = provider.get_runtime(tool_name) {
                    if let Ok(Some(exe_path)) = runtime
                        .get_executable_path_for_version(version, &context)
                        .await
                    {
                        let dirs = runtime
                            .possible_bin_dirs()
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect();
                        (dirs, exe_path.parent().map(|p| p.to_path_buf()))
                    } else {
                        (vec!["bin".to_string()], None)
                    }
                } else {
                    (vec!["bin".to_string()], None)
                }
            } else {
                (vec!["bin".to_string()], None)
            };

        let mut spec = RuntimeSpec::with_bin_dirs(tool_name.clone(), version.clone(), bin_dirs);
        if let Some(bin_dir) = resolved_bin_dir {
            spec = spec.set_resolved_bin_dir(bin_dir);
        }
        tool_specs.push(spec);
    }

    // Inject system-PATH deps (discovered in Phase 1) into env_vars PATH
    // so vx-env can include them when building the final PATH.
    let system_paths = env_vars.remove("_VX_SYSTEM_PATHS");

    let mut builder = ToolEnvironment::new()
        .tools_from_specs(tool_specs)
        .env_vars(&env_vars)
        .warn_missing(verbose)
        .isolation(config.isolation);

    // Add passenv patterns if in isolation mode
    if config.isolation && !config.passenv.is_empty() {
        builder = builder.passenv(config.passenv.clone());
    }

    let mut env_result = builder.build()?;

    // Append system-PATH deps to the final PATH
    if let Some(sys_paths) = system_paths {
        let sep = if cfg!(windows) { ";" } else { ":" };
        let current_path = env_result.get("PATH").cloned().unwrap_or_default();
        let new_path = if current_path.is_empty() {
            sys_paths
        } else {
            format!("{}{}{}", current_path, sep, sys_paths)
        };
        env_result.insert("PATH".to_string(), new_path);
    }

    // Set VX_DEV environment variable to indicate we're in a dev shell
    env_result.insert("VX_DEV".to_string(), "1".to_string());

    // Set VX_PROJECT_NAME for prompt customization
    env_result.insert("VX_PROJECT_NAME".to_string(), config.project_name.clone());

    // Set VX_PROJECT_ROOT
    if let Ok(current_dir) = env::current_dir() {
        env_result.insert(
            "VX_PROJECT_ROOT".to_string(),
            current_dir.to_string_lossy().to_string(),
        );
    }

    // Log tool paths if verbose
    if verbose {
        if config.isolation {
            UI::info("Running in isolation mode");
            if !config.passenv.is_empty() {
                UI::info(&format!("  passenv: {}", config.passenv.join(", ")));
            }
        }
        if let Some(path) = env_result.get("PATH") {
            let sep = if cfg!(windows) { ";" } else { ":" };
            for entry in path.split(sep).take(resolved_order.len() + 5) {
                UI::info(&format!("  PATH: {}", entry));
            }
        }
    }

    Ok(env_result)
}

/// Execute a command inside the dev environment
fn execute_command_in_env(cmd: &[String], env_vars: &HashMap<String, String>) -> Result<()> {
    if cmd.is_empty() {
        return Err(anyhow::anyhow!("No command specified"));
    }

    let program = &cmd[0];
    let args = &cmd[1..];

    // Clear inherited environment and set only our variables
    let mut command = Command::new(program);
    command.args(args);
    command.env_clear();

    // Set all environment variables from our isolated/configured environment
    for (key, value) in env_vars {
        command.env(key, value);
    }

    let status = command
        .status()
        .with_context(|| format!("Failed to execute: {}", program))?;

    if !status.success() {
        std::process::exit(vx_resolver::exit_code_from_status(&status));
    }

    Ok(())
}

/// Build environment variables for script execution
///
/// Uses vx-env's ToolEnvironment for consistent environment building.
///
/// This function works in both async Tokio contexts and synchronous contexts
/// (e.g. unit tests). When called outside a Tokio runtime it creates a
/// temporary single-threaded runtime for the async provider lookups.
pub fn build_script_environment(config: &ConfigView) -> Result<HashMap<String, String>> {
    // Merge env from vx.toml with setenv from settings
    let mut env_vars = config.env.clone();
    env_vars.extend(config.setenv.clone());

    // Get registry to query runtime bin directories
    let (registry, context) = get_registry()?;

    // Prepare a local single-threaded Tokio runtime when there is no active
    // runtime (e.g. called from synchronous test code).  When an active
    // runtime IS present we use block_in_place / Handle::current() instead.
    let local_rt: Option<tokio::runtime::Runtime> =
        tokio::runtime::Handle::try_current().err().map(|_| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to build local Tokio runtime for build_script_environment")
        });

    // Create RuntimeSpecs with proper bin directories from runtime providers
    let mut tool_specs = Vec::new();
    for (tool_name, version) in &config.tools {
        let (bin_dirs, resolved_bin_dir) = if let Some(provider) =
            registry.providers().iter().find(|p| p.supports(tool_name))
        {
            if let Some(runtime) = provider.get_runtime(tool_name) {
                let exe_path = if let Some(ref rt) = local_rt {
                    // No active Tokio runtime — use our temporary one.
                    rt.block_on(runtime.get_executable_path_for_version(version, &context))
                } else {
                    // Inside an active Tokio runtime — block the thread.
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current()
                            .block_on(runtime.get_executable_path_for_version(version, &context))
                    })
                };
                if let Ok(Some(exe_path)) = exe_path {
                    let dirs = runtime
                        .possible_bin_dirs()
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect();
                    (dirs, exe_path.parent().map(|p| p.to_path_buf()))
                } else {
                    (vec!["bin".to_string()], None)
                }
            } else {
                (vec!["bin".to_string()], None)
            }
        } else {
            (vec!["bin".to_string()], None)
        };

        let mut spec = RuntimeSpec::with_bin_dirs(tool_name.clone(), version.clone(), bin_dirs);
        if let Some(bin_dir) = resolved_bin_dir {
            spec = spec.set_resolved_bin_dir(bin_dir);
        }
        tool_specs.push(spec);
    }

    let mut builder = ToolEnvironment::new()
        .tools_from_specs(tool_specs)
        .env_vars(&env_vars)
        .isolation(config.isolation);

    if config.isolation && !config.passenv.is_empty() {
        builder = builder.passenv(config.passenv.clone());
    }

    builder.build()
}
