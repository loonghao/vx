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
use vx_runtime::{
    CommandShim, Platform, ShimRegistry, ShimType, create_command_shim_in_home,
    validate_command_shim_name,
};

use super::args::{AddShimArgs, ListShimArgs, RemoveShimArgs, ShimCommand};
use super::target::resolve_target;
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
pub(super) fn handle_add(ctx: &CommandContext, args: &AddShimArgs) -> Result<()> {
    let (runtime, default_name) = resolve_target(ctx, &args.runtime)?;
    let name = args.name.as_deref().unwrap_or(&default_name);
    validate_command_shim_name(name)?;
    let paths = ctx.runtime_context().paths.clone();
    let registry_path = ShimRegistry::default_path(&paths.config_dir());
    let mut registry = ShimRegistry::load(&registry_path)?;

    let launcher = current_launcher()?;
    let vx_home = absolute_directory(&paths.vx_home())?;
    let previous = registry.get(name).cloned();
    if let Some(entry) = &previous {
        check_home(entry, &paths.vx_home())?;
    }
    let requested_dirs = if args.dir.is_empty() {
        previous
            .as_ref()
            .map(|entry| entry.dirs.as_slice())
            .unwrap_or(&[])
    } else {
        &args.dir
    };
    let dirs = target_dirs(&paths.bin_dir(), requested_dirs)?;

    check_collision(name, previous.as_ref(), args.force)?;

    let entry = create_command_shim_in_home(
        name,
        &runtime,
        &launcher,
        &vx_home,
        &dirs,
        &Platform::current(),
        previous.as_ref(),
    )?;
    let files = entry.files.clone();
    if let Some(previous) = &previous {
        for file in &previous.files {
            if !files
                .iter()
                .any(|current| normalize(current) == normalize(file))
                && previous.owns_file(file)
            {
                std::fs::remove_file(file)
                    .with_context(|| format!("Failed to remove old shim: {}", file.display()))?;
            }
        }
    }
    registry.upsert(entry);
    registry
        .save(&registry_path)
        .with_context(|| format!("Failed to save shim registry: {}", registry_path.display()))?;

    UI::success(&format!("'{}' now runs 'vx {}'", name, runtime));
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
    let registry = ShimRegistry::load(&registry_path)?;

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
pub(super) fn handle_remove(ctx: &CommandContext, args: &RemoveShimArgs) -> Result<()> {
    let paths = ctx.runtime_context().paths.clone();
    let registry_path = ShimRegistry::default_path(&paths.config_dir());
    let mut registry = ShimRegistry::load(&registry_path)?;

    if let Some(entry) = registry.get(&args.name) {
        check_home(entry, &paths.vx_home())?;
    }

    let Some(entry) = registry.remove(&args.name) else {
        bail!(
            "'{}' is not a vx command shim. Nothing was removed.",
            args.name
        );
    };

    for file in &entry.files {
        if file.symlink_metadata().is_ok() && !entry.owns_file(file) {
            UI::warn(&format!(
                "preserving modified or unowned file: {}",
                file.display()
            ));
        }
    }
    let removed = entry.remove_files()?.len();

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
    let mut registry = ShimRegistry::load(&registry_path)?;

    if registry.is_empty() {
        UI::info("No command shims to sync.");
        return Ok(());
    }

    let launcher = current_launcher()?;
    let vx_home = absolute_directory(&paths.vx_home())?;
    let platform = Platform::current();
    let entries: Vec<CommandShim> = registry.shims.clone();

    let mut rewritten = 0;
    let mut stale = 0;

    for entry in entries {
        check_home(&entry, &paths.vx_home())?;
        if entry.launcher != launcher || !entry.is_complete() {
            stale += 1;
        }

        let refreshed = create_command_shim_in_home(
            &entry.name,
            &entry.runtime,
            &launcher,
            &vx_home,
            &entry.dirs,
            &platform,
            Some(&entry),
        )?;
        registry.upsert(refreshed);
        registry.save(&registry_path)?;
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
    let dirs = target_dirs(&paths.bin_dir(), &[])?;

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

fn check_home(entry: &CommandShim, vx_home: &Path) -> Result<()> {
    if let Some(home) = &entry.vx_home
        && normalize(home) != normalize(&absolute_directory(vx_home)?)
    {
        bail!(
            "Shim '{}' belongs to another VX_HOME: {}",
            entry.name,
            home.display()
        );
    }
    Ok(())
}

/// Resolve directory paths without storing a shell-incompatible Windows prefix.
fn absolute_directory(home: &Path) -> Result<PathBuf> {
    let absolute = std::path::absolute(home)?;
    let resolved = if absolute.exists() {
        absolute.canonicalize()?
    } else {
        // Bind new homes and output directories through their existing parent.
        // Their spelling must remain stable after creation, including /var on
        // macOS and short user-directory names on Windows runners.
        let mut normalized = PathBuf::new();
        for component in absolute.components() {
            if component == std::path::Component::ParentDir {
                normalized.pop();
            } else {
                normalized.push(component);
            }
        }
        let mut ancestor = normalized.as_path();
        let mut missing = Vec::new();
        while !ancestor.exists() {
            if let Some(name) = ancestor.file_name() {
                missing.push(name.to_os_string());
            }
            ancestor = ancestor
                .parent()
                .context("Directory has no existing ancestor")?;
        }
        let mut resolved = ancestor.canonicalize()?;
        for name in missing.iter().rev() {
            resolved.push(name);
        }
        resolved
    };
    #[cfg(windows)]
    {
        let value = resolved.to_string_lossy();
        if let Some(unc) = value.strip_prefix(r"\\?\UNC\") {
            return Ok(PathBuf::from(format!(r"\\{unc}")));
        }
        if let Some(path) = value.strip_prefix(r"\\?\") {
            return Ok(PathBuf::from(path));
        }
    }
    Ok(resolved)
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
fn target_dirs(bin_dir: &Path, explicit: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let resolve = |dir: &Path| {
        anyhow::ensure!(
            !dir.is_symlink(),
            "Shim output directory cannot be a symbolic link: {}",
            dir.display()
        );
        absolute_directory(dir)
    };
    if !explicit.is_empty() {
        return Ok(dedup(
            explicit
                .iter()
                .map(|dir| resolve(dir))
                .collect::<Result<Vec<_>>>()?,
        ));
    }

    let mut dirs = vec![bin_dir.to_path_buf()];
    if let Ok(current) = std::env::current_exe()
        && let Some(parent) = current.parent()
    {
        dirs.push(parent.to_path_buf());
    }

    Ok(dedup(
        dirs.iter()
            .map(|dir| resolve(dir))
            .collect::<Result<Vec<_>>>()?,
    ))
}

/// Drop duplicate directories while keeping order
///
/// Comparison uses [`normalize`] so `C:\Bin` and `C:\bin\` collapse together on
/// Windows, but the original path is preserved — returning the normalized form
/// would break case-sensitive filesystems.
fn dedup(dirs: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen: Vec<PathBuf> = Vec::new();
    for dir in dirs {
        let key = normalize(&dir);
        if !seen.iter().any(|existing| normalize(existing) == key) {
            seen.push(dir);
        }
    }
    seen
}

/// Normalize Windows directory spelling while preserving Unix case sensitivity.
fn normalize(dir: &Path) -> PathBuf {
    let resolved = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    #[cfg(windows)]
    {
        let text = resolved.to_string_lossy().replace('\\', "/").to_lowercase();
        if let Some(unc) = text.strip_prefix("//?/unc/") {
            return PathBuf::from(format!("//{}", unc.trim_end_matches('/')));
        }
        PathBuf::from(
            text.strip_prefix("//?/")
                .unwrap_or(&text)
                .trim_end_matches('/'),
        )
    }
    #[cfg(not(windows))]
    {
        resolved
    }
}

/// Refuse to shadow a binary vx did not create unless `--force` is given
fn check_collision(name: &str, previous: Option<&CommandShim>, force: bool) -> Result<()> {
    let Ok(existing) = which::which(name) else {
        return Ok(());
    };

    if previous.is_some_and(|entry| entry.owns_file(&existing)) {
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
