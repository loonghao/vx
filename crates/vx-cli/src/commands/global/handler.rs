//! Global package command handlers

use super::args::{
    GlobalCommand, InfoGlobalArgs, InstallGlobalArgs, ListGlobalArgs, OutputFormat,
    UninstallGlobalArgs,
};
use crate::commands::CommandContext;
use crate::ui::UI;
use anyhow::{Context, Result};
use std::io::Write;
use vx_ecosystem_pm::{get_installer, InstallOptions};
use vx_paths::global_packages::{GlobalPackage, PackageRegistry};
use vx_paths::package_spec::PackageSpec;
use vx_paths::shims;

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
    UI::info(&format!(
        "Installing {}:{}@{}...",
        spec.ecosystem, spec.package, version
    ));

    // Get the appropriate installer for this ecosystem
    let installer = get_installer(&spec.ecosystem)
        .with_context(|| format!("Unsupported ecosystem: {}", spec.ecosystem))?;

    // Build install options
    let options = InstallOptions {
        force: args.force,
        verbose: args.verbose,
        runtime_version: None, // TODO: Support runtime version selection
        extra_args: args.extra_args.clone(),
    };

    // Get the installation directory
    let install_dir = paths.global_package_dir(&spec.ecosystem, &spec.package, version);

    // Perform the actual installation (async)
    let result = installer
        .install(&install_dir, &spec.package, version, &options)
        .await
        .with_context(|| {
            format!(
                "Failed to install {}:{}@{}",
                spec.ecosystem, spec.package, version
            )
        })?;

    // Create GlobalPackage from EcosystemInstallResult for registry
    let global_package = GlobalPackage::new(
        spec.package.clone(),
        result.version.clone(),
        spec.ecosystem.clone(),
        result.install_dir.clone(),
    )
    .with_executables(result.executables.clone());

    // Register package
    registry.register(global_package);
    registry.save(&registry_path)?;

    UI::success(&format!(
        "Installed {}:{} {} to {}",
        spec.ecosystem,
        spec.package,
        version,
        result.install_dir.display()
    ));

    // Report detected executables
    if !result.executables.is_empty() {
        UI::detail(&format!(
            "Detected executables: {}",
            result.executables.join(", ")
        ));
    }

    // Create shims for package executables
    let shims_dir = paths.shims_dir();
    let bin_dir = result.bin_dir.clone();

    let mut shim_count = 0;
    for exe in &result.executables {
        let exe_path = bin_dir.join(if cfg!(windows) {
            format!("{}.exe", exe)
        } else {
            exe.to_string()
        });

        // Try with the extension first, then without on Windows
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
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&packages)?;
            println!("{}", json);
        }
        OutputFormat::Plain => {
            for pkg in &packages {
                println!("{}:{}@{}", pkg.ecosystem, pkg.name, pkg.version);
            }
        }
        OutputFormat::Table => {
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

    if args.verbose {
        UI::detail(&format!(
            "Removing directory: {}",
            package.install_dir.display()
        ));
    }

    // Remove package directory
    if package.install_dir.exists() {
        std::fs::remove_dir_all(&package.install_dir)
            .with_context(|| format!("Failed to remove {}", package.install_dir.display()))?;
    }

    // Remove shims
    let shims_dir = paths.shims_dir();
    for exe in &package.executables {
        if shims::shim_exists(&shims_dir, exe) {
            shims::remove_shim(&shims_dir, exe)?;
            if args.verbose {
                UI::detail(&format!("Removed shim: {}", exe));
            }
        }
    }

    // Unregister from registry
    registry.unregister(&spec.ecosystem, &spec.package);
    registry.save(&registry_path)?;

    UI::success(&format!(
        "Uninstalled {}:{} {}",
        spec.ecosystem, spec.package, package.version
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
                let ecosystems = ["npm", "pip", "cargo", "go", "gem"];
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
