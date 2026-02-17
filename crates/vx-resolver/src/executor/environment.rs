//! Environment preparation and PATH building
//!
//! This module handles:
//! - Preparing runtime environment variables
//! - Template expansion ({install_dir}, {version}, etc.)
//! - Building the vx tools PATH for subprocess execution

use crate::{Resolver, ResolverConfig, Result};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::{debug, info, trace, warn};
use vx_cache::BinDirCache;
use vx_runtime::{ProviderRegistry, RuntimeContext};

use super::project_config::ProjectToolsConfig;

/// Track which tools have been warned about missing versions
/// This prevents duplicate warnings when building PATH
pub(crate) static WARNED_TOOLS: Mutex<Option<HashSet<String>>> = Mutex::new(None);

/// Process-level bin directory cache, backed by disk persistence.
///
/// On first access, loads from `~/.vx/cache/bin-dirs.bin`. New entries are
/// accumulated in memory and flushed to disk via `save_bin_dir_cache()`.
/// This avoids the cold-start penalty of walkdir traversals when the
/// process-level cache would otherwise be empty.
static BIN_DIR_CACHE: Mutex<Option<BinDirCache>> = Mutex::new(None);

/// Initialize the bin directory cache from disk (if not already loaded).
///
/// Call this early in the execution pipeline to pre-warm the cache.
pub fn init_bin_dir_cache(cache_dir: &std::path::Path) {
    let mut cache = BIN_DIR_CACHE.lock().unwrap();
    if cache.is_none() {
        *cache = Some(BinDirCache::load(cache_dir));
    }
}

/// Save the bin directory cache to disk.
///
/// Call this after command execution to persist newly discovered entries.
pub fn save_bin_dir_cache(cache_dir: &std::path::Path) {
    let cache = BIN_DIR_CACHE.lock().unwrap();
    if let Some(ref c) = *cache
        && let Err(e) = c.save(cache_dir)
    {
        tracing::debug!("Failed to save bin dir cache: {}", e);
    }
}

/// Invalidate the bin directory cache for a specific runtime.
///
/// Call this after installing or uninstalling a runtime.
pub fn invalidate_bin_dir_cache(runtime_store_prefix: &str) {
    let mut cache = BIN_DIR_CACHE.lock().unwrap();
    if let Some(ref mut c) = *cache {
        c.invalidate_runtime(runtime_store_prefix);
    }
}

