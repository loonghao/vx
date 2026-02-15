//! Global package command handlers

use super::args::{
    GlobalCommand, GlobalListFormat, InfoGlobalArgs, InstallGlobalArgs, ListGlobalArgs,
    UninstallGlobalArgs,
};
use crate::commands::CommandContext;
use crate::ui::{ProgressSpinner, UI, progress_manager};
use anyhow::{Context, Result};
use colored::Colorize;
use std::io::Write;
use vx_ecosystem_pm::{InstallOptions, get_installer};
use vx_paths::global_packages::{GlobalPackage, PackageRegistry};
use vx_paths::package_spec::PackageSpec;
use vx_paths::shims;

/// Ensure a runtime is installed, auto-installing if necessary
///
/// Returns `Some(version)` if the runtime is installed (either was already installed or was just installed),
/// `None` if the runtime is not available.
async fn ensure_runtime_installed(
    ctx: &CommandContext,
    runtime_name: &str,
    _verbose: bool,
) -> Result<Option<String>> {
    // Check if runtime is already available
    if let Some(runtime) = ctx.registry().get_runtime(runtime_name) {
        let context = ctx.runtime_context();

        // Check if already installed - get the installed version
        let spinner = ProgressSpinner::new(&format!("Checking {} installation...", runtime_name));
        let installed_versions: Vec<String> = runtime
            .installed_versions(context)
            .await
            .unwrap_or_default();

        if !installed_versions.is_empty() {
            let version = installed_versions.last().cloned().unwrap_or_default();
            spinner.finish_with_message(&format!(
                "{} {} v{} already installed",
                "✓".green(),
                runtime_name,
                version
            ));
            return Ok(Some(version));
        }
        spinner.finish_and_clear();

        // Not installed, auto-install with progress
        let pm = progress_manager();
        let install_spinner = pm.add_spinner(&format!("Auto-installing {}...", runtime_name));

        // Fetch versions
        install_spinner.set_message(&format!("Fetching versions for {}...", runtime_name));
        let versions = match runtime.fetch_versions(context).await {
            Ok(v) => v,
            Err(e) => {
                install_spinner
                    .finish_error(&format!("Failed to fetch versions for {}", runtime_name));
                return Err(anyhow::anyhow!(
                    "Failed to fetch versions for {}: {}. Please install it manually.",
                    runtime_name,
                    e
                ));
            }
        };

        let version = versions
            .iter()
            .find(|v| !v.prerelease)
            .map(|v| v.version.clone())
            .or_else(|| versions.first().map(|v| v.version.clone()))
            .ok_or_else(|| anyhow::anyhow!("No versions found for {}", runtime_name))?;

        install_spinner.set_message(&format!("Installing {} v{}...", runtime_name, version));

        // Run pre-install hook
        runtime.pre_install(&version, context).await?;

        // Install the runtime
        let result = runtime.install(&version, context).await?;

        // Verify the installation
        if !context.fs.exists(&result.executable_path) {
            install_spinner.finish_error(&format!("Installation of {} failed", runtime_name));
            return Err(anyhow::anyhow!(
                "Installation completed but executable not found at {}",
                result.executable_path.display()
            ));
        }

        // Run post-install hook
        runtime.post_install(&version, context).await?;

        install_spinner.finish_success(&format!("Installed {} v{}", runtime_name, version));

        Ok(Some(version))
    } else {
        Err(anyhow::anyhow!(
            "Runtime {} not found in registry. Cannot auto-install.",
            runtime_name
        ))
    }
}

/// Handle global package commands
pub async fn handle(ctx: &CommandContext, command: &GlobalCommand) -> Result<()> {
    match command {
        GlobalCommand::Install(args) => handle_install(ctx, args).await,
        GlobalCommand::List(args) => handle_list(ctx, args).await,
        GlobalCommand::Uninstall(args) => handle_uninstall(ctx, args).await,
        GlobalCommand::Info(args) => handle_info(ctx, args).await,
        GlobalCommand::ShimUpdate => handle_shim_update(ctx).await,
    }
}

