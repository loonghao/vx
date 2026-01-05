//! Executor - the core command forwarding engine
//!
//! This module implements the main execution logic:
//! 1. Resolve runtime and dependencies
//! 2. Auto-install missing components
//! 3. Forward command to the appropriate executable

use crate::resolution_cache::log_cache_result;
use crate::{ResolutionCache, ResolutionCacheKey, ResolvedGraph, Resolver, ResolverConfig, Result};
use std::process::{ExitStatus, Stdio};
use tokio::process::Command;
use tracing::{debug, info, warn};
use vx_runtime::{CacheMode, ProviderRegistry, RuntimeContext};

/// Executor for runtime command forwarding
pub struct Executor<'a> {
    /// Configuration
    config: ResolverConfig,

    /// Runtime resolver
    resolver: Resolver,

    /// Optional disk-backed resolution cache
    resolution_cache: Option<ResolutionCache>,

    /// Optional provider registry for installation
    registry: Option<&'a ProviderRegistry>,

    /// Runtime context for installation
    context: Option<&'a RuntimeContext>,
}

impl<'a> Executor<'a> {
    /// Create a new executor
    pub fn new(config: ResolverConfig) -> Result<Self> {
        let resolver = Resolver::new(config.clone())?;
        let resolution_cache = ResolutionCache::default_paths(&config)
            .map_err(|e| {
                debug!("Resolution cache disabled: {}", e);
                e
            })
            .ok();
        Ok(Self {
            config,
            resolver,
            resolution_cache,
            registry: None,
            context: None,
        })
    }

    /// Create an executor with a provider registry for auto-installation
    pub fn with_registry(config: ResolverConfig, registry: &'a ProviderRegistry) -> Result<Self> {
        let resolver = Resolver::new(config.clone())?;
        let resolution_cache = ResolutionCache::default_paths(&config)
            .map_err(|e| {
                debug!("Resolution cache disabled: {}", e);
                e
            })
            .ok();
        Ok(Self {
            config,
            resolver,
            resolution_cache,
            registry: Some(registry),
            context: None,
        })
    }

    /// Create an executor with a provider registry and runtime context
    pub fn with_registry_and_context(
        config: ResolverConfig,
        registry: &'a ProviderRegistry,
        context: &'a RuntimeContext,
    ) -> Result<Self> {
        let resolver = Resolver::new(config.clone())?;
        let resolution_cache = ResolutionCache::default_paths(&config)
            .map_err(|e| {
                debug!("Resolution cache disabled: {}", e);
                e
            })
            .ok();
        Ok(Self {
            config,
            resolver,
            resolution_cache,
            registry: Some(registry),
            context: Some(context),
        })
    }

    /// Set the runtime context
    pub fn set_context(&mut self, context: &'a RuntimeContext) {
        self.context = Some(context);
    }

