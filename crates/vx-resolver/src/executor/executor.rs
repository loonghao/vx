//! Executor Core - the main command forwarding engine
//!
//! This module contains the Executor struct and its main execution flow.
//! The implementation is split across multiple modules for single responsibility:
//!
//! - `installation.rs` - Runtime installation logic
//! - `fallback.rs` - Fallback installation methods
//! - `environment.rs` - Environment variable preparation and PATH building
//! - `command.rs` - Command building and execution

use super::bundle::{execute_bundle, has_bundle, is_online, try_get_bundle_context};
use super::environment::EnvironmentManager;
use super::installation::InstallationManager;
use super::pipeline::error::PipelineError;
use super::project_config::ProjectToolsConfig;
use crate::{ResolutionCache, Resolver, ResolverConfig, Result, RuntimeMap};
use std::path::PathBuf;
use tracing::{debug, info, info_span};
use vx_paths::project::find_vx_config;
use vx_runtime::{CacheMode, ProviderRegistry, RuntimeContext};

/// Executor for runtime command forwarding
pub struct Executor<'a> {
    /// Configuration
    config: ResolverConfig,

    /// Runtime resolver
    resolver: Resolver,

    /// Optional disk-backed resolution cache
    #[allow(dead_code)]
    resolution_cache: Option<ResolutionCache>,

    /// Optional provider registry for installation
    registry: Option<&'a ProviderRegistry>,

    /// Runtime context for installation
    context: Option<&'a RuntimeContext>,

    /// Project configuration (loaded from vx.toml if present)
    project_config: Option<ProjectToolsConfig>,
}

impl<'a> Executor<'a> {
    /// Create an executor with a provider registry, runtime context, and runtime map
    pub fn new(
        config: ResolverConfig,
        registry: &'a ProviderRegistry,
        context: &'a RuntimeContext,
        runtime_map: RuntimeMap,
    ) -> Result<Self> {
        let resolver = Resolver::new(config.clone(), runtime_map)?;
        let resolution_cache = ResolutionCache::default_paths(&config)
            .map_err(|e| {
                debug!("Resolution cache disabled: {}", e);
                e
            })
            .ok();

        // Load project configuration from vx.toml
        let project_config = ProjectToolsConfig::load();
        if project_config.is_some() {
            debug!("Project configuration loaded for executor");
        }

        Ok(Self {
            config,
            resolver,
            resolution_cache,
            registry: Some(registry),
            context: Some(context),
            project_config,
        })
    }

    /// Set the runtime context
    pub fn set_context(&mut self, context: &'a RuntimeContext) {
        self.context = Some(context);
    }

    /// Execute a runtime with the given arguments
    pub async fn execute(&self, runtime_name: &str, args: &[String]) -> Result<i32> {
        self.execute_with_version(runtime_name, None, args).await
    }

    /// Execute a runtime with the given arguments and optional version constraint
    pub async fn execute_with_version(
        &self,
        runtime_name: &str,
        version: Option<&str>,
        args: &[String],
    ) -> Result<i32> {
        self.execute_with_version_and_env(runtime_name, version, args, false)
            .await
    }

    /// Execute a runtime with environment inheritance control
    pub async fn execute_with_version_and_env(
        &self,
        runtime_name: &str,
        version: Option<&str>,
        args: &[String],
        inherit_env: bool,
    ) -> Result<i32> {
        self.execute_with_with_deps(runtime_name, version, args, inherit_env, &[])
            .await
    }