/// Get the required runtime for an ecosystem
fn get_required_runtime_for_ecosystem(ecosystem: &str) -> Option<&'static str> {
    match ecosystem.to_lowercase().as_str() {
        // Node.js ecosystem requires node (which provides npm)
        "npm" | "node" | "yarn" | "pnpm" | "bun" => Some("node"),
        // Python ecosystem requires uv or python
        "pip" | "python" | "pypi" => Some("uv"),
        "uv" => None, // uv is self-contained
        // Rust ecosystem
        "cargo" | "rust" | "crates" => Some("cargo"),
        // Go ecosystem
        "go" | "golang" => Some("go"),
        // Ruby ecosystem
        "gem" | "ruby" | "rubygems" => Some("ruby"),
        // Windows ecosystem (choco is self-contained when managed by vx)
        "choco" | "chocolatey" => Some("choco"),
        _ => None,
    }
}

/// Handle install-global command
async fn handle_install(ctx: &CommandContext, args: &InstallGlobalArgs) -> Result<()> {
    // Parse package specification
    let spec = PackageSpec::parse(&args.package)
        .with_context(|| format!("Invalid package specification: {}", args.package))?;

    if args.verbose {
        UI::detail(&format!(
            "Parsed: ecosystem={}, package={}, version={:?}",
            spec.ecosystem, spec.package, spec.version
        ));
    }

    // Load registry
    let paths = ctx.runtime_context().paths.clone();
    let registry_path = paths.packages_registry_file();
    let mut registry = PackageRegistry::load_or_create(&registry_path)?;

    // Check if already installed
    if let Some(existing) = registry.get(&spec.ecosystem, &spec.package) {
        if !args.force {
            UI::warn(&format!(
                "{} {} is already installed (version {})",
                spec.ecosystem, spec.package, existing.version
            ));
            UI::hint("Use --force to reinstall");
            return Ok(());
        }
        UI::info(&format!(
            "Reinstalling {} {} (was version {})...",
            spec.ecosystem, spec.package, existing.version
        ));
    }

    // Get version to install
    let version = spec.version.as_deref().unwrap_or("latest");

    // Create multi-step progress
    let pm = progress_manager();
    let steps = vec![
        format!("Preparing {}:{}", spec.ecosystem, spec.package),
        format!("Installing {}@{}", spec.package, version),
        "Creating shims".to_string(),
    ];
    let mut progress = crate::ui::MultiProgress::new(steps);

    // Step 1: Ensure runtime dependency
    let (runtime_name, runtime_version) = match get_required_runtime_for_ecosystem(&spec.ecosystem)
    {
        Some(required_runtime) => {
            let version = ensure_runtime_installed(ctx, required_runtime, args.verbose).await?;
            (Some(required_runtime), version)
        }
        None => (None, None),
    };

    // Get the appropriate installer for this ecosystem
    let installer: Box<dyn vx_ecosystem_pm::EcosystemInstaller> = match spec.ecosystem.as_str() {
        "npm" | "node" => {
            let npm_path = if runtime_version.is_some() {
                match vx_paths::get_bundled_tool_path("node", "npm") {
                    Ok(Some(path)) => Some(path),
                    Ok(None) => {
                        tracing::warn!("npm not found in installed node");
                        None
                    }
                    Err(e) => {
                        tracing::warn!("Failed to get npm path from RuntimeRoot: {}", e);
                        None
                    }
                }
            } else {
                None
            };

            if let Some(path) = npm_path {
                if path.exists() {
                    if args.verbose {
                        UI::detail(&format!("Using npm from: {}", path.display()));
                    }
                    Box::new(vx_ecosystem_pm::installers::NpmInstaller::with_npm_path(
                        path,
                    ))
                } else {
                    tracing::warn!("npm path does not exist: {}", path.display());
                    Box::new(vx_ecosystem_pm::installers::NpmInstaller::new())
                }
            } else {
                Box::new(vx_ecosystem_pm::installers::NpmInstaller::new())
            }
        }
        _ => get_installer(&spec.ecosystem)
            .with_context(|| format!("Unsupported ecosystem: {}", spec.ecosystem))?,
    };

    // Step 2: Perform installation
    progress.next_step();

    let options = InstallOptions {
        force: args.force,
        verbose: args.verbose,
        runtime_version: None,
        extra_args: args.extra_args.clone(),
    };

    let install_dir = paths.global_package_dir(&spec.ecosystem, &spec.package, version);

    let install_spinner = pm.add_spinner(&format!(
        "Installing {}:{}@{}...",
        spec.ecosystem, spec.package, version
    ));

    let result = installer
        .install(&install_dir, &spec.package, version, &options)
        .await
        .with_context(|| {
            format!(
                "Failed to install {}:{}@{}",
                spec.ecosystem, spec.package, version
            )
        });

    let result = match result {
        Ok(r) => {
            install_spinner.finish_success(&format!(
                "Installed {}:{}@{}",
                spec.ecosystem, spec.package, version
            ));
            r
        }
        Err(e) => {
            install_spinner.finish_error(&format!(
                "Failed to install {}:{}",
                spec.ecosystem, spec.package
            ));
            return Err(e);
        }
    };

    // Register package
    let mut global_package = GlobalPackage::new(
        spec.package.clone(),
        result.version.clone(),
        spec.ecosystem.clone(),
        result.install_dir.clone(),
    )
    .with_executables(result.executables.clone());

    if let (Some(rt_name), Some(rt_version)) = (runtime_name, runtime_version) {
        global_package = global_package.with_runtime_dependency(rt_name, rt_version);
        if args.verbose {
            UI::detail(&format!(
                "Package has runtime dependency: {}@{}",
                rt_name,
                global_package
                    .runtime_dependency
                    .as_ref()
                    .map(|d| d.version.as_str())
                    .unwrap_or("unknown")
            ));
        }
    }

    registry.register(global_package);
    registry.save(&registry_path)?;

    // Step 3: Create shims
    progress.next_step();

    if !result.executables.is_empty() && args.verbose {
        UI::detail(&format!(
            "Detected executables: {}",
            result.executables.join(", ")
        ));
    }

    let shims_dir = paths.shims_dir();
    let bin_dir = result.bin_dir.clone();

    let mut shim_count = 0;
    for exe in &result.executables {
        let exe_path = bin_dir.join(if cfg!(windows) {
            format!("{}.exe", exe)
        } else {
            exe.to_string()
        });

        let target_path = if exe_path.exists() {
            exe_path
        } else {
            bin_dir.join(exe)
        };

        if target_path.exists() {
            match shims::create_shim(&shims_dir, exe, &target_path) {
                Ok(_) => {
                    shim_count += 1;
                    if args.verbose {
                        UI::detail(&format!("Created shim for: {}", exe));
                    }
                }
                Err(e) => {
                    UI::warn(&format!("Failed to create shim for {}: {}", exe, e));
                }
            }
        } else if args.verbose {
            UI::warn(&format!(
                "Executable not found for shim: {}",
                target_path.display()
            ));
        }
    }

    // Final summary
    progress.finish(&format!(
        "{} Installed {}:{} {}",
        "✓".green(),
        spec.ecosystem,
        spec.package,
        result.version
    ));

    if shim_count > 0 {
        UI::success(&format!("Created {} shim(s)", shim_count));
        UI::hint(&format!(
            "Add {} to your PATH to use global tools directly",
            shims_dir.display()
        ));
    }

    Ok(())
}

