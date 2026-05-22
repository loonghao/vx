//! Environment preparation and PATH building
//!
//! This module handles:
//! - Preparing runtime environment variables
//! - Template expansion ({install_dir}, {version}, etc.)
//! - Building the vx tools PATH for subprocess execution

use crate::{Resolver, ResolverConfig, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, trace, warn};
use vx_runtime::{ProviderRegistry, RuntimeContext};

use super::bin_dir_cache;
use super::project_config::ProjectToolsConfig;
use super::version_utils;

/// Environment preparation manager
pub struct EnvironmentManager<'a> {
    #[allow(dead_code)]
    pub(crate) config: &'a ResolverConfig,
    pub(crate) resolver: &'a Resolver,
    pub(crate) registry: Option<&'a ProviderRegistry>,
    pub(crate) context: Option<&'a RuntimeContext>,
    pub(crate) project_config: Option<&'a ProjectToolsConfig>,
}

impl<'a> EnvironmentManager<'a> {
    /// Create a new environment manager
    pub fn new(
        config: &'a ResolverConfig,
        resolver: &'a Resolver,
        registry: Option<&'a ProviderRegistry>,
        context: Option<&'a RuntimeContext>,
        project_config: Option<&'a ProjectToolsConfig>,
    ) -> Self {
        Self {
            config,
            resolver,
            registry,
            context,
            project_config,
        }
    }