    /// Execute a runtime with additional runtime dependencies (--with flag)
    ///
    /// This method supports injecting additional runtimes into the PATH before execution,
    /// similar to uvx --with or rez-env. Useful when a tool requires multiple runtimes.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Execute opencode with bun in PATH
    /// executor.execute_with_with_deps(
    ///     "npm:opencode",
    ///     None,
    ///     &args,
    ///     false,
    ///     &[WithDependency::parse("bun")],
    /// ).await?;
    /// ```
    pub async fn execute_with_with_deps(
        &self,
        runtime_name: &str,
        version: Option<&str>,
        args: &[String],
        inherit_env: bool,
        with_deps: &[vx_core::WithDependency],
    ) -> Result<i32> {
        let span = info_span!(
            "execute",
            tool = %runtime_name,
            ver = version.unwrap_or("latest"),
        );
        let _guard = span.enter();

        // Log the command being executed
        if let Some(ver) = version {
            debug!(">>> vx {}@{} {}", runtime_name, ver, args.join(" "));
        } else {
            debug!(">>> vx {} {}", runtime_name, args.join(" "));
        }

        // Log --with dependencies if any
        if !with_deps.is_empty() {
            info!(
                "Injecting additional runtimes via --with: {:?}",
                with_deps.iter().map(|d| d.to_string()).collect::<Vec<_>>()
            );
        }

        // -------------------------
        // Pre-check: Offline Bundle
        // -------------------------
        if let Some(exit_code) = self.try_offline_bundle(runtime_name, args).await? {
            return Ok(exit_code);
        }

        // -------------------------
        // Pre-check: Platform Support
        // -------------------------
        if let Some(registry) = self.registry {
            if let Some(runtime) = registry.get_runtime(runtime_name) {
                if let Err(e) = runtime.check_platform_support() {
                    return Err(PipelineError::PlatformCheckFailed {
                        runtime: runtime_name.to_string(),
                        reason: e.to_string(),
                    }
                    .into());
                }
            }
        }

        // -------------------------
        // Pipeline: Resolve → Ensure → Prepare → Execute
        // -------------------------
        use super::pipeline::stage::Stage;
        use super::pipeline::stages::ensure::EnsureStage;
        use super::pipeline::stages::execute::ExecuteStage;
        use super::pipeline::stages::prepare::PrepareStage;
        use super::pipeline::stages::resolve::{ResolveRequest, ResolveStage, WithDepRequest};

        // Build ResolveRequest from arguments
        let with_dep_requests: Vec<WithDepRequest> = with_deps.iter().map(Into::into).collect();
        let mut request = ResolveRequest::new(runtime_name, args.to_vec());
        request.version = version.map(|v| v.to_string());
        request.with_deps = with_dep_requests;
        request.inherit_env = inherit_env;
        request.auto_install = self.config.auto_install;
        request.inherit_vx_path = self.config.inherit_vx_path;

        // Build stages
        let store_base = self.context.map(|ctx| ctx.paths.store_dir());

        let mut resolve_stage = ResolveStage::new(&self.resolver, &self.config);
        if let Some(ref project_config) = self.project_config {
            resolve_stage = resolve_stage.with_project_config(project_config);
        }
        if let Some(ref base) = store_base {
            resolve_stage = resolve_stage.with_store_base(base.clone());
        }

        let ensure_stage =
            EnsureStage::new(&self.resolver, &self.config, self.registry, self.context);

        let mut prepare_stage =
            PrepareStage::new(&self.resolver, &self.config, self.registry, self.context);
        if let Some(ref project_config) = self.project_config {
            prepare_stage = prepare_stage.with_project_config(project_config);
        }

        let execute_stage = if let Some(timeout) = self.config.execution_timeout {
            ExecuteStage::new().with_timeout(timeout)
        } else {
            ExecuteStage::new()
        };

        // Stage 1: Resolve
        let plan = {
            let _span = tracing::info_span!("resolve", runtime = %runtime_name).entered();
            debug!("[Pipeline] Resolve");
            resolve_stage
                .execute(request)
                .await
                .map_err(PipelineError::from)?
        };

        // Check incompatible dependencies (from resolution)
        let resolved_version = plan.primary.version_string().map(|s| s.to_string());
        if let Ok(check_resolution) = self
            .resolver
            .resolve_with_version(runtime_name, resolved_version.as_deref())
        {
            if !check_resolution.incompatible_dependencies.is_empty() {
                let details: Vec<String> = check_resolution
                    .incompatible_dependencies
                    .iter()
                    .map(|ic| {
                        format!(
                            "{}: current={:?}, recommended={:?}",
                            ic.runtime_name, ic.current_version, ic.recommended_version
                        )
                    })
                    .collect();
                return Err(PipelineError::IncompatibleDependencies {
                    details: details.join("; "),
                }
                .into());
            }
        }

        // Stage 2: Ensure installed
        let plan = {
            let _span = tracing::info_span!("ensure", runtime = %runtime_name).entered();
            debug!("[Pipeline] Ensure");
            ensure_stage
                .execute(plan)
                .await
                .map_err(PipelineError::from)?
        };

        // Stage 3: Prepare environment
        let mut prepared = {
            let _span = tracing::info_span!("prepare", runtime = %runtime_name).entered();
            debug!("[Pipeline] Prepare");
            prepare_stage
                .execute(plan)
                .await
                .map_err(PipelineError::from)?
        };

        // -------------------------
        // Post-prepare: --with Dependencies Injection
        // -------------------------
        if !with_deps.is_empty() {
            debug!("[WITH_DEPS]");
            self.inject_with_dependencies(&mut prepared.env, with_deps)
                .await?;
            debug!("[/WITH_DEPS]");
        }

        // -------------------------
        // Post-prepare: RFC 0028 Proxy Execution
        // -------------------------
        self.apply_proxy_execution(runtime_name, &mut prepared, inherit_env)
            .await?;

        // Add executable's parent directory to PATH
        self.add_executable_dir_to_prepared_path(&mut prepared);

        // Stage 4: Execute
        let exit_code = {
            let _span = tracing::info_span!("execute_process", runtime = %runtime_name).entered();
            debug!("[Pipeline] Execute");
            execute_stage
                .execute(prepared)
                .await
                .map_err(PipelineError::from)?
        };

        // Persist exec path cache (new entries discovered during resolution)
        self.resolver.save_exec_cache();

        Ok(exit_code)
    }