/// Clear the entire bin directory cache.
pub fn clear_bin_dir_cache() {
    let mut cache = BIN_DIR_CACHE.lock().unwrap();
    *cache = None;
}

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

        // Inject companion tools' prepare_environment() from vx.toml
        //
        // When vx.toml specifies tools like [tools.msvc], ALL other tool executions
        // (vx node, vx cmake, vx cargo, vx dotnet, etc.) will have MSVC's marker
        // environment variables (VCINSTALLDIR, VCToolsInstallDir, VX_MSVC_*, etc.)
        // injected. This enables any tool that needs a C/C++ compiler to discover
        // the vx-managed MSVC installation — including node-gyp, cmake, meson,
        // cargo (for C dependencies via cc crate), dotnet native AOT, etc.
        //
        // This only calls prepare_environment() (not execution_environment()), so it
        // injects discovery/marker variables without polluting LIB/INCLUDE/PATH.
        //
        // See: https://github.com/loonghao/vx/issues/573
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

                // Check if the companion tool is installed
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

                        // Check if the companion tool has component requirements (e.g., MSVC Spectre)
                        // that might be missing from the existing installation.
                        // If so, go through ensure_version_installed to trigger component installation.
                        let has_components = self
                            .project_config
                            .and_then(|pc| {
                                pc.get_install_options(companion_name)
                                    .map(|opts| opts.contains_key("VX_MSVC_COMPONENTS"))
                            })
                            .unwrap_or(false);

                        if has_components {
                            // Fast-path: check if component installation was already attempted.
                            // If so, skip the expensive ensure_version_installed call entirely.
                            // The marker file is written by MsvcRuntime::install() after the
                            // first installation attempt, preventing repeated downloads/extractions.
                            let component_already_attempted = self
                                .context
                                .map(|ctx| {
                                    let store_dir = ctx
                                        .paths
                                        .version_store_dir(companion_name, &matched_version);
                                    store_dir.join(".component-install-attempted").exists()
                                })
                                .unwrap_or(false);

                            if component_already_attempted {
                                debug!(
                                    "Companion {} component installation already attempted, skipping integrity check",
                                    companion_name
                                );
                                matched_version
                            } else {
                                debug!(
                                    "Companion {} has component requirements, verifying installation integrity...",
                                    companion_name
                                );
                                let mut install_mgr = super::installation::InstallationManager::new(
                                    self.config,
                                    self.resolver,
                                    self.registry,
                                    self.context,
                                );
                                if let Some(project_config) = self.project_config {
                                    install_mgr = install_mgr.with_project_config(project_config);
                                }
                                match install_mgr
                                    .ensure_version_installed(companion_name, &matched_version)
                                    .await
                                {
                                    Ok(Some(result)) => {
                                        debug!(
                                            "Companion {} component check complete (version: {})",
                                            companion_name, result.version
                                        );
                                        result.version
                                    }
                                    Ok(None) => matched_version,
                                    Err(e) => {
                                        warn!(
                                            "Failed to verify companion {} components: {}",
                                            companion_name, e
                                        );
                                        matched_version
                                    }
                                }
                            }
                        } else {
                            matched_version
                        }
                    }
                    Ok(_) | Err(_) => {
                        // Companion not installed — try auto-install
                        info!(
                            "Companion {} is not installed. Auto-installing {}@{} ...",
                            companion_name, companion_name, companion_version
                        );
                        let mut install_mgr = super::installation::InstallationManager::new(
                            self.config,
                            self.resolver,
                            self.registry,
                            self.context,
                        );
                        // Pass project_config so install_options (e.g., MSVC components)
                        // from vx.toml are injected into RuntimeContext during installation
                        if let Some(project_config) = self.project_config {
                            install_mgr = install_mgr.with_project_config(project_config);
                        }
                        // Use ensure_version_installed which goes through the unified
                        // version resolution path (runtime.resolve_version()), checks
                        // if already installed, and returns a consistent version string.
                        match install_mgr
                            .ensure_version_installed(companion_name, companion_version)
                            .await
                        {
                            Ok(Some(result)) => {
                                info!(
                                    "Auto-installed companion {}@{}",
                                    companion_name, result.version
                                );
                                result.version
                            }
                            Ok(None) => {
                                debug!(
                                    "  Companion {} could not be installed (no registry/context), skipping",
                                    companion_name
                                );
                                continue;
                            }
                            Err(e) => {
                                warn!("Failed to auto-install companion {}: {}", companion_name, e);
                                continue;
                            }
                        }
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

            // First try platform-specific directory
            if platform_dir.exists() {
                return Some(platform_dir);
            }

            // Fallback to version directory without platform
            if version_dir.exists() {
                debug!(
                    "Using version directory without platform suffix for {}: {}",
                    runtime_name,
                    version_dir.display()
                );
                return Some(version_dir);
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

        if platform_dir.exists() {
            Some(platform_dir)
        } else if version_dir.exists() {
            Some(version_dir)
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

                    if let Some(bin_dir) = self.find_bin_dir(&store_dir, runtime_name)
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

            let matching_version =
                self.find_matching_version(runtime_name, requested_version, installed_versions);

            if let Some(version) = matching_version {
                trace!("Using {} version {} from vx.toml", runtime_name, version);
                return Some(version);
            } else {
                // Requested version not installed, warn and fall back to latest
                let mut warned = WARNED_TOOLS.lock().unwrap();
                let warned_set = warned.get_or_insert_with(HashSet::new);
                if warned_set.insert(runtime_name.to_string()) {
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
        // First try exact match
        if installed.contains(&requested.to_string()) {
            return Some(requested.to_string());
        }

        // Try prefix match for partial versions
        let mut matches: Vec<&String> = installed
            .iter()
            .filter(|v| {
                v.starts_with(requested)
                    && (v.len() == requested.len() || v.chars().nth(requested.len()) == Some('.'))
            })
            .collect();

        if matches.is_empty() {
            return None;
        }

        // Sort and return the latest matching version
        matches.sort_by(|a, b| self.compare_versions(a, b));
        matches.last().map(|s| (*s).clone())
    }

    /// Compare two version strings
    pub fn compare_versions(&self, a: &str, b: &str) -> std::cmp::Ordering {
        let a_clean = a.trim_start_matches('v');
        let b_clean = b.trim_start_matches('v');

        let a_parts: Vec<u64> = a_clean
            .split('.')
            .filter_map(|s| s.split('-').next())
            .filter_map(|s| s.parse().ok())
            .collect();
        let b_parts: Vec<u64> = b_clean
            .split('.')
            .filter_map(|s| s.split('-').next())
            .filter_map(|s| s.parse().ok())
            .collect();

        for (ap, bp) in a_parts.iter().zip(b_parts.iter()) {
            match ap.cmp(bp) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            }
        }

        a_parts.len().cmp(&b_parts.len())
    }

    /// Find the bin directory for a runtime
    ///
    /// This method searches for the executable within the store directory
    /// structure and returns the directory containing it. This handles various archive
    /// structures like:
    /// - `<version>/<platform>/bin/node.exe` (standard)
    /// - `<version>/<platform>/node-v25.6.0-win-x64/node.exe` (Node.js style)
    /// - `<version>/<platform>/yarn-v1.22.19/bin/yarn.cmd` (Yarn style)
    ///
    /// Results are cached in a process-level `BIN_DIR_CACHE` to avoid repeated
    /// walkdir traversals on subsequent calls.
    ///
    /// ## Performance optimization
    ///
    /// Uses a two-phase search:
    /// 1. **Quick check**: Common locations (root, bin/, one-level subdirs)
    /// 2. **Walkdir fallback**: Only if quick check fails, with directory filtering
    pub fn find_bin_dir(&self, store_dir: &std::path::Path, runtime_name: &str) -> Option<PathBuf> {
        let cache_key = store_dir.to_string_lossy().to_string();

        // Check process-level cache first (backed by disk persistence)
        {
            let mut cache = BIN_DIR_CACHE.lock().unwrap();
            if let Some(ref mut c) = *cache
                && let Some(cached) = c.get(&cache_key)
            {
                trace!(
                    "BIN_DIR_CACHE hit for {} in {}",
                    runtime_name,
                    store_dir.display()
                );
                return Some(cached);
            }
        }

        let platform = vx_paths::manager::CurrentPlatform::current();

        // Build the list of possible executable names
        let exe_names: Vec<String> = if cfg!(windows) {
            vec![
                format!("{}.exe", runtime_name),
                format!("{}.cmd", runtime_name),
                runtime_name.to_string(),
            ]
        } else {
            vec![runtime_name.to_string()]
        };

        // Platform-specific directory
        let platform_dir = store_dir.join(platform.as_str());
        let search_dir = if platform_dir.exists() {
            &platform_dir
        } else if store_dir.exists() {
            store_dir
        } else {
            return None;
        };

        // Phase 1: Quick check common locations first (avoids walkdir entirely
        // for most runtimes like uv, go, bun, pnpm where exe is in root or bin/)
        if let Some(result) = self.quick_find_bin_dir(search_dir, &exe_names) {
            let mut cache = BIN_DIR_CACHE.lock().unwrap();
            let c = cache.get_or_insert_with(BinDirCache::new);
            c.put(cache_key, result.clone());
            return Some(result);
        }

        // Phase 2: Walkdir with directory filtering
        for entry in walkdir::WalkDir::new(search_dir)
            .max_depth(5)
            .into_iter()
            .filter_entry(|e| {
                // Skip known non-target directories
                if e.file_type().is_dir()
                    && let Some(name) = e.file_name().to_str()
                {
                    return !matches!(
                        name,
                        "node_modules"
                            | ".git"
                            | "__pycache__"
                            | "site-packages"
                            | "lib"
                            | "share"
                            | "include"
                            | "man"
                            | "doc"
                            | "docs"
                    );
                }
                true
            })
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file()
                && let Some(name) = path.file_name().and_then(|n| n.to_str())
                && exe_names.iter().any(|exe_name| name == exe_name)
                && let Some(parent) = path.parent()
            {
                trace!(
                    "Found executable for {} at {}, using bin dir: {}",
                    runtime_name,
                    path.display(),
                    parent.display()
                );
                let result = parent.to_path_buf();
                let mut cache = BIN_DIR_CACHE.lock().unwrap();
                let c = cache.get_or_insert_with(BinDirCache::new);
                c.put(cache_key, result.clone());
                return Some(result);
            }
        }

        // Fallback: check standard locations
        let bin_dir = platform_dir.join("bin");
        if bin_dir.exists() {
            let mut cache = BIN_DIR_CACHE.lock().unwrap();
            let c = cache.get_or_insert_with(BinDirCache::new);
            c.put(cache_key, bin_dir.clone());
            return Some(bin_dir);
        }

        let bin_dir = store_dir.join("bin");
        if bin_dir.exists() {
            let mut cache = BIN_DIR_CACHE.lock().unwrap();
            let c = cache.get_or_insert_with(BinDirCache::new);
            c.put(cache_key, bin_dir.clone());
            return Some(bin_dir);
        }

        // Last resort: return platform dir if it exists
        if platform_dir.exists() {
            let mut cache = BIN_DIR_CACHE.lock().unwrap();
            let c = cache.get_or_insert_with(BinDirCache::new);
            c.put(cache_key, platform_dir.clone());
            return Some(platform_dir);
        }

        None
    }

    /// Quick check common locations for bin directory (avoids walkdir).
    ///
    /// Most runtimes follow predictable patterns:
    /// - Root dir contains executable (uv, pnpm, kubectl)
    /// - bin/ subdirectory (go, java)
    /// - Single subdirectory contains executable (node-v25.6.0-win-x64/node.exe)
    /// - Single subdirectory + bin/ (yarn-v1.22.19/bin/yarn)
    fn quick_find_bin_dir(
        &self,
        search_dir: &std::path::Path,
        exe_names: &[String],
    ) -> Option<PathBuf> {
        // Check root directory
        for name in exe_names {
            if search_dir.join(name).is_file() {
                return Some(search_dir.to_path_buf());
            }
        }

        // Check bin/ subdirectory
        let bin_dir = search_dir.join("bin");
        if bin_dir.is_dir() {
            for name in exe_names {
                if bin_dir.join(name).is_file() {
                    return Some(bin_dir);
                }
            }
        }

        // Check one level of subdirectories
        if let Ok(entries) = std::fs::read_dir(search_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let sub_path = entry.path();
                if !sub_path.is_dir() {
                    continue;
                }
                if let Some(dir_name) = sub_path.file_name().and_then(|n| n.to_str())
                    && matches!(
                        dir_name,
                        "bin" | "node_modules" | "lib" | "share" | "include" | "man" | "doc"
                    )
                {
                    continue;
                }
                // Check files in subdirectory
                for name in exe_names {
                    if sub_path.join(name).is_file() {
                        return Some(sub_path);
                    }
                }
                // Check subdirectory/bin/
                let sub_bin = sub_path.join("bin");
                if sub_bin.is_dir() {
                    for name in exe_names {
                        if sub_bin.join(name).is_file() {
                            return Some(sub_bin);
                        }
                    }
                }
            }
        }

        None
    }
}