    /// Prepare environment variables for a runtime
    pub async fn prepare_runtime_environment(
        &self,
        runtime_name: &str,
        version: Option<&str>,
        inherit_env: bool,
    ) -> Result<HashMap<String, String>> {
        let mut env = HashMap::new();

        // Get the provider runtime info for bundled tools
        let (effective_runtime_name, effective_version) =
            self.get_provider_runtime(runtime_name, version);

        let effective_runtime_name_ref = effective_runtime_name.as_deref().unwrap_or(runtime_name);
        let effective_version_ref = effective_version.as_deref().or(version);

        debug!(
            "  prepare_env for {} (effective: {}@{:?})",
            runtime_name, effective_runtime_name_ref, effective_version_ref
        );

        // Get runtime spec for base environment variables
        if let Some(spec) = self.resolver.get_spec(effective_runtime_name_ref) {
            // Handle env_config if present
            if let Some(env_config) = &spec.env_config {
                // Handle advanced environment configuration
                if let Some(advanced) = &env_config.advanced {
                    // Handle PATH manipulation
                    let mut path_parts = Vec::new();

                    // Prepend entries
                    for entry in &advanced.path_prepend {
                        let expanded = self.expand_template(
                            entry,
                            effective_runtime_name_ref,
                            effective_version_ref,
                        )?;
                        path_parts.push(expanded);
                    }

                    // Get current PATH
                    let isolate_env = if inherit_env { false } else { advanced.isolate };
                    let current_path = if !isolate_env {
                        std::env::var("PATH").unwrap_or_default()
                    } else {
                        // When isolated, filter PATH to only include system directories
                        if let Ok(full_path) = std::env::var("PATH") {
                            vx_manifest::filter_system_path(&full_path)
                        } else {
                            String::new()
                        }
                    };

                    // Split current_path and add each directory
                    if !current_path.is_empty() {
                        for part in vx_paths::split_path(&current_path) {
                            path_parts.push(part.to_string());
                        }
                    }

                    // Ensure essential system paths are present
                    // This covers both Unix (sh, bash, etc.) and Windows (cmd.exe, PowerShell)
                    {
                        #[cfg(unix)]
                        let essential_paths = ["/bin", "/usr/bin", "/usr/local/bin"];

                        #[cfg(windows)]
                        let essential_paths = {
                            let system_root = std::env::var("SYSTEMROOT")
                                .unwrap_or_else(|_| r"C:\Windows".to_string());
                            let system32 = format!(r"{}\System32", system_root);
                            [
                                system32.clone(),
                                format!(r"{}\Wbem", system32),
                                format!(r"{}\WindowsPowerShell\v1.0", system32),
                            ]
                        };

                        for essential in &essential_paths {
                            let essential_str = essential.to_string();
                            #[cfg(unix)]
                            let already_present = path_parts.iter().any(|p| p == &essential_str);
                            #[cfg(windows)]
                            let already_present = path_parts
                                .iter()
                                .any(|p| p.eq_ignore_ascii_case(&essential_str));

                            if !already_present && std::path::Path::new(essential).exists() {
                                path_parts.push(essential_str);
                                trace!("Added essential system path: {}", essential);
                            }
                        }
                    }

                    // Append entries
                    for entry in &advanced.path_append {
                        let expanded = self.expand_template(
                            entry,
                            effective_runtime_name_ref,
                            effective_version_ref,
                        )?;
                        path_parts.push(expanded);
                    }

                    // Ensure vx executable's own directory is in PATH
                    // so sub-processes (e.g., just recipes) can call `vx`
                    if let Ok(current_exe) = std::env::current_exe()
                        && let Some(exe_dir) = current_exe.parent()
                    {
                        let exe_dir_str = exe_dir.to_string_lossy().to_string();
                        if !path_parts.iter().any(|p| p == &exe_dir_str) {
                            path_parts.push(exe_dir_str);
                        }
                    }

                    // Set PATH
                    if !path_parts.is_empty()
                        && let Ok(new_path) = std::env::join_paths(&path_parts)
                    {
                        let path_str = new_path.to_string_lossy().to_string();
                        env.insert("PATH".to_string(), path_str);
                    }

                    // Handle advanced env vars
                    for (var_name, var_config) in &advanced.env_vars {
                        match var_config {
                            vx_manifest::EnvVarConfig::Simple(value) => {
                                let expanded = self.expand_template(
                                    value,
                                    effective_runtime_name_ref,
                                    effective_version_ref,
                                )?;
                                env.insert(var_name.clone(), expanded);
                            }
                            vx_manifest::EnvVarConfig::Advanced {
                                value,
                                prepend,
                                append,
                                ..
                            } => {
                                // Build the value
                                let mut parts = Vec::new();
                                if let Some(pre_list) = prepend {
                                    for pre in pre_list {
                                        parts.push(self.expand_template(
                                            pre,
                                            effective_runtime_name_ref,
                                            effective_version_ref,
                                        )?);
                                    }
                                }
                                if let Some(val) = value {
                                    parts.push(self.expand_template(
                                        val,
                                        effective_runtime_name_ref,
                                        effective_version_ref,
                                    )?);
                                }
                                if let Some(app_list) = append {
                                    for app in app_list {
                                        parts.push(self.expand_template(
                                            app,
                                            effective_runtime_name_ref,
                                            effective_version_ref,
                                        )?);
                                    }
                                }
                                if !parts.is_empty() {
                                    env.insert(var_name.clone(), parts.join(""));
                                }
                            }
                        }
                    }

                    // Get effective inherit_system_vars (defaults + provider-specific)
                    let inherit_vars = env_config.effective_inherit_system_vars();

                    // Inherit system vars (excluding PATH which is handled above)
                    for var_pattern in &inherit_vars {
                        if var_pattern == "PATH" {
                            continue;
                        }

                        // Handle glob patterns like "LC_*"
                        if var_pattern.contains('*') {
                            let prefix = var_pattern.trim_end_matches('*');
                            for (key, value) in std::env::vars() {
                                if key.starts_with(prefix) && !env.contains_key(&key) {
                                    env.insert(key, value);
                                }
                            }
                        } else if let Ok(value) = std::env::var(var_pattern)
                            && !env.contains_key(var_pattern)
                        {
                            env.insert(var_pattern.clone(), value);
                        }
                    }
                } else if inherit_env {
                    // No advanced config, but inherit_env requested - inherit everything
                    for (key, value) in std::env::vars() {
                        env.entry(key).or_insert(value);
                    }
                }
            } else if inherit_env {
                // No env_config, but inherit_env requested - inherit everything
                for (key, value) in std::env::vars() {
                    env.entry(key).or_insert(value);
                }
            }

            // Add basic env_vars from spec
            for (key, value) in &spec.env_vars {
                let expanded =
                    self.expand_template(value, effective_runtime_name_ref, effective_version_ref)?;
                env.insert(key.clone(), expanded);
            }
        } else {
            trace!("No spec found for {}", effective_runtime_name_ref);
        }

        // If we don't have registry and context, return what we have
        let (registry, context) = match (self.registry, self.context) {
            (Some(r), Some(c)) => (r, c),
            _ => return Ok(env),
        };

        // Get the runtime
        let runtime = match registry.get_runtime(runtime_name) {
            Some(r) => r,
            None => return Ok(env),
        };

        // Determine the version to use
        let version = match version {
            Some(v) => v.to_string(),
            None => {
                // Try to get the installed version from the store
                match runtime.installed_versions(context).await {
                    Ok(versions) if !versions.is_empty() => versions[0].clone(),
                    _ => return Ok(env),
                }
            }
        };

        // For bundled runtimes (e.g., npm bundled with node), we must also call the
        // parent runtime's execution_environment() so that the parent's bin/ directory
        // is added to PATH. Without this, npm's shell script (#!/usr/bin/env node)
        // cannot find `node`, causing `node ci` to be executed instead of `npm ci`.
        if let Some(ref provider_name) = effective_runtime_name
            && provider_name != runtime_name
            && let Some(provider_runtime) = registry.get_runtime(provider_name)
        {
            // Find the installed version of the parent runtime
            let provider_version = match provider_runtime.installed_versions(context).await {
                Ok(versions) if !versions.is_empty() => {
                    let mut sorted = versions;
                    sorted.sort_by(|a, b| self.compare_versions(a, b));
                    sorted
                        .last()
                        .cloned()
                        .unwrap_or_else(|| "latest".to_string())
                }
                _ => "latest".to_string(),
            };
            debug!(
                "[prepare_env] Bundled runtime {} → injecting parent {} ({}) environment",
                runtime_name, provider_name, provider_version
            );
            // ManifestDrivenRuntime::execution_environment() returns an empty HashMap,
            // so we must read the parent runtime's PATH directly from its spec's
            // env_config.advanced.path_prepend and expand the templates ourselves.
            if let Some(parent_spec) = self.resolver.get_spec(provider_name)
                && let Some(parent_env_config) = &parent_spec.env_config
                && let Some(parent_advanced) = &parent_env_config.advanced
            {
                let mut extra_paths: Vec<String> = Vec::new();
                for entry in &parent_advanced.path_prepend {
                    match self.expand_template(entry, provider_name, Some(&provider_version)) {
                        Ok(expanded) => {
                            debug!("[prepare_env]   parent PATH entry: {}", expanded);
                            extra_paths.push(expanded);
                        }
                        Err(e) => {
                            warn!(
                                "Failed to expand parent PATH template '{}' for {}: {}",
                                entry, provider_name, e
                            );
                        }
                    }
                }
                if !extra_paths.is_empty() {
                    // Prepend parent bin dirs to the PATH already in env
                    let current_path = env
                        .get("PATH")
                        .cloned()
                        .unwrap_or_else(|| std::env::var("PATH").unwrap_or_default());
                    let mut all_parts = extra_paths;
                    if !current_path.is_empty() {
                        for part in vx_paths::split_path(&current_path) {
                            all_parts.push(part.to_string());
                        }
                    }
                    if let Ok(new_path) = std::env::join_paths(&all_parts) {
                        env.insert("PATH".to_string(), new_path.to_string_lossy().to_string());
                    }
                }
            }
        }

        // Call execution_environment for the primary runtime being invoked
        // This uses execution_environment() which may provide additional env vars
        // needed only when the tool is directly invoked (e.g., MSVC's LIB/INCLUDE/PATH
        // are only needed when directly running cl/link/nmake, not when running npm)
        // See: https://github.com/loonghao/vx/issues/573
        match runtime.execution_environment(&version, context).await {
            Ok(runtime_env) => {
                env.extend(runtime_env);
            }
            Err(e) => {
                warn!(
                    "Failed to prepare environment for {} {}: {}",
                    runtime_name, version, e
                );
            }
        }

        // Add RuntimeRoot environment variables (VX_{NAME}_ROOT, VX_{NAME}_BIN, etc.)
        // This provides REZ-like environment variables for dependency discovery
        let vx_paths = vx_paths::VxPaths::with_base_dir(context.paths.vx_home());
        if let Ok(Some(runtime_root)) =
            vx_paths::RuntimeRoot::find(runtime_name, &version, &vx_paths)
        {
            let root_env_vars = runtime_root.env_vars();
            debug!(
                "Adding RuntimeRoot env vars for {}: {:?}",
                runtime_name,
                root_env_vars.keys().collect::<Vec<_>>()
            );
            env.extend(root_env_vars);
        }

        // RFC 0028: Add RuntimeRoot environment variables for dependencies
        // This ensures dependent runtimes (like yarn depending on node) can find
        // the correct dependency executable via VX_{DEP}_BIN environment variables
        //
        // We check both static dependencies (from RuntimeSpec) and version-specific
        // dependencies (from provider.toml constraints like when=">=2, <4")
        if let Some(spec) = self.resolver.get_spec(runtime_name) {
            // Static dependencies
            for dep in &spec.dependencies {
                if dep.required {
                    let dep_runtime_name = dep.provided_by.as_deref().unwrap_or(&dep.runtime_name);

                    // Find the latest installed version of the dependency
                    if let Ok(Some(dep_root)) =
                        vx_paths::RuntimeRoot::find_latest(dep_runtime_name, &vx_paths)
                    {
                        let dep_env_vars = dep_root.env_vars();
                        debug!(
                            "Adding RuntimeRoot env vars for static dependency {}: {:?}",
                            dep_runtime_name,
                            dep_env_vars.keys().collect::<Vec<_>>()
                        );
                        env.extend(dep_env_vars);
                    }
                }
            }
        }

        // Version-specific dependencies (from provider.toml constraints)
        let version_deps = self
            .resolver
            .get_dependencies_for_version(runtime_name, &version);
        for dep in version_deps {
            if dep.required {
                let dep_runtime_name = dep.provided_by.as_deref().unwrap_or(&dep.runtime_name);

                // Skip if we already added this dependency
                let env_key = format!(
                    "VX_{}_BIN",
                    dep_runtime_name.to_uppercase().replace('-', "_")
                );
                if env.contains_key(&env_key) {
                    continue;
                }

                // Find the latest installed version of the dependency
                if let Ok(Some(dep_root)) =
                    vx_paths::RuntimeRoot::find_latest(dep_runtime_name, &vx_paths)
                {
                    let dep_env_vars = dep_root.env_vars();
                    debug!(
                        "Adding RuntimeRoot env vars for version-specific dependency {} (for {}@{}): {:?}",
                        dep_runtime_name,
                        runtime_name,
                        version,
                        dep_env_vars.keys().collect::<Vec<_>>()
                    );
                    env.extend(dep_env_vars);
                }
            }
        }

        // Inject companion tools' prepare_environment() from vx.toml when they
        // are already available.
        //
        // When vx.toml specifies tools like [tools.msvc], other executions can
        // have MSVC's marker environment variables (VCINSTALLDIR,
        // VCToolsInstallDir, VX_MSVC_*, etc.) injected. This enables tools that
        // need a C/C++ compiler to discover an existing vx-managed MSVC
        // installation.
        //
        // This only calls prepare_environment() (not execution_environment()), so it
        // injects discovery/marker variables without polluting LIB/INCLUDE/PATH.
        // It must not install or repair companions while preparing an unrelated
        // command: a successful `vx git --version` should not emit MSVC repair
        // failures just because the project declares msvc.
        //
        // See: https://github.com/loonghao/vx/issues/573
        // See: https://github.com/loonghao/vx/issues/889
        if let Some(project_config) = self.project_config {
            let companion_tools = project_config.get_companion_tools(runtime_name);
            debug!(
                "Companion tools for {}: {:?}",
                runtime_name,
                companion_tools
                    .iter()
                    .map(|(n, v)| format!("{}@{}", n, v))
                    .collect::<Vec<_>>()
            );
            for (companion_name, companion_version) in &companion_tools {
                let companion_runtime = match registry.get_runtime(companion_name) {
                    Some(r) => r,
                    None => {
                        debug!(
                            "  Companion {} not found in registry, skipping",
                            companion_name
                        );
                        continue;
                    }
                };

                // Check if the companion tool is installed. Environment
                // preparation is intentionally side-effect-free for companions:
                // missing or broken companions are skipped instead of
                // auto-installed/repaired here.
                let companion_installed_version = match companion_runtime
                    .installed_versions(context)
                    .await
                {
                    Ok(versions) if !versions.is_empty() => {
                        // Find the best matching version
                        let env_mgr_for_version = EnvironmentManager::new(
                            self.config,
                            self.resolver,
                            self.registry,
                            self.context,
                            self.project_config,
                        );
                        let matched_version = env_mgr_for_version
                            .find_matching_version(companion_name, companion_version, &versions)
                            .unwrap_or_else(|| versions[0].clone());

                        // Component requirements (for example MSVC Spectre) are
                        // honored when the companion is installed explicitly or
                        // invoked directly. Do not repair them from the
                        // unrelated command's environment preparation path.
                        let has_install_options = self
                            .project_config
                            .and_then(|pc| {
                                pc.get_install_options(companion_name)
                                    .map(|opts| !opts.is_empty())
                            })
                            .unwrap_or(false);

                        if has_install_options {
                            debug!(
                                "Companion {} has install options; deferring install/repair until it is invoked directly",
                                companion_name
                            );
                        }
                        matched_version
                    }
                    Ok(_) => {
                        debug!(
                            "Companion {} is not installed; skipping environment injection for primary {}",
                            companion_name, runtime_name
                        );
                        continue;
                    }
                    Err(e) => {
                        debug!(
                            "Could not inspect companion {} installation state: {}; skipping environment injection for primary {}",
                            companion_name, e, runtime_name
                        );
                        continue;
                    }
                };

                debug!(
                    "Injecting companion tool environment: {}@{} (for primary {})",
                    companion_name, companion_installed_version, runtime_name
                );

                // Call prepare_environment() (NOT execution_environment())
                // This gives us marker/discovery variables without full compilation env
                match companion_runtime
                    .prepare_environment(&companion_installed_version, context)
                    .await
                {
                    Ok(companion_env) => {
                        if !companion_env.is_empty() {
                            debug!(
                                "  Companion {} injected {} environment variables",
                                companion_name,
                                companion_env.len()
                            );
                            // Merge companion env, but don't override existing vars
                            // (primary runtime's vars take precedence)
                            for (key, value) in companion_env {
                                env.entry(key).or_insert(value);
                            }
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to prepare companion tool environment for {}: {}",
                            companion_name, e
                        );
                    }
                }
            }
        }

        if !env.is_empty() {
            debug!(
                "Prepared {} environment variables for {} {}",
                env.len(),
                runtime_name,
                version
            );
        }

        Ok(env)
    }

