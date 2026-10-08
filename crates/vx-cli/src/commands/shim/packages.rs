//! Compatibility integration for explicit global package installation and removal.

use anyhow::{Context, Result, bail};
use vx_paths::global_packages::GlobalPackage;
use vx_runtime::ShimRegistry;
use vx_shim::PackageRequest;

use super::args::{AddShimArgs, RemoveShimArgs};
use super::handler::{handle_add, handle_remove};
use super::target::resolve_target;
use crate::commands::CommandContext;

/// Publish explicitly installed packages through the command shim lifecycle.
pub(crate) fn publish_package(ctx: &CommandContext, package: &GlobalPackage) -> Result<usize> {
    let path = ShimRegistry::default_path(&ctx.runtime_context().paths.config_dir());
    for executable in &package.executables {
        let mut target = format!("{}:{}::{}", package.ecosystem, package.name, executable);
        let registry = ShimRegistry::load(&path)?;
        if let Some(entry) = registry.get(executable) {
            let (existing, _) = resolve_target(ctx, &entry.runtime)?;
            let request = PackageRequest::parse(&existing).with_context(|| {
                format!("'{}' is already registered for another runtime", executable)
            })?;
            if request.ecosystem != package.ecosystem || request.package != package.name {
                bail!("'{}' is already registered for another package", executable);
            }
            if request.is_shell_request() || request.executable_name() != executable {
                bail!(
                    "'{}' is already registered for another executable",
                    executable
                );
            }
            target.clone_from(&entry.runtime);
        }
        handle_add(
            ctx,
            &AddShimArgs {
                runtime: target,
                name: Some(executable.clone()),
                dir: Vec::new(),
                force: false,
            },
        )?;
    }
    Ok(package.executables.len())
}

/// Remove only registered commands belonging to the package being uninstalled.
pub(crate) fn remove_package_shims(ctx: &CommandContext, package: &GlobalPackage) -> Result<usize> {
    let path = ShimRegistry::default_path(&ctx.runtime_context().paths.config_dir());
    let registry = ShimRegistry::load(&path)?;
    let mut removed = 0;
    for entry in registry.shims {
        let (target, _) = resolve_target(ctx, &entry.runtime)?;
        if !PackageRequest::is_package_request(&target) {
            continue;
        }
        let request = PackageRequest::parse(&target)?;
        if request.ecosystem == package.ecosystem && request.package == package.name {
            handle_remove(
                ctx,
                &RemoveShimArgs {
                    name: entry.name,
                    force: true,
                },
            )?;
            removed += 1;
        }
    }
    Ok(removed)
}