/// Handle list-global command
async fn handle_list(ctx: &CommandContext, args: &ListGlobalArgs) -> Result<()> {
    let paths = ctx.runtime_context().paths.clone();
    let registry_path = paths.packages_registry_file();

    // Load registry
    let registry = match PackageRegistry::load(&registry_path) {
        Ok(r) => r,
        Err(_) => {
            UI::info("No global packages installed.");
            return Ok(());
        }
    };

    // Filter by ecosystem if specified
    let packages: Vec<_> = if let Some(eco) = &args.ecosystem {
        registry.list_by_ecosystem(eco).collect()
    } else {
        registry.all_packages().collect()
    };

    if packages.is_empty() {
        if let Some(eco) = &args.ecosystem {
            UI::info(&format!(
                "No global packages installed for ecosystem: {}",
                eco
            ));
        } else {
            UI::info("No global packages installed.");
        }
        return Ok(());
    }

    match args.format {
        GlobalListFormat::Json => {
            let json = serde_json::to_string_pretty(&packages)?;
            println!("{}", json);
        }
        GlobalListFormat::Plain => {
            for pkg in &packages {
                println!("{}:{}@{}", pkg.ecosystem, pkg.name, pkg.version);
            }
        }
        GlobalListFormat::Table => {
            println!(
                "\n{:<12} {:<25} {:<12} EXECUTABLES",
                "ECOSYSTEM", "PACKAGE", "VERSION"
            );
            println!("{}", "-".repeat(70));
            for pkg in &packages {
                let exes = pkg.executables.join(", ");
                println!(
                    "{:<12} {:<25} {:<12} {}",
                    pkg.ecosystem, pkg.name, pkg.version, exes
                );
            }
            println!();
            UI::detail(&format!("Total: {} package(s)", packages.len()));
        }
    }

    Ok(())
}

