//! Command shim handlers (RFC 0042)
//!
//! Turns any runtime into a first-class command. `vx shim add jq` writes a
//! platform-appropriate wrapper into a directory on PATH so `jq --version`
//! works without typing `vx` first — the same wrapper users otherwise write by
//! hand:
//!
//! ```sh
//! #!/bin/sh
//! exec vx jq "$@"
//! ```

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use vx_runtime::{CommandShim, Platform, ShimRegistry, ShimType, create_command_shim};

use super::args::{AddShimArgs, ListShimArgs, RemoveShimArgs, ShimCommand};
use crate::commands::CommandContext;
use crate::ui::UI;

/// Handle the `vx shim` command family
pub async fn handle(ctx: &CommandContext, command: &ShimCommand) -> Result<()> {
    match command {
        ShimCommand::Add(args) => handle_add(ctx, args),
        ShimCommand::List(args) => handle_list(ctx, args),
        ShimCommand::Remove(args) => handle_remove(ctx, args),
        ShimCommand::Sync => handle_sync(ctx),
        ShimCommand::Path => handle_path(ctx),
    }
}

/// Expose a runtime as a directly callable command
fn handle_add(ctx: &CommandContext, args: &AddShimArgs) -> Result<()> {
    let name = shim_name(&args.runtime, args.name.as_deref())?;
    let paths = ctx.runtime_context().paths.clone();
    let registry_path = ShimRegistry::default_path(&paths.config_dir());
    let mut registry = ShimRegistry::load_or_default(&registry_path);

    let launcher = current_launcher()?;
    let dirs = target_dirs(&paths.bin_dir(), &args.dir);

    check_collision(&name, &dirs, args.force)?;

    let entry = create_command_shim(&name, &args.runtime, &launcher, &dirs, &Platform::current())?;
    let files = entry.files.clone();
    registry.upsert(entry);
    registry
        .save(&registry_path)
        .with_context(|| format!("Failed to save shim registry: {}", registry_path.display()))?;

    UI::success(&format!("'{}' now runs 'vx {}'", name, args.runtime));
    for file in &files {
        UI::detail(&format!("created {}", file.display()));
    }

    report_path_status(&dirs);

    Ok(())
}

/// List the shims vx created
fn handle_list(ctx: &CommandContext, args: &ListShimArgs) -> Result<()> {
    let paths = ctx.runtime_context().paths.clone();
    let registry_path = ShimRegistry::default_path(&paths.config_dir());
    let registry = ShimRegistry::load_or_default(&registry_path);

    if args.json || ctx.is_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&registry.shims)
                .context("Failed to serialize shim registry")?
        );
        return Ok(());
    }

    if registry.is_empty() {
        UI::info("No command shims. Create one with 'vx shim add <runtime>'.");
        return Ok(());
    }

    UI::header(&format!("Command shims ({})", registry.len()));
    for shim in &registry.shims {
        let status = if shim.is_complete() {
            "ok"
        } else {
            "incomplete"
        };
        println!(
            "  {:<16} -> vx {:<20} [{}]",
            shim.name, shim.runtime, status
        );
        UI::detail(
            &shim
                .dirs
                .iter()
                .map(|dir| dir.display().to_string())
                .collect::<Vec<_>>()
                .join(", "),
        );
    }

    Ok(())
}

/// Remove a shim vx created
fn handle_remove(ctx: &CommandContext, args: &RemoveShimArgs) -> Result<()> {
    let paths = ctx.runtime_context().paths.clone();
    let registry_path = ShimRegistry::default_path(&paths.config_dir());
    let mut registry = ShimRegistry::load_or_default(&registry_path);

    let Some(entry) = registry.remove(&args.name) else {
        bail!(
            "'{}' is not a vx command shim. Nothing was removed.",
            args.name
        );
    };

    let mut removed = 0;
    for file in &entry.files {
        if !vx_runtime::Shim::is_managed(file) {
            UI::warn(&format!("skipping {} — not created by vx", file.display()));
            continue;
        }

        match std::fs::remove_file(file) {
            Ok(()) => removed += 1,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("Failed to remove shim file: {}", file.display()));
            }
        }
    }

    registry.save(&registry_path)?;

    if removed == 0 && !args.force {
        UI::warn(&format!("no shim files found for '{}'", args.name));
        return Ok(());
    }

    UI::success(&format!("Removed command shim '{}'", args.name));
    Ok(())
}

/// Rewrite every shim against the current vx executable
fn handle_sync(ctx: &CommandContext) -> Result<()> {
    let paths = ctx.runtime_context().paths.clone();
    let registry_path = ShimRegistry::default_path(&paths.config_dir());
    let mut registry = ShimRegistry::load_or_default(&registry_path);

    if registry.is_empty() {
        UI::info("No command shims to sync.");
        return Ok(());
    }

    let launcher = current_launcher()?;
    let platform = Platform::current();
    let entries: Vec<CommandShim> = registry.shims.clone();

    let mut rewritten = 0;
    let mut stale = 0;

    for entry in entries {
        if entry.launcher != launcher || !entry.is_complete() {
            stale += 1;
        }

        let refreshed = create_command_shim(
            &entry.name,
            &entry.runtime,
            &launcher,
            &entry.dirs,
            &platform,
        )?;
        registry.upsert(refreshed);
        rewritten += 1;
    }

    registry.save(&registry_path)?;

    UI::success(&format!(
        "Refreshed {} shim(s) against {}",
        rewritten,
        launcher.display()
    ));
    if stale > 0 {
        UI::detail(&format!("{} shim(s) pointed at an older vx", stale));
    }

    Ok(())
}

