//! Resolve command publication targets to stable runtime and package identities.

use anyhow::Result;
use vx_paths::global_packages::PackageRegistry;
use vx_resolver::RuntimeRequest;
use vx_shim::PackageRequest;

use crate::commands::CommandContext;

/// Resolve aliases once so publication and package removal share an identity.
pub(super) fn resolve_target(ctx: &CommandContext, spec: &str) -> Result<(String, String)> {
    if PackageRequest::is_package_request(spec) {
        let request = PackageRequest::parse(spec)?;
        let name = request
            .shell_name()
            .or(request.executable.as_deref())
            .unwrap_or_else(|| {
                request
                    .package
                    .rsplit('/')
                    .next()
                    .unwrap_or(&request.package)
            });
        let target = if request.executable.is_none() && request.shell.is_none() {
            format!("{spec}::{name}")
        } else {
            spec.to_string()
        };
        return Ok((target, name.to_string()));
    }
    let request = RuntimeRequest::parse(spec);
    if let Some(alias) = ctx.get_package_alias(&request.name) {
        let version = request
            .version
            .as_ref()
            .map(|v| format!("@{v}"))
            .unwrap_or_default();
        let executable = request
            .executable
            .as_deref()
            .or(request.shell.as_deref())
            .or(alias.executable.as_deref())
            .unwrap_or_else(|| alias.package.rsplit('/').next().unwrap_or(&alias.package));
        return Ok((
            format!(
                "{}:{}{}::{}",
                alias.ecosystem, alias.package, version, executable
            ),
            executable.to_string(),
        ));
    }
    if ctx.registry().get_runtime(&request.name).is_none() {
        let registry_path = ctx.runtime_context().paths.packages_registry_file();
        if registry_path.exists() {
            let registry = PackageRegistry::load(&registry_path)?;
            if let Some(package) = registry.find_by_executable(&request.name) {
                let version = request
                    .version
                    .as_ref()
                    .map(|version| format!("@{version}"))
                    .unwrap_or_default();
                let executable = request
                    .executable
                    .as_deref()
                    .or(request.shell.as_deref())
                    .unwrap_or(&request.name);
                return Ok((
                    format!(
                        "{}:{}{}::{}",
                        package.ecosystem, package.name, version, executable
                    ),
                    executable.to_string(),
                ));
            }
        }
    }
    let name = request.executable.or(request.shell).unwrap_or(request.name);
    Ok((spec.to_string(), name))
}