    /// Get the provider runtime for a bundled tool
    pub fn get_provider_runtime(
        &self,
        runtime_name: &str,
        version: Option<&str>,
    ) -> (Option<String>, Option<String>) {
        if let Some(spec) = self.resolver.get_spec(runtime_name) {
            for dep in &spec.dependencies {
                if dep.required
                    && let Some(ref provider) = dep.provided_by
                {
                    // For bundled tools (npx, npm), use the provider's version
                    return (Some(provider.clone()), version.map(|v| v.to_string()));
                }
            }
        }
        (None, None)
    }

    /// Expand template variables in environment values
    pub fn expand_template(
        &self,
        template: &str,
        runtime_name: &str,
        version: Option<&str>,
    ) -> Result<String> {
        let mut result = template.to_string();

        // Get runtime spec for template expansion
        if let Some(_spec) = self.resolver.get_spec(runtime_name) {
            // Replace {install_dir} using PathProvider
            if result.contains("{install_dir}") {
                let install_dir = self.resolve_install_dir(runtime_name, version);

                if let Some(dir) = install_dir {
                    result = result.replace("{install_dir}", &dir.to_string_lossy());
                    debug!("  expand_template: {{install_dir}} -> {}", dir.display());
                } else {
                    warn!(
                        "Could not expand {{install_dir}} for {}: no version available or path does not exist",
                        runtime_name
                    );
                }
            }

            // Replace {version}
            if let Some(ver) = version {
                result = result.replace("{version}", ver);
            }

            // Replace {executable} using PathProvider
            if result.contains("{executable}")
                && let (Some(ctx), Some(ver)) = (self.context, version)
            {
                let exe_path = ctx.paths.executable_path(runtime_name, ver);
                result = result.replace("{executable}", &exe_path.to_string_lossy());
            }

            // Replace {PATH} with current PATH
            if let Ok(path) = std::env::var("PATH") {
                result = result.replace("{PATH}", &path);
            }

            // Replace shell-style variables
            if result.contains("$HOME")
                && let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))
            {
                result = result.replace("$HOME", &home);
            }

