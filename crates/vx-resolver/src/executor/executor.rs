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
use super::command::{build_command, run_command};
use super::environment::EnvironmentManager;
use super::installation::InstallationManager;
use super::project_config::ProjectToolsConfig;
use crate::{ResolutionCache, Resolver, ResolverConfig, Result, RuntimeMap};
use std::path::PathBuf;
use tracing::{debug, info, info_span};
use vx_core::exit_code_from_status;
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

        // -------------------------
        // Offline Bundle Check
        // -------------------------
        if let Some(exit_code) = self.try_offline_bundle(runtime_name, args).await? {
            return Ok(exit_code);
        }

        // Check platform support
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
        debug!("[RESOLVE]");
        let resolved_version = self.resolve_version(runtime_name, version);
        debug!(
            "  version: {:?}",
            resolved_version.as_ref().unwrap_or(&"latest".to_string())
        );

        let mut resolution = self
            .resolver
            .resolve_with_version(runtime_name, resolved_version.as_deref())?;
        debug!("  executable: {}", resolution.executable.display());
        debug!("  needs_install: {}", resolution.runtime_needs_install);
        debug!("[/RESOLVE]");

        // -------------------------
        // Ensure Installed
        // -------------------------
        debug!("[INSTALL_CHECK]");
        let install_mgr = self.installation_manager();
        let mut installed_version: Option<String> = None;
        let needs_re_resolve =
            resolution.runtime_needs_install || !resolution.executable.is_absolute();

        // If a specific version is requested, ensure it's installed first
        if let Some(requested_version) = resolved_version.clone() {
            installed_version = install_mgr
                .ensure_version_installed(runtime_name, &requested_version)
                .await?;
            debug!("  installed_version: {:?}", installed_version);
        }

        // Install missing runtimes/dependencies (if any)
        if !resolution.install_order.is_empty() && self.config.auto_install {
            let runtimes_to_install = self.filter_installable_runtimes(
                &resolution.install_order,
                runtime_name,
                installed_version.is_some(),
            );

            if !runtimes_to_install.is_empty() {
                info!("  auto-installing: {:?}", runtimes_to_install);
                install_mgr.install_runtimes(&runtimes_to_install).await?;
            }
        } else if !resolution.install_order.is_empty() {
            return Err(self.missing_dependencies_error(runtime_name, &resolution));
        }

        // Re-resolve after installation
        if needs_re_resolve {
            debug!("[RE-RESOLVE] Re-resolving after installation");
            let re_resolve_version = installed_version.as_deref().or(resolved_version.as_deref());
            resolution = self
                .resolver
                .resolve_with_version(runtime_name, re_resolve_version)?;
            debug!(
                "[RE-RESOLVE] Updated executable: {}",
                resolution.executable.display()
            );
        }

        // Check incompatible dependencies
        if !resolution.incompatible_dependencies.is_empty() {
            for incompatible in &resolution.incompatible_dependencies {
                tracing::warn!(
                    "Incompatible dependency {}: current={:?}, recommended={:?}",
                    incompatible.runtime_name,
                    incompatible.current_version,
                    incompatible.recommended_version
                );
            }
            return Err(anyhow::anyhow!(
                "Some dependencies are incompatible. Please install compatible versions."
            ));
        }

        // -------------------------
        // Prepare Environment
        // -------------------------
        debug!("[/INSTALL_CHECK]");
        debug!("[PREPARE_ENV]");
        let env_mgr = self.environment_manager();
        let mut runtime_env = env_mgr
            .prepare_runtime_environment(runtime_name, installed_version.as_deref(), inherit_env)
            .await?;
        debug!("  env_vars: {} variables set", runtime_env.len());
        debug!("[/PREPARE_ENV]");

        // -------------------------
        // RFC 0028: Prepare Proxy Execution
        // -------------------------
        if let Some(registry) = self.registry {
            if let Some(runtime) = registry.get_runtime(runtime_name) {
                let version_to_check = installed_version
                    .as_deref()
                    .or(resolved_version.as_deref())
                    .unwrap_or("latest");

                if !runtime.is_version_installable(version_to_check) {
                    let prep = self
                        .prepare_proxy_execution(
                            runtime_name,
                            version_to_check,
                            &runtime_env,
                            inherit_env,
                            runtime.as_ref(),
                        )
                        .await?;

                    // Apply execution prep
                    if let Some(ref msg) = prep.message {
                        info!("{}", msg);
                    }

                    if !prep.proxy_ready
                        && !prep.use_system_path
                        && prep.executable_override.is_none()
                    {
                        return Err(anyhow::anyhow!(
                            "Proxy setup for {}@{} failed. The proxy mechanism is not ready.",
                            runtime_name,
                            version_to_check
                        ));
                    }

                    // Apply overrides
                    if prep.use_system_path {
                        if let Ok(system_exe) = which::which(runtime_name) {
                            resolution.executable = system_exe;
                        }
                    }
                    if let Some(exe_override) = prep.executable_override {
                        resolution.executable = exe_override;
                    }
                    if !prep.command_prefix.is_empty() {
                        resolution.command_prefix = prep.command_prefix;
                    }
                    runtime_env.extend(prep.env_vars);

                    // Apply PATH prepend
                    if !prep.path_prepend.is_empty() {
                        self.apply_path_prepend(&mut runtime_env, &prep.path_prepend);
                    }
                }
            }
        }

        // -------------------------
        // Execute
        // -------------------------
        debug!("[EXECUTE]");
        // Verify executable exists
        if resolution.executable.is_absolute() && !resolution.executable.exists() {
            return Err(anyhow::anyhow!(
                "Executable not found at '{}'. Try running 'vx install {}'.",
                resolution.executable.display(),
                runtime_name
            ));
        }

        // Add executable's parent directory to PATH
        self.add_executable_dir_to_path(&resolution, &mut runtime_env);

        // Build and run command
        let vx_tools_path = if self.config.inherit_vx_path {
            env_mgr.build_vx_tools_path()
        } else {
            None
        };

        let mut cmd = build_command(
            &resolution,
            args,
            &runtime_env,
            self.config.inherit_vx_path,
            vx_tools_path,
        )?;

        debug!("  cmd: {} {:?}", resolution.executable.display(), args);
        debug!("--- tool output below ---");

        let status = run_command(&mut cmd, self.config.execution_timeout).await?;
        debug!("--- tool output above ---");
        debug!("[/EXECUTE] exit={}", exit_code_from_status(&status));

        Ok(exit_code_from_status(&status))
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
    async fn try_offline_bundle(
        &self,
        runtime_name: &str,
        args: &[String],
    ) -> Result<Option<i32>> {
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
                return Err(anyhow::anyhow!(
                    "Offline mode: tool '{}' not found in bundle. \
                     Run 'vx bundle create' while online to add it.",
                    runtime_name
                ));
            } else if network_offline {
                return Err(anyhow::anyhow!(
                    "Offline mode: no bundle available and network is offline. \
                     Run 'vx bundle create' while online to create one.",
                ));
            }
        }

        Ok(None)
    }

    /// Resolve version from command line or project config
    fn resolve_version(&self, runtime_name: &str, version: Option<&str>) -> Option<String> {
        if let Some(v) = version {
            Some(v.to_string())
        } else if let Some(ref project_config) = self.project_config {
            project_config
                .get_version_with_fallback(runtime_name)
                .map(|s| s.to_string())
        } else {
            None
        }
    }

    /// Filter out bundled runtimes from install list
    fn filter_installable_runtimes(
        &self,
        install_order: &[String],
        primary_runtime: &str,
        already_installed: bool,
    ) -> Vec<String> {
        install_order
            .iter()
            .filter(|r| {
                // Skip primary runtime if already processed
                if already_installed && *r == primary_runtime {
                    return false;
                }
                // RFC 0028: Skip bundled runtimes
                if let Some(registry) = self.registry {
                    if let Some(rt) = registry.get_runtime(r) {
                        if !rt.is_version_installable("latest") {
                            debug!(
                                "Skipping bundled runtime '{}' from install_order",
                                r
                            );
                            return false;
                        }
                    }
                }
                true
            })
            .cloned()
            .collect()
    }

    /// Create error for missing dependencies
    fn missing_dependencies_error(
        &self,
        runtime_name: &str,
        resolution: &crate::resolver::ResolutionResult,
    ) -> anyhow::Error {
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

        anyhow::anyhow!(
            "{}. Run 'vx install {}' or enable auto-install.",
            missing,
            runtime_name
        )
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
                        self.resolver.get_parent_runtime_for_version(runtime_name, version)
                    });

                if let Some(ref parent) = parent_runtime {
                    if self.config.auto_install {
                        info!(
                            "'{}' requires '{}'. Auto-installing parent runtime...",
                            runtime_name, parent
                        );

                        // Install the parent runtime
                        let install_mgr = self.installation_manager();
                        install_mgr.install_runtimes(&[parent.clone()]).await?;

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
                                anyhow::anyhow!(
                                    "Failed to prepare '{}' after installing '{}': {}",
                                    runtime_name,
                                    parent,
                                    retry_err
                                )
                            })
                    } else {
                        Err(anyhow::anyhow!(
                            "'{}' requires '{}' which is not installed.\n\n\
                             To install it, run:\n\n  vx install {}\n\n\
                             Or enable auto-install to install it automatically.\n\n\
                             Original error: {}",
                            runtime_name,
                            parent,
                            parent,
                            e
                        ))
                    }
                } else {
                    Err(e)
                }
            }
        }
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

    /// Add executable's parent directory to PATH
    fn add_executable_dir_to_path(
        &self,
        resolution: &crate::resolver::ResolutionResult,
        runtime_env: &mut std::collections::HashMap<String, String>,
    ) {
        if resolution.executable.is_absolute() {
            if let Some(exe_dir) = resolution.executable.parent() {
                let exe_dir_str = exe_dir.to_string_lossy().to_string();
                let path_sep = vx_paths::path_separator();
                let grandparent_dir = exe_dir.parent().map(|p| p.to_string_lossy().to_string());

                let current_path = runtime_env
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

                runtime_env.insert("PATH".to_string(), new_path);
                debug!("  Added executable dir to PATH: {}", exe_dir.display());
            }
        }
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