    /// Execute a runtime with the given arguments
    ///
    /// This is the main entry point for command forwarding.
    /// Format: vx <runtime> <args...>
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use vx_resolver::{Executor, ResolverConfig};
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///     let executor = Executor::new(ResolverConfig::default())?;
    ///
    ///     // Execute: npm install express
    ///     let exit_code = executor.execute("npm", &["install".into(), "express".into()]).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn execute(&self, runtime_name: &str, args: &[String]) -> Result<i32> {
        self.execute_with_version(runtime_name, None, args).await
    }

    /// Execute a runtime with the given arguments and optional version constraint
    ///
    /// This supports the `runtime@version` syntax:
    /// - `vx yarn@1.21.1 global add terminalizer`
    /// - `vx node@20 --version`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use vx_resolver::{Executor, ResolverConfig};
    ///
    /// async fn example() -> anyhow::Result<()> {
    ///     let executor = Executor::new(ResolverConfig::default())?;
    ///
    ///     // Execute: yarn@1.21.1 global add terminalizer
    ///     let exit_code = executor.execute_with_version(
    ///         "yarn",
    ///         Some("1.21.1"),
    ///         &["global".into(), "add".into(), "terminalizer".into()]
    ///     ).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn execute_with_version(
        &self,
        runtime_name: &str,
        version: Option<&str>,
        args: &[String],
    ) -> Result<i32> {
        if let Some(ver) = version {
            debug!("Executing: {}@{} {}", runtime_name, ver, args.join(" "));
        } else {
            debug!("Executing: {} {}", runtime_name, args.join(" "));
        }

        // Check platform support before any operation
        if let Some(registry) = self.registry {
            if let Some(runtime) = registry.get_runtime(runtime_name) {
                if let Err(e) = runtime.check_platform_support() {
                    return Err(anyhow::anyhow!("{}", e));
                }
            }
        }

        // -------------------------
        // Resolve
        // -------------------------
        let resolve_stage = |runtime_name: &str,
                             version: Option<&str>,
                             args: &[String]|
         -> Result<(ResolutionCacheKey, ResolvedGraph)> {
            let key = ResolutionCacheKey::from_context(runtime_name, version, args, &self.config);

            let graph = if let Some(cache) = &self.resolution_cache {
                let cached = cache.get(&key);
                log_cache_result(cached.is_some(), runtime_name);

                if cached.is_none() && cache.mode() == CacheMode::Offline {
                    return Err(anyhow::anyhow!(
                        "Offline mode: no cached resolution available for {}. Run without offline mode to resolve.",
                        runtime_name
                    ));
                }

                if let Some(g) = cached {
                    g
                } else {
                    let g = self
                        .resolver
                        .resolve_graph_with_version(runtime_name, version)?;
                    let _ = cache.set(&key, &g);
                    g
                }
            } else {
                self.resolver
                    .resolve_graph_with_version(runtime_name, version)?
            };

            Ok((key, graph))
        };

        let (mut cache_key, mut graph) = resolve_stage(runtime_name, version, args)?;
        let mut resolution: crate::ResolutionResult = graph.clone().into();

        // -------------------------
        // Ensure Installed
        // -------------------------
        // Track the installed version for environment preparation
        let mut installed_version: Option<String> = None;

        // Track dependency environment overrides (for version-constrained dependencies)
        let mut dependency_env_overrides: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        // If a specific version is requested, ensure it's installed first, then resolve again
        if let Some(requested_version) = version {
            installed_version = self
                .ensure_version_installed(runtime_name, requested_version)
                .await?;

            (cache_key, graph) = resolve_stage(runtime_name, installed_version.as_deref(), args)?;
            resolution = graph.clone().into();
        }

        // Install missing runtimes/dependencies (if any)
        if !resolution.install_order.is_empty() {
            if self.config.auto_install {
                info!(
                    "Auto-installing missing runtimes: {:?}",
                    resolution.install_order
                );

                // Best-effort: keep the last installed version for env preparation.
                installed_version = self.install_runtimes(&resolution.install_order).await?;

                // Re-resolve after installation
                (cache_key, graph) = resolve_stage(runtime_name, version, args)?;
                resolution = graph.clone().into();

                debug!(
                    "Re-resolved after installation: executable={}",
                    resolution.executable.display()
                );
            } else {
                // Report missing dependencies
                let missing = if resolution.runtime_needs_install {
                    format!(
                        "Runtime '{}' is not installed. Missing dependencies: {:?}",
                        runtime_name, resolution.missing_dependencies
                    )
                } else {
                    format!(
                        "Missing dependencies for '{}': {:?}",
                        runtime_name, resolution.missing_dependencies
                    )
                };

                return Err(anyhow::anyhow!(
                    "{}. Run 'vx install {}' or enable auto-install.",
                    missing,
                    runtime_name
                ));
            }
        }

        // Handle incompatible dependencies - find or install compatible versions
        if !resolution.incompatible_dependencies.is_empty() {
            for incompatible in &resolution.incompatible_dependencies {
                warn!(
                    "Dependency {} version {:?} is incompatible with {} (requires: min={:?}, max={:?})",
                    incompatible.runtime_name,
                    incompatible.current_version,
                    runtime_name,
                    incompatible.constraint.min_version,
                    incompatible.constraint.max_version
                );

                // Try to find or install a compatible version
                if let Some(compatible_env) = self
                    .ensure_compatible_dependency(
                        &incompatible.runtime_name,
                        &incompatible.constraint,
                    )
                    .await?
                {
                    // Merge the environment overrides
                    dependency_env_overrides.extend(compatible_env);
                }
            }
        }

        // -------------------------
        // Plan
        // -------------------------
        // Prepare environment variables for the runtime
        let mut runtime_env = self
            .prepare_runtime_environment(runtime_name, installed_version.as_deref())
            .await?;

        // Apply dependency environment overrides (these take precedence)
        runtime_env.extend(dependency_env_overrides);

        // -------------------------
        // Execute
        // -------------------------
        // Call pre_run hook if provider is available
        if let Some(registry) = self.registry {
            if let Some(runtime) = registry.get_runtime(runtime_name) {
                let should_continue = runtime.pre_run(args, &resolution.executable).await?;
                if !should_continue {
                    debug!("pre_run returned false, skipping execution");
                    return Ok(0);
                }
            }
        }

        // Build the command with environment variables
        let mut cmd = self.build_command(&resolution, args, &runtime_env)?;

        // Execute
        let status = self.run_command(&mut cmd).await?;

        // Keep cache_key used to silence unused warning in some builds.
        let _ = cache_key;

        Ok(status.code().unwrap_or(1))
    }

    /// Ensure a compatible version of a dependency is available
    ///
    /// Returns environment variables to override PATH to use the compatible version
    async fn ensure_compatible_dependency(
        &self,
        dep_name: &str,
        constraint: &crate::RuntimeDependency,
    ) -> Result<Option<std::collections::HashMap<String, String>>> {
        let (registry, context) = match (self.registry, self.context) {
            (Some(r), Some(c)) => (r, c),
            _ => {
                warn!(
                    "Cannot ensure compatible {} version: no registry or context available",
                    dep_name
                );
                return Ok(None);
            }
        };

        let runtime = match registry.get_runtime(dep_name) {
            Some(r) => r,
            None => {
                warn!("Unknown runtime: {}", dep_name);
                return Ok(None);
            }
        };

        // First, try to find an already installed compatible version
        let installed_versions = runtime
            .installed_versions(context)
            .await
            .unwrap_or_default();
        debug!(
            "Checking {} installed versions of {} for compatibility",
            installed_versions.len(),
            dep_name
        );

        for version in &installed_versions {
            if constraint.is_version_compatible(version) {
                info!(
                    "Found compatible {} version {} already installed",
                    dep_name, version
                );
                return self.prepare_dependency_env(dep_name, version).await;
            }
        }

        // No compatible version installed, need to install one
        if !self.config.auto_install {
            return Err(anyhow::anyhow!(
                "No compatible {} version found. Required: min={:?}, max={:?}. \
                 Run 'vx install {}@{}' or enable auto-install.",
                dep_name,
                constraint.min_version,
                constraint.max_version,
                dep_name,
                constraint
                    .recommended_version
                    .as_deref()
                    .unwrap_or("compatible-version")
            ));
        }

        // Determine which version to install
        let version_to_install = if let Some(ref recommended) = constraint.recommended_version {
            // Use recommended version
            info!(
                "Installing recommended {} version {} for compatibility",
                dep_name, recommended
            );
            recommended.clone()
        } else if let Some(ref max) = constraint.max_version {
            // Use max version as a hint (install the highest compatible version)
            info!(
                "Installing {} version <= {} for compatibility",
                dep_name, max
            );
            max.clone()
        } else if let Some(ref min) = constraint.min_version {
            // Use min version
            info!(
                "Installing {} version >= {} for compatibility",
                dep_name, min
            );
            min.clone()
        } else {
            // No constraints, use latest
            return Ok(None);
        };

        // Resolve and install the version
        let resolved_version = runtime
            .resolve_version(&version_to_install, context)
            .await?;
        info!(
            "Resolved {}@{} → {}",
            dep_name, version_to_install, resolved_version
        );

        // Check if this resolved version is compatible
        if !constraint.is_version_compatible(&resolved_version) {
            return Err(anyhow::anyhow!(
                "Resolved {} version {} does not meet constraints (min={:?}, max={:?}). \
                 Please install a compatible version manually.",
                dep_name,
                resolved_version,
                constraint.min_version,
                constraint.max_version
            ));
        }

        // Install if not already installed
        if !runtime.is_installed(&resolved_version, context).await? {
            info!(
                "Installing {} {} for compatibility",
                dep_name, resolved_version
            );
            runtime.pre_install(&resolved_version, context).await?;
            let result = runtime.install(&resolved_version, context).await?;
            if !context.fs.exists(&result.executable_path) {
                return Err(anyhow::anyhow!(
                    "Installation completed but executable not found at {}",
                    result.executable_path.display()
                ));
            }
            runtime.post_install(&resolved_version, context).await?;
        }

        self.prepare_dependency_env(dep_name, &resolved_version)
            .await
    }

    /// Prepare environment variables to use a specific version of a dependency
    async fn prepare_dependency_env(
        &self,
        dep_name: &str,
        version: &str,
    ) -> Result<Option<std::collections::HashMap<String, String>>> {
        let (registry, context) = match (self.registry, self.context) {
            (Some(r), Some(c)) => (r, c),
            _ => return Ok(None),
        };

        let runtime = match registry.get_runtime(dep_name) {
            Some(r) => r,
            None => return Ok(None),
        };

        // Get environment from the runtime's prepare_environment
        let mut env = runtime
            .prepare_environment(version, context)
            .await
            .unwrap_or_default();

        // Get the bin directory for this version and prepend to PATH
        let store_dir = context.paths.version_store_dir(dep_name, version);
        let bin_dir = self.find_bin_dir(&store_dir, dep_name);

        if let Some(bin) = bin_dir {
            // Prepend the bin directory to PATH
            let current_path = std::env::var("PATH").unwrap_or_default();
            let new_path = if cfg!(windows) {
                format!("{};{}", bin.display(), current_path)
            } else {
                format!("{}:{}", bin.display(), current_path)
            };
            env.insert("PATH".to_string(), new_path);
            info!(
                "Using {} {} from {} (prepended to PATH)",
                dep_name,
                version,
                bin.display()
            );
        }

        if env.is_empty() {
            Ok(None)
        } else {
            Ok(Some(env))
        }
    }

    /// Find the bin directory within a tool's installation directory
    fn find_bin_dir(
        &self,
        store_dir: &std::path::Path,
        tool_name: &str,
    ) -> Option<std::path::PathBuf> {
        if !store_dir.exists() {
            return None;
        }

        // Common bin directory patterns
        let patterns = [
            store_dir.join("bin"),
            store_dir.to_path_buf(),
            store_dir.join(tool_name).join("bin"),
        ];

        for pattern in &patterns {
            if pattern.exists() && pattern.is_dir() {
                // Check if this directory contains executables
                let exe_name = if cfg!(windows) {
                    format!("{}.exe", tool_name)
                } else {
                    tool_name.to_string()
                };
                if pattern.join(&exe_name).exists() {
                    return Some(pattern.clone());
                }
            }
        }

        // Fallback: just use the store dir if it exists
        if store_dir.exists() {
            Some(store_dir.to_path_buf())
        } else {
            None
        }
    }

    /// Ensure a specific version of a runtime is installed
    ///
    /// Returns the resolved version string
    async fn ensure_version_installed(
        &self,
        runtime_name: &str,
        requested_version: &str,
    ) -> Result<Option<String>> {
        let (registry, context) = match (self.registry, self.context) {
            (Some(r), Some(c)) => (r, c),
            _ => {
                return Err(anyhow::anyhow!(
                    "Cannot install {}@{}: no registry or context available",
                    runtime_name,
                    requested_version
                ))
            }
        };

        let runtime = registry
            .get_runtime(runtime_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown runtime: {}", runtime_name))?;

        // Resolve the version constraint to an actual version
        let resolved_version = runtime.resolve_version(requested_version, context).await?;
        info!(
            "Resolved {}@{} → {}",
            runtime_name, requested_version, resolved_version
        );

        // Check if this version is already installed
        if runtime.is_installed(&resolved_version, context).await? {
            debug!("{} {} is already installed", runtime_name, resolved_version);
            return Ok(Some(resolved_version));
        }

        // Install the specific version
        if !self.config.auto_install {
            return Err(anyhow::anyhow!(
                "{}@{} is not installed. Run 'vx install {}@{}' or enable auto-install.",
                runtime_name,
                resolved_version,
                runtime_name,
                requested_version
            ));
        }

        info!(
            "Auto-installing {} {} (requested: {})",
            runtime_name, resolved_version, requested_version
        );

        // Check for and install dependencies first
        self.install_dependencies_for_version(runtime_name, &resolved_version)
            .await?;

        // Run pre-install hook
        runtime.pre_install(&resolved_version, context).await?;

        // Install the runtime
        let result = runtime.install(&resolved_version, context).await?;

        // Verify the installation
        if !context.fs.exists(&result.executable_path) {
            return Err(anyhow::anyhow!(
                "Installation completed but executable not found at {}",
                result.executable_path.display()
            ));
        }

        // Run post-install hook
        runtime.post_install(&resolved_version, context).await?;

        info!(
            "Successfully installed {} {}",
            runtime_name, resolved_version
        );
        Ok(Some(resolved_version))
    }

    /// Install dependencies for a specific version of a runtime
    async fn install_dependencies_for_version(
        &self,
        runtime_name: &str,
        _version: &str,
    ) -> Result<()> {
        // Get the runtime spec to check dependencies
        let spec = self.resolver.get_spec(runtime_name);

        if let Some(spec) = spec {
            for dep in &spec.dependencies {
                if !dep.required {
                    continue;
                }

                // Get the provider name (the actual runtime to install)
                let dep_runtime = dep.provided_by.as_deref().unwrap_or(&dep.runtime_name);

                // Check if the dependency is installed
                let resolution = self.resolver.resolve(dep_runtime);
                if resolution.is_err() || resolution.as_ref().unwrap().runtime_needs_install {
                    // Install the dependency
                    // TODO: Support version constraints from RuntimeDependency.min_version
                    info!(
                        "Installing dependency {} for {} ({})",
                        dep_runtime, runtime_name, dep.reason
                    );
                    self.install_runtime(dep_runtime).await?;
                }
            }
        }

        Ok(())
    }

    /// Prepare environment variables for a runtime
    ///
    /// This calls the runtime's `prepare_environment` method to get any
    /// additional environment variables needed for execution.
    async fn prepare_runtime_environment(
        &self,
        runtime_name: &str,
        version: Option<&str>,
    ) -> Result<std::collections::HashMap<String, String>> {
        use std::collections::HashMap;

        // If we don't have registry and context, return empty environment
        let (registry, context) = match (self.registry, self.context) {
            (Some(r), Some(c)) => (r, c),
            _ => return Ok(HashMap::new()),
        };

        // Get the runtime
        let runtime = match registry.get_runtime(runtime_name) {
            Some(r) => r,
            None => return Ok(HashMap::new()),
        };

        // Determine the version to use
        let version = match version {
            Some(v) => v.to_string(),
            None => {
                // Try to get the installed version from the store
                match runtime.installed_versions(context).await {
                    Ok(versions) if !versions.is_empty() => versions[0].clone(),
                    _ => return Ok(HashMap::new()),
                }
            }
        };

        // Call prepare_environment
        match runtime.prepare_environment(&version, context).await {
            Ok(env) => {
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
            Err(e) => {
                warn!(
                    "Failed to prepare environment for {} {}: {}",
                    runtime_name, version, e
                );
                Ok(HashMap::new())
            }
        }
    }

    /// Install a list of runtimes in order
    ///
    /// Returns the version of the last installed runtime (typically the primary runtime)
    async fn install_runtimes(&self, runtimes: &[String]) -> Result<Option<String>> {
        let mut last_version = None;
        for runtime in runtimes {
            last_version = self.install_runtime(runtime).await?;
        }
        Ok(last_version)
    }

    /// Install a single runtime
    ///
    /// Returns the installed version if successful
    async fn install_runtime(&self, runtime_name: &str) -> Result<Option<String>> {
        info!("Installing: {}", runtime_name);

        // Try using the provider registry first
        if let (Some(registry), Some(context)) = (self.registry, self.context) {
            if let Some(runtime) = registry.get_runtime(runtime_name) {
                // Check platform support before attempting installation
                if let Err(e) = runtime.check_platform_support() {
                    return Err(anyhow::anyhow!("{}", e));
                }

                // Fetch versions to get the latest
                debug!("Fetching versions for {}", runtime_name);
                let versions = runtime.fetch_versions(context).await?;
                let version = versions
                    .iter()
                    .find(|v| !v.prerelease)
                    .map(|v| v.version.clone())
                    .or_else(|| versions.first().map(|v| v.version.clone()))
                    .ok_or_else(|| anyhow::anyhow!("No versions found for {}", runtime_name))?;

                info!("Installing {} {} via provider", runtime_name, version);

                // Run pre-install hook
                runtime.pre_install(&version, context).await?;

                // Actually install the runtime
                // Note: Runtime::install() calls post_extract() internally before verification,
                // which handles file renaming (e.g., pnpm-macos-arm64 -> pnpm)
                debug!("Calling runtime.install() for {} {}", runtime_name, version);
                let result = runtime.install(&version, context).await?;
                debug!(
                    "Install result: path={}, exe={}, already_installed={}",
                    result.install_path.display(),
                    result.executable_path.display(),
                    result.already_installed
                );

                // Verify the installation actually succeeded
                if !context.fs.exists(&result.executable_path) {
                    return Err(anyhow::anyhow!(
                        "Installation completed but executable not found at {}",
                        result.executable_path.display()
                    ));
                }

                // Run post-install hook (for symlinks, PATH setup, etc.)
                runtime.post_install(&version, context).await?;

                info!("Successfully installed {} {}", runtime_name, version);
                return Ok(Some(version));
            }
        }

        // Fallback: try to install using known methods
        self.install_runtime_fallback(runtime_name).await?;
        Ok(None)
    }

    /// Fallback installation methods for runtimes
    async fn install_runtime_fallback(&self, runtime_name: &str) -> Result<()> {
        let spec = self.resolver.get_spec(runtime_name);

        match runtime_name {
            // Node.js ecosystem
            "node" | "nodejs" => {
                self.install_via_command("node", &["--version"])
                    .await
                    .map_err(|_| {
                        anyhow::anyhow!(
                        "Node.js is not installed. Please install it from https://nodejs.org/ or run 'vx install node'"
                    )
                    })?;
            }

            "npm" | "npx" => {
                // npm/npx come with Node.js
                warn!("{} should be installed with Node.js", runtime_name);
                return Err(anyhow::anyhow!(
                    "{} is bundled with Node.js. Please install Node.js first.",
                    runtime_name
                ));
            }

            "yarn" => {
                // Install yarn via npm
                self.run_install_command("npm", &["install", "-g", "yarn"])
                    .await?;
            }

            "pnpm" => {
                // Install pnpm via npm
                self.run_install_command("npm", &["install", "-g", "pnpm"])
                    .await?;
            }

            // Python ecosystem
            "uv" => {
                // Install uv via pip or standalone installer
                if self.check_command_exists("pip").await {
                    self.run_install_command("pip", &["install", "uv"]).await?;
                } else if self.check_command_exists("pip3").await {
                    self.run_install_command("pip3", &["install", "uv"]).await?;
                } else {
                    // Use standalone installer
                    #[cfg(windows)]
                    {
                        self.run_install_command(
                            "powershell",
                            &[
                                "-ExecutionPolicy",
                                "ByPass",
                                "-c",
                                "irm https://astral.sh/uv/install.ps1 | iex",
                            ],
                        )
                        .await?;
                    }
                    #[cfg(not(windows))]
                    {
                        self.run_install_command(
                            "sh",
                            &["-c", "curl -LsSf https://astral.sh/uv/install.sh | sh"],
                        )
                        .await?;
                    }
                }
            }

            "uvx" => {
                // uvx is part of uv
                return Err(anyhow::anyhow!(
                    "uvx is part of uv. Please install uv first."
                ));
            }

            // Rust ecosystem
            "rustup" => {
                #[cfg(windows)]
                {
                    return Err(anyhow::anyhow!(
                        "Please install Rust from https://rustup.rs/"
                    ));
                }
                #[cfg(not(windows))]
                {
                    self.run_install_command(
                        "sh",
                        &[
                            "-c",
                            "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
                        ],
                    )
                    .await?;
                }
            }

            "cargo" | "rustc" => {
                return Err(anyhow::anyhow!(
                    "{} is installed via rustup. Please install Rust first.",
                    runtime_name
                ));
            }

            // Go
            "go" | "golang" => {
                return Err(anyhow::anyhow!(
                    "Please install Go from https://go.dev/dl/ or run 'vx install go'"
                ));
            }

            // Bun
            "bun" => {
                #[cfg(windows)]
                {
                    self.run_install_command(
                        "powershell",
                        &[
                            "-ExecutionPolicy",
                            "ByPass",
                            "-c",
                            "irm bun.sh/install.ps1 | iex",
                        ],
                    )
                    .await?;
                }
                #[cfg(not(windows))]
                {
                    self.run_install_command(
                        "sh",
                        &["-c", "curl -fsSL https://bun.sh/install | bash"],
                    )
                    .await?;
                }
            }

            _ => {
                // Unknown runtime
                if let Some(spec) = spec {
                    return Err(anyhow::anyhow!(
                        "Cannot auto-install '{}' ({}). Please install it manually.",
                        runtime_name,
                        spec.description
                    ));
                } else {
                    return Err(anyhow::anyhow!(
                        "Unknown runtime '{}'. Cannot auto-install.",
                        runtime_name
                    ));
                }
            }
        }

        Ok(())
    }

    /// Check if a command exists
    async fn check_command_exists(&self, cmd: &str) -> bool {
        which::which(cmd).is_ok()
    }

    /// Run an installation command
    async fn run_install_command(&self, cmd: &str, args: &[&str]) -> Result<()> {
        info!("Running: {} {}", cmd, args.join(" "));

        let status = Command::new(cmd)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await?;

        if !status.success() {
            return Err(anyhow::anyhow!(
                "Installation command failed with exit code: {:?}",
                status.code()
            ));
        }

        Ok(())
    }

    /// Try to run a command to verify installation
    async fn install_via_command(&self, cmd: &str, args: &[&str]) -> Result<()> {
        let status = Command::new(cmd)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Command failed"))
        }
    }

    /// Build the command to execute
    fn build_command(
        &self,
        resolution: &crate::resolver::ResolutionResult,
        args: &[String],
        runtime_env: &std::collections::HashMap<String, String>,
    ) -> Result<Command> {
        let executable = &resolution.executable;

        // On Windows, .cmd and .bat files need to be executed via cmd.exe
        #[cfg(windows)]
        let mut cmd = {
            let ext = executable
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if ext == "cmd" || ext == "bat" {
                let mut c = Command::new("cmd.exe");
                c.arg("/c").arg(executable);
                c
            } else {
                Command::new(executable)
            }
        };

        #[cfg(not(windows))]
        let mut cmd = Command::new(executable);

        // Add command prefix if any (e.g., "tool run" for uvx)
        for prefix in &resolution.command_prefix {
            cmd.arg(prefix);
        }

        // Add user arguments
        cmd.args(args);

        // Inject runtime-specific environment variables
        if !runtime_env.is_empty() {
            debug!(
                "Injecting {} environment variables for {}",
                runtime_env.len(),
                resolution.runtime
            );
            for (key, value) in runtime_env {
                cmd.env(key, value);
            }
        }

        // Inherit stdio
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        Ok(cmd)
    }

    /// Run the command and return its status
    async fn run_command(&self, cmd: &mut Command) -> Result<ExitStatus> {
        let status = if let Some(timeout) = self.config.execution_timeout {
            tokio::time::timeout(timeout, cmd.status())
                .await
                .map_err(|_| anyhow::anyhow!("Command execution timed out"))??
        } else {
            cmd.status().await?
        };

        Ok(status)
    }

    /// Get the resolver (for inspection)
    pub fn resolver(&self) -> &Resolver {
        &self.resolver
    }

    /// Get the configuration
    pub fn config(&self) -> &ResolverConfig {
        &self.config
    }
}

/// Execute a runtime directly using system PATH (simple fallback)
pub async fn execute_system_runtime(runtime_name: &str, args: &[String]) -> Result<i32> {
    debug!(
        "Executing system runtime: {} {}",
        runtime_name,
        args.join(" ")
    );

    let status = Command::new(runtime_name)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to execute '{}': {}", runtime_name, e))?;

    Ok(status.code().unwrap_or(1))
}