/// Handle uninstall-global command
async fn handle_uninstall(ctx: &CommandContext, args: &UninstallGlobalArgs) -> Result<()> {
    // Parse package specification (version not needed for uninstall)
    let spec = PackageSpec::parse(&args.package)
        .with_context(|| format!("Invalid package specification: {}", args.package))?;

    let paths = ctx.runtime_context().paths.clone();
    let registry_path = paths.packages_registry_file();

    // Load registry
    let mut registry = match PackageRegistry::load(&registry_path) {
        Ok(r) => r,
        Err(_) => {
            UI::error(&format!(
                "Package {}:{} is not installed",
                spec.ecosystem, spec.package
            ));
            return Ok(());
        }
    };

    // Check if package exists
    let package = match registry.get(&spec.ecosystem, &spec.package) {
        Some(p) => p.clone(),
        None => {
            UI::error(&format!(
                "Package {}:{} is not installed",
                spec.ecosystem, spec.package
            ));
            return Ok(());
        }
    };

    // Confirm removal
    if !args.force {
        print!(
            "Remove {}:{} {} from {}? [y/N] ",
            package.ecosystem,
            package.name,
            package.version,
            package.install_dir.display()
        );
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            UI::info("Cancelled.");
            return Ok(());
        }
    }

    // Multi-step uninstall with progress
    let pm = progress_manager();
    let uninstall_spinner = pm.add_spinner(&format!(
        "Removing {}:{} {}...",
        package.ecosystem, package.name, package.version
    ));

    // Remove package directory
    if package.install_dir.exists() {
        uninstall_spinner.set_message(&format!(
            "Removing package files for {}:{}...",
            package.ecosystem, package.name
        ));
        std::fs::remove_dir_all(&package.install_dir)
            .with_context(|| format!("Failed to remove {}", package.install_dir.display()))?;
    }

    // Remove shims
    uninstall_spinner.set_message(&format!(
        "Removing shims for {}:{}...",
        package.ecosystem, package.name
    ));
    let shims_dir = paths.shims_dir();
    let mut shim_count = 0;
    for exe in &package.executables {
        if shims::shim_exists(&shims_dir, exe) {
            shims::remove_shim(&shims_dir, exe)?;
            shim_count += 1;
            if args.verbose {
                UI::detail(&format!("Removed shim: {}", exe));
            }
        }
    }

    // Unregister from registry
    registry.unregister(&spec.ecosystem, &spec.package);
    registry.save(&registry_path)?;

    uninstall_spinner.finish_success(&format!(
        "Uninstalled {}:{} {}{}",
        spec.ecosystem,
        spec.package,
        package.version,
        if shim_count > 0 {
            format!(" ({} shims removed)", shim_count)
        } else {
            String::new()
        }
    ));

    Ok(())
}