            if result.contains("$CARGO_HOME")
                && let Ok(cargo_home) = std::env::var("CARGO_HOME")
            {
                result = result.replace("$CARGO_HOME", &cargo_home);
            }

            if result.contains("$RUSTUP_HOME")
                && let Ok(rustup_home) = std::env::var("RUSTUP_HOME")
            {
                result = result.replace("$RUSTUP_HOME", &rustup_home);
            }

            if result.contains("$USER")
                && let Ok(user) = std::env::var("USER").or_else(|_| std::env::var("USERNAME"))
            {
                result = result.replace("$USER", &user);
            }

            // Replace {env:VAR} with environment variable
            while let Some(start) = result.find("{env:") {
                if let Some(end) = result[start..].find('}') {
                    let env_var = &result[start + 5..start + end];
                    let env_value = std::env::var(env_var).unwrap_or_default();
                    result.replace_range(start..=start + end, &env_value);
                } else {
                    break;
                }
            }
        }

        Ok(result)
    }

    /// Resolve the installation directory for a runtime
    pub fn resolve_install_dir(
        &self,
        runtime_name: &str,
        version: Option<&str>,
    ) -> Option<PathBuf> {
        let ctx = self.context?;
        let platform = vx_paths::manager::CurrentPlatform::current();

        // If version is provided, use it directly
        if let Some(ver) = version {
            let version_dir = ctx.paths.version_store_dir(runtime_name, ver);
            let platform_dir = version_dir.join(platform.as_str());

            // New layout: try version_dir first (no platform subdirectory).
            // Old layout: fall back to platform_dir.
            if version_dir.exists() {
                return Some(version_dir);
            }

            // Fallback to old platform-specific directory
            if platform_dir.exists() {
                debug!(
                    "Using old platform-specific directory for {}: {}",
                    runtime_name,
                    platform_dir.display()
                );
                return Some(platform_dir);
            }

            debug!(
                "Version directory does not exist for {} v{}: {}",
                runtime_name,
                ver,
                version_dir.display()
            );
            return None;
        }

        // No version provided - scan installed versions and select the latest
        let runtime_store_dir = ctx.paths.runtime_store_dir(runtime_name);
        let entries = match std::fs::read_dir(&runtime_store_dir) {
            Ok(e) => e,
            Err(_) => {
                debug!(
                    "Cannot read runtime store directory: {}",
                    runtime_store_dir.display()
                );
                return None;
            }
        };

        // Collect valid version directories
        let mut versions: Vec<String> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();

        if versions.is_empty() {
            return None;
        }

        // Sort by semver and get latest
        versions.sort_by(|a, b| self.compare_versions(a, b));
        let latest = versions.last()?;

        let version_dir = ctx.paths.version_store_dir(runtime_name, latest);
        let platform_dir = version_dir.join(platform.as_str());

        // New layout: try version_dir first, then fallback to old platform_dir
        if version_dir.exists() {
            Some(version_dir)
        } else if platform_dir.exists() {
            Some(platform_dir)
        } else {
            None
        }
    }

    /// Build PATH string containing all vx-managed tool bin directories
    ///
    /// **Performance optimization**: Instead of calling `registry.supported_runtimes()`
    /// (which triggers `materialize_all()` and instantiates all ~45 providers), we
    /// directly scan `~/.vx/store/` to discover installed runtimes. This avoids
    /// provider materialization entirely, reducing prepare stage from ~400ms to <50ms.
    pub fn build_vx_tools_path(&self) -> Option<String> {
        let context = self.context?;

        let mut paths: Vec<String> = Vec::new();

        // Add vx bin directory first (for shims)
        let vx_bin = context.paths.bin_dir();
        if vx_bin.exists() {
            paths.push(vx_bin.to_string_lossy().to_string());
        }

        // Scan store directory directly to find installed runtimes.
        // This is much faster than materialize_all() + supported_runtimes() because
        // it only does a shallow read_dir of ~/.vx/store/ and doesn't instantiate
        // any provider factories.
        let store_dir = context.paths.store_dir();
        if !store_dir.exists() {
            return if paths.is_empty() {
                None
            } else {
                Some(vx_paths::join_paths_simple(&paths))
            };
        }

        let installed_runtimes: Vec<String> = match std::fs::read_dir(&store_dir) {
            Ok(entries) => entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect(),
            Err(_) => Vec::new(),
        };

        for runtime_name in &installed_runtimes {
            let runtime_store_dir = context.paths.runtime_store_dir(runtime_name);

            if let Ok(entries) = std::fs::read_dir(&runtime_store_dir) {
                let installed_versions: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect();

                let version_to_use =
                    self.select_version_for_runtime(runtime_name, &installed_versions);

                if let Some(version) = version_to_use {
                    let store_dir = context.paths.version_store_dir(runtime_name, &version);

                    if let Some(bin_dir) = bin_dir_cache::find_bin_dir(&store_dir, runtime_name)
                        && bin_dir.exists()
                    {
                        let bin_path = bin_dir.to_string_lossy().to_string();
                        if !paths.contains(&bin_path) {
                            paths.push(bin_path);
                        }
                    }
                }
            }
        }

        if paths.is_empty() {
            None
        } else {
            Some(vx_paths::join_paths_simple(&paths))
        }
    }

    /// Select the version to use for a runtime, prioritizing project configuration
    pub fn select_version_for_runtime(
        &self,
        runtime_name: &str,
        installed_versions: &[String],
    ) -> Option<String> {
        if installed_versions.is_empty() {
            return None;
        }

        // Check if project configuration specifies a version for this runtime
        if let Some(project_config) = self.project_config
            && let Some(requested_version) = project_config.get_version_with_fallback(runtime_name)
        {
            // "latest" means "use the latest installed version" — skip matching
            if requested_version == "latest" {
                let mut versions = installed_versions.to_vec();
                versions.sort_by(|a, b| self.compare_versions(a, b));
                if let Some(latest) = versions.last() {
                    trace!(
                        "Using {} version {} (latest installed) from vx.toml",
                        runtime_name, latest
                    );
                    return Some(latest.clone());
                }
            }

            if ProjectToolsConfig::is_toolchain_managed_runtime_version(
                runtime_name,
                requested_version,
            ) {
                let mut versions = installed_versions.to_vec();
                versions.sort_by(|a, b| self.compare_versions(a, b));
                if let Some(latest) = versions.last() {
                    trace!(
                        "Using {} store version {} for requested toolchain {} from vx.toml",
                        runtime_name, latest, requested_version
                    );
                    return Some(latest.clone());
                }
            }

            let matching_version =
                self.find_matching_version(runtime_name, requested_version, installed_versions);

            if let Some(version) = matching_version {
                trace!("Using {} version {} from vx.toml", runtime_name, version);
                return Some(version);
            } else {
                // Requested version not installed, warn and fall back to latest
                if bin_dir_cache::record_warned_tool(runtime_name) {
                    warn!(
                        "Version {} specified in vx.toml for {} is not installed. \
                             Using latest installed version instead. \
                             Run 'vx install {}@{}' to install the specified version.",
                        requested_version, runtime_name, runtime_name, requested_version
                    );
                }
            }
        }

        // Fall back to latest installed version
        let mut versions = installed_versions.to_vec();
        versions.sort_by(|a, b| self.compare_versions(a, b));

        versions.last().cloned()
    }

    /// Find a matching version from installed versions
    pub fn find_matching_version(
        &self,
        _runtime_name: &str,
        requested: &str,
        installed: &[String],
    ) -> Option<String> {
        version_utils::find_matching_version(requested, installed)
    }

    /// Compare two version strings
    pub fn compare_versions(&self, a: &str, b: &str) -> std::cmp::Ordering {
        version_utils::compare_versions(a, b)
    }
}