    /// Apply RFC 0028 proxy execution overrides to a PreparedExecution
    async fn apply_proxy_execution(
        &self,
        runtime_name: &str,
        prepared: &mut super::pipeline::stages::prepare::PreparedExecution,
        inherit_env: bool,
    ) -> Result<()> {
        let registry = match self.registry {
            Some(r) => r,
            None => return Ok(()),
        };

        let runtime = match registry.get_runtime(runtime_name) {
            Some(r) => r,
            None => return Ok(()),
        };

        // Determine version to check
        let version_to_check = prepared
            .plan
            .primary
            .version_string()
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                // Try to get from installed versions
                "latest".to_string()
            });

        if runtime.is_version_installable(&version_to_check) {
            return Ok(());
        }

        let prep = self
            .prepare_proxy_execution(
                runtime_name,
                &version_to_check,
                &prepared.env,
                inherit_env,
                runtime.as_ref(),
            )
            .await?;

        if let Some(ref msg) = prep.message {
            info!("{}", msg);
        }

        if !prep.proxy_ready && !prep.use_system_path && prep.executable_override.is_none() {
            return Err(PipelineError::Prepare(
                super::pipeline::error::PrepareError::ProxyNotAvailable {
                    runtime: runtime_name.to_string(),
                    proxy: version_to_check.clone(),
                    reason: "proxy mechanism is not ready".to_string(),
                },
            )
            .into());
        }

        // Apply overrides
        if prep.use_system_path {
            if let Ok(system_exe) = which::which(runtime_name) {
                prepared.executable = system_exe;
            }
        }
        if let Some(exe_override) = prep.executable_override {
            prepared.executable = exe_override;
        }
        if !prep.command_prefix.is_empty() {
            prepared.command_prefix = prep.command_prefix;
        }
        prepared.env.extend(prep.env_vars);

        if !prep.path_prepend.is_empty() {
            self.apply_path_prepend(&mut prepared.env, &prep.path_prepend);
        }

        Ok(())
    }

    /// Add executable's parent directory to PATH in a PreparedExecution
    fn add_executable_dir_to_prepared_path(
        &self,
        prepared: &mut super::pipeline::stages::prepare::PreparedExecution,
    ) {
        if prepared.executable.is_absolute() {
            if let Some(exe_dir) = prepared.executable.parent() {
                let exe_dir_str = exe_dir.to_string_lossy().to_string();
                let path_sep = vx_paths::path_separator();
                let grandparent_dir = exe_dir.parent().map(|p| p.to_string_lossy().to_string());

                let current_path = prepared
                    .env
                    .get("PATH")
                    .cloned()
                    .or_else(|| std::env::var("PATH").ok())
                    .unwrap_or_default();

                let mut new_path = exe_dir_str.clone();
                if let Some(ref gp) = grandparent_dir {
                    if !new_path.contains(gp) {
                        new_path = format!("{}{}{}", new_path, path_sep, gp);
                    }
                }
                if !current_path.is_empty() {
                    new_path = format!("{}{}{}", new_path, path_sep, current_path);
                }

                prepared.env.insert("PATH".to_string(), new_path);
                debug!("  Added executable dir to PATH: {}", exe_dir.display());
            }
        }
    }

    // ========== Helper Methods ==========

    /// Create an installation manager
    fn installation_manager(&self) -> InstallationManager<'_> {
        InstallationManager::new(&self.config, &self.resolver, self.registry, self.context)
    }

    /// Create an environment manager
    fn environment_manager(&self) -> EnvironmentManager<'_> {
        EnvironmentManager::new(
            &self.config,
            &self.resolver,
            self.registry,
            self.context,
            self.project_config.as_ref(),
        )
    }

    /// Try to use offline bundle
    async fn try_offline_bundle(&self, runtime_name: &str, args: &[String]) -> Result<Option<i32>> {
        let force_offline = self
            .context
            .map(|ctx| ctx.config.cache_mode == CacheMode::Offline)
            .unwrap_or(false);

        if let Some(bundle_ctx) = try_get_bundle_context(runtime_name, force_offline) {
            info!(
                "Using offline bundle for {} {} (network: {})",
                runtime_name,
                bundle_ctx.version,
                if is_online() {
                    "online but forced offline"
                } else {
                    "offline"
                }
            );
            return Ok(Some(execute_bundle(&bundle_ctx, args).await?));
        }

        let network_offline = !is_online();
        if force_offline || network_offline {
            let cwd = std::env::current_dir().ok();
            let has_project_bundle = cwd
                .and_then(|cwd| {
                    find_vx_config(&cwd)
                        .ok()
                        .and_then(|p| p.parent().map(has_bundle))
                })
                .unwrap_or(false);

            if has_project_bundle {
                return Err(PipelineError::Offline(format!(
                    "tool '{}' not found in bundle. Run 'vx bundle create' while online to add it.",
                    runtime_name
                ))
                .into());
            } else if network_offline {
                return Err(PipelineError::Offline(
                    "no bundle available and network is offline. \
                     Run 'vx bundle create' while online to create one."
                        .to_string(),
                )
                .into());
            }
        }

        Ok(None)
    }

    /// Prepare proxy execution for bundled runtimes (RFC 0028)
    async fn prepare_proxy_execution(
        &self,
        runtime_name: &str,
        version: &str,
        runtime_env: &std::collections::HashMap<String, String>,
        inherit_env: bool,
        runtime: &dyn vx_runtime::Runtime,
    ) -> Result<vx_runtime::ExecutionPrep> {
        debug!(
            "[PREPARE_PROXY] Preparing proxy execution for {}@{}",
            runtime_name, version
        );

        let exec_ctx = vx_runtime::ExecutionContext {
            working_dir: std::env::current_dir().ok(),
            env: runtime_env.clone(),
            capture_output: false,
            timeout: self.config.execution_timeout,
            executor: std::sync::Arc::new(vx_runtime::RealCommandExecutor),
        };

        // Try prepare_execution first
        let prep_result = runtime.prepare_execution(version, &exec_ctx).await;

        match prep_result {
            Ok(p) => {
                debug!("[/PREPARE_PROXY]");
                Ok(p)
            }
            Err(e) => {
                // RFC 0028: If prepare_execution fails for a bundled runtime,
                // try to auto-install the parent runtime and retry
                // Look for a required dependency with provided_by set
                //
                // First check static dependencies (from bundled_with, managed_by, or when="*" constraints)
                // Then check version-specific dependencies (from when=">=2" etc. constraints)
                let parent_runtime = self
                    .resolver
                    .get_spec(runtime_name)
                    .and_then(|spec| {
                        spec.dependencies
                            .iter()
                            .find(|dep| dep.required && dep.provided_by.is_some())
                            .and_then(|dep| dep.provided_by.clone())
                    })
                    .or_else(|| {
                        // Query version-specific dependencies from provider.toml constraints
                        // This handles cases like Yarn 2.x+ where when=">=2, <4" constraints
                        // specify that Node.js (via corepack) provides Yarn
                        self.resolver
                            .get_parent_runtime_for_version(runtime_name, version)
                    });

                if let Some(ref parent) = parent_runtime {
                    if self.config.auto_install {
                        info!(
                            "'{}' requires '{}'. Auto-installing parent runtime...",
                            runtime_name, parent
                        );

                        // Install the parent runtime
                        let install_mgr = self.installation_manager();
                        install_mgr
                            .install_runtimes(std::slice::from_ref(parent))
                            .await?;

                        // Update runtime_env with parent runtime's environment
                        let env_mgr = self.environment_manager();
                        let parent_env = env_mgr
                            .prepare_runtime_environment(parent, None, inherit_env)
                            .await?;
                        let mut updated_env = runtime_env.clone();
                        updated_env.extend(parent_env);

                        // Retry prepare_execution
                        let retry_exec_ctx = vx_runtime::ExecutionContext {
                            working_dir: std::env::current_dir().ok(),
                            env: updated_env,
                            capture_output: false,
                            timeout: self.config.execution_timeout,
                            executor: std::sync::Arc::new(vx_runtime::RealCommandExecutor),
                        };

                        runtime
                            .prepare_execution(version, &retry_exec_ctx)
                            .await
                            .map_err(|retry_err| {
                                super::pipeline::error::PrepareError::ProxyRetryFailed {
                                    runtime: runtime_name.to_string(),
                                    dependency: parent.clone(),
                                    reason: retry_err.to_string(),
                                }
                                .into()
                            })
                    } else {
                        Err(super::pipeline::error::PrepareError::DependencyRequired {
                            runtime: runtime_name.to_string(),
                            dependency: parent.clone(),
                            reason: e.to_string(),
                        }
                        .into())
                    }
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Inject additional runtime dependencies via --with flag
    ///
    /// This method handles the --with dependencies similar to uvx --with or rez-env:
    /// 1. Install missing runtimes if auto_install is enabled
    /// 2. Prepend their bin directories to PATH
    /// 3. Add their environment variables
    async fn inject_with_dependencies(
        &self,
        runtime_env: &mut std::collections::HashMap<String, String>,
        with_deps: &[vx_core::WithDependency],
    ) -> Result<()> {
        let registry = match self.registry {
            Some(r) => r,
            None => return Ok(()),
        };
        let context = match self.context {
            Some(c) => c,
            None => return Ok(()),
        };

        let mut path_prepend: Vec<PathBuf> = Vec::new();

        for dep in with_deps {
            let runtime_name = &dep.runtime;
            let requested_version = dep.version.as_deref();

            debug!(
                "  Injecting --with dependency: {}@{}",
                runtime_name,
                requested_version.unwrap_or("latest")
            );

            // Check if runtime exists in registry
            let runtime = match registry.get_runtime(runtime_name) {
                Some(r) => r,
                None => {
                    return Err(
                        super::pipeline::error::ResolveError::UnknownWithDependency {
                            runtime: runtime_name.to_string(),
                            available: registry
                                .supported_runtimes()
                                .iter()
                                .map(|r| r.name())
                                .collect::<Vec<_>>()
                                .join(", "),
                        }
                        .into(),
                    );
                }
            };

            // Determine version to use
            let version = if let Some(v) = requested_version {
                v.to_string()
            } else {
                // Get latest installed version or install latest
                match runtime.installed_versions(context).await {
                    Ok(versions) if !versions.is_empty() => versions[0].clone(),
                    _ => {
                        // Not installed, try to auto-install
                        if self.config.auto_install {
                            info!(
                                "  --with dependency '{}' is not installed. Auto-installing...",
                                runtime_name
                            );
                            let install_mgr = self.installation_manager();
                            install_mgr
                                .install_runtimes(std::slice::from_ref(runtime_name))
                                .await?;

                            // Get the installed version
                            runtime
                                .installed_versions(context)
                                .await
                                .ok()
                                .and_then(|v| v.into_iter().next())
                                .unwrap_or_else(|| "latest".to_string())
                        } else {
                            return Err(super::pipeline::error::EnsureError::AutoInstallDisabled {
                                runtime: runtime_name.to_string(),
                                version: "latest".to_string(),
                            }
                            .into());
                        }
                    }
                }
            };

            // Ensure requested version is installed
            if requested_version.is_some()
                && !runtime
                    .is_installed(&version, context)
                    .await
                    .unwrap_or(false)
            {
                if self.config.auto_install {
                    info!(
                        "  --with dependency '{}@{}' is not installed. Auto-installing...",
                        runtime_name, version
                    );
                    let install_mgr = self.installation_manager();
                    install_mgr
                        .ensure_version_installed(runtime_name, &version)
                        .await?;
                } else {
                    return Err(super::pipeline::error::EnsureError::AutoInstallDisabled {
                        runtime: runtime_name.to_string(),
                        version: version.clone(),
                    }
                    .into());
                }
            }

            // Get bin directory for this runtime
            let vx_paths = vx_paths::VxPaths::with_base_dir(context.paths.vx_home());
            if let Ok(Some(runtime_root)) =
                vx_paths::RuntimeRoot::find(runtime_name, &version, &vx_paths)
            {
                // Add bin directory to path_prepend
                let bin_dir = runtime_root.bin_dir();
                if bin_dir.exists() {
                    debug!("    Adding bin dir to PATH: {}", bin_dir.display());
                    path_prepend.push(bin_dir.to_path_buf());
                }

                // Add environment variables
                let env_vars = runtime_root.env_vars();
                debug!(
                    "    Adding {} env vars for {}",
                    env_vars.len(),
                    runtime_name
                );
                runtime_env.extend(env_vars);
            } else {
                // Fallback: try to get executable path directly
                let exe_path = context.paths.executable_path(runtime_name, &version);
                if let Some(bin_dir) = exe_path.parent() {
                    if bin_dir.exists() {
                        debug!(
                            "    Adding bin dir to PATH (fallback): {}",
                            bin_dir.display()
                        );
                        path_prepend.push(bin_dir.to_path_buf());
                    }
                }
            }

            info!("  Injected --with dependency: {}@{}", runtime_name, version);
        }

        // Prepend all --with dependency paths to PATH
        if !path_prepend.is_empty() {
            self.apply_path_prepend(runtime_env, &path_prepend);
            debug!(
                "  Prepended {} paths from --with dependencies",
                path_prepend.len()
            );
        }

        Ok(())
    }

    /// Apply PATH prepend from execution prep
    fn apply_path_prepend(
        &self,
        runtime_env: &mut std::collections::HashMap<String, String>,
        path_prepend: &[PathBuf],
    ) {
        let current_path = runtime_env
            .get("PATH")
            .cloned()
            .or_else(|| std::env::var("PATH").ok())
            .unwrap_or_default();

        let path_sep = vx_paths::path_separator();
        let prepend_str: String = path_prepend
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(&path_sep.to_string());

        let new_path = if current_path.is_empty() {
            prepend_str
        } else {
            format!("{}{}{}", prepend_str, path_sep, current_path)
        };

        runtime_env.insert("PATH".to_string(), new_path);
        debug!("  Prepended {} paths to PATH", path_prepend.len());
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