/// Show where shims are written and whether those directories are on PATH
fn handle_path(ctx: &CommandContext) -> Result<()> {
    let paths = ctx.runtime_context().paths.clone();
    let dirs = target_dirs(&paths.bin_dir(), &[]);

    UI::header("Command shim directories");
    for (dir, on_path) in path_status(&dirs) {
        let marker = if on_path { "on PATH" } else { "not on PATH" };
        println!("  {:<60} {}", dir.display(), marker);
    }

    UI::detail(&format!(
        "Platform variants: {}",
        describe_variants(&Platform::current())
    ));

    report_path_status(&dirs);

    Ok(())
}

/// Derive the command name from a runtime spec and an optional override
fn shim_name(runtime: &str, override_name: Option<&str>) -> Result<String> {
    let name = match override_name {
        Some(name) => name.to_string(),
        None => runtime
            .split_once('@')
            .map(|(base, _)| base)
            .unwrap_or(runtime)
            .to_string(),
    };

    if name.trim().is_empty() {
        bail!("Shim name cannot be empty");
    }
    if name.contains(std::path::MAIN_SEPARATOR) || name.contains('/') || name.contains('\\') {
        bail!("Shim name cannot contain path separators: '{}'", name);
    }

    Ok(name)
}

/// Absolute path of the running vx executable
fn current_launcher() -> Result<PathBuf> {
    std::env::current_exe().context("Failed to locate the running vx executable")
}

/// Directories shims are written to
///
/// With no explicit directories this mirrors the stacked layout used by global
/// packages: the vx bin directory plus the directory holding the `vx`
/// executable, which is the one directory known to already be on PATH.
fn target_dirs(bin_dir: &Path, explicit: &[PathBuf]) -> Vec<PathBuf> {
    if !explicit.is_empty() {
        return dedup(explicit.to_vec());
    }

    let mut dirs = vec![bin_dir.to_path_buf()];
    if let Ok(current) = std::env::current_exe()
        && let Some(parent) = current.parent()
    {
        dirs.push(parent.to_path_buf());
    }

    dedup(dirs)
}

/// Drop duplicates and non-absolute entries while keeping order
fn dedup(dirs: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = Vec::new();
    for dir in dirs {
        let key = normalize(&dir);
        if !seen.iter().any(|existing| existing == &key) {
            seen.push(key);
        }
    }
    seen
}

/// Lowercase, separator-normalized form used for directory comparison
fn normalize(dir: &Path) -> PathBuf {
    let text = dir.to_string_lossy().replace('\\', "/").to_lowercase();
    PathBuf::from(text.trim_end_matches('/'))
}

/// Refuse to shadow a binary vx did not create unless `--force` is given
fn check_collision(name: &str, dirs: &[PathBuf], force: bool) -> Result<()> {
    let Ok(existing) = which::which(name) else {
        return Ok(());
    };

    if vx_runtime::Shim::is_managed(&existing) {
        return Ok(());
    }

    let owned_dir = dirs.iter().any(|dir| {
        let dir = normalize(dir);
        existing
            .parent()
            .map(|parent| normalize(parent) == dir)
            .unwrap_or(false)
    });
    if owned_dir && force {
        return Ok(());
    }

    if force {
        UI::warn(&format!(
            "'{}' will shadow {} — it was not created by vx",
            name,
            existing.display()
        ));
        return Ok(());
    }

    bail!(
        "'{}' already resolves to {} on PATH. Re-run with --force to shadow it, \
         or pick another name with --as.",
        name,
        existing.display()
    )
}

/// Pair each directory with whether it is on PATH
fn path_status(dirs: &[PathBuf]) -> Vec<(PathBuf, bool)> {
    let entries: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|value| {
            std::env::split_paths(&value)
                .filter(|entry| !entry.as_os_str().is_empty())
                .collect()
        })
        .unwrap_or_default();

    let on_path: Vec<PathBuf> = entries.iter().map(|entry| normalize(entry)).collect();

    dirs.iter()
        .map(|dir| (dir.clone(), on_path.contains(&normalize(dir))))
        .collect()
}

/// Print a hint for any target directory that is not on PATH
fn report_path_status(dirs: &[PathBuf]) {
    let missing: Vec<PathBuf> = path_status(dirs)
        .into_iter()
        .filter(|(_, on_path)| !on_path)
        .map(|(dir, _)| dir)
        .collect();

    if missing.is_empty() {
        return;
    }

    UI::warn("These directories are not on PATH, so the command will not resolve yet:");
    for dir in &missing {
        UI::detail(&dir.display().to_string());
    }
    if cfg!(windows) {
        UI::hint(&format!(
            "$env:PATH = \"{};$env:PATH\"",
            missing
                .iter()
                .map(|dir| dir.display().to_string())
                .collect::<Vec<_>>()
                .join(";")
        ));
    } else {
        UI::hint(&format!(
            "export PATH=\"{}:$PATH\"",
            missing
                .iter()
                .map(|dir| dir.display().to_string())
                .collect::<Vec<_>>()
                .join(":")
        ));
    }
}

/// Human-readable summary of the files one shim produces on this platform
fn describe_variants(platform: &Platform) -> String {
    ShimType::platform_variants(platform)
        .iter()
        .map(|variant| match variant {
            ShimType::Batch => "<name>.cmd (cmd.exe, PowerShell)",
            ShimType::PowerShell => "<name>.ps1 (PowerShell)",
            ShimType::Shell => "<name> (sh, Git Bash, MSYS2)",
        })
        .collect::<Vec<_>>()
        .join(", ")
}