/// Handle info-global command
async fn handle_info(ctx: &CommandContext, args: &InfoGlobalArgs) -> Result<()> {
    let paths = ctx.runtime_context().paths.clone();
    let registry_path = paths.packages_registry_file();

    let registry = match PackageRegistry::load(&registry_path) {
        Ok(r) => r,
        Err(_) => {
            UI::error(&format!("Package '{}' not found", args.package));
            return Ok(());
        }
    };

    // Try to find by executable name first
    let package = if let Some(pkg) = registry.find_by_executable(&args.package) {
        pkg.clone()
    } else {
        // Try to parse as package spec
        match PackageSpec::parse(&args.package) {
            Ok(spec) => match registry.get(&spec.ecosystem, &spec.package) {
                Some(p) => p.clone(),
                None => {
                    UI::error(&format!(
                        "Package '{}' not found (searched as executable and package name)",
                        args.package
                    ));
                    return Ok(());
                }
            },
            Err(_) => {
                // Try all ecosystems
                let ecosystems = ["npm", "pip", "cargo", "go", "gem", "choco"];
                let mut found = None;
                for eco in ecosystems {
                    if let Some(p) = registry.get(eco, &args.package) {
                        found = Some(p.clone());
                        break;
                    }
                }
                match found {
                    Some(p) => p,
                    None => {
                        UI::error(&format!("Package '{}' not found", args.package));
                        return Ok(());
                    }
                }
            }
        }
    };

    if args.json {
        let json = serde_json::to_string_pretty(&package)?;
        println!("{}", json);
    } else {
        println!("\nPackage: {}", package.name);
        println!("Version: {}", package.version);
        println!("Ecosystem: {}", package.ecosystem);
        println!("Installed at: {}", package.installed_at);
        println!("Location: {}", package.install_dir.display());
        if !package.executables.is_empty() {
            println!("Executables: {}", package.executables.join(", "));
        }
        if let Some(ref runtime) = package.runtime_dependency {
            println!("Runtime: {}@{}", runtime.runtime, runtime.version);
        }
        println!();
    }

    Ok(())
}

/// Handle shim-update command
async fn handle_shim_update(ctx: &CommandContext) -> Result<()> {
    let paths = ctx.runtime_context().paths.clone();
    let registry_path = paths.packages_registry_file();

    let registry = match PackageRegistry::load(&registry_path) {
        Ok(r) => r,
        Err(_) => {
            UI::info("No global packages installed. Nothing to update.");
            return Ok(());
        }
    };

    // Collect all executables
    let mut packages_with_exes: Vec<(String, std::path::PathBuf)> = Vec::new();
    for pkg in registry.all_packages() {
        let bin_dir = paths.global_package_bin_dir(&pkg.ecosystem, &pkg.name, &pkg.version);
        for exe in &pkg.executables {
            let exe_path = bin_dir.join(exe);
            packages_with_exes.push((exe.clone(), exe_path));
        }
    }

    if packages_with_exes.is_empty() {
        UI::info("No executables to create shims for.");
        return Ok(());
    }

    // Sync shims
    let shims_dir = paths.shims_dir();
    let result = shims::sync_shims_from_registry(&shims_dir, &packages_with_exes)?;

    UI::success(&format!(
        "Shims updated: {} created, {} removed",
        result.created, result.removed
    ));

    if !result.errors.is_empty() {
        UI::warn("Errors encountered:");
        for err in &result.errors {
            UI::error(err);
        }
    }

    UI::hint(&format!(
        "Add {} to your PATH to use global tools directly",
        shims_dir.display()
    ));

    Ok(())
}
