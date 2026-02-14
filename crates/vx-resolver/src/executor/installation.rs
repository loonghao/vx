//! Runtime installation logic
//!
//! This module handles:
//! - Installing runtimes via provider registry
//! - Ensuring specific versions are installed
//! - Installing dependencies
//! - Proxy runtime installation (RFC 0028)
//! - Version fallback on installation failure

use super::pipeline::error::EnsureError;
use crate::{Resolver, ResolverConfig, Result};
use tracing::{debug, info, warn};
use vx_console::ProgressSpinner;
use vx_runtime::{InstallResult, ProviderRegistry, RuntimeContext};

/// Maximum number of version fallback attempts when installation fails
const MAX_FALLBACK_ATTEMPTS: usize = 3;

/// Installation operations for the executor
pub struct InstallationManager<'a> {
    pub(crate) config: &'a ResolverConfig,
    pub(crate) resolver: &'a Resolver,
    pub(crate) registry: Option<&'a ProviderRegistry>,
    pub(crate) context: Option<&'a RuntimeContext>,
}

impl<'a> InstallationManager<'a> {
    /// Create a new installation manager
    pub fn new(
        config: &'a ResolverConfig,
        resolver: &'a Resolver,
        registry: Option<&'a ProviderRegistry>,
        context: Option<&'a RuntimeContext>,
    ) -> Self {
        Self {
            config,
            resolver,
            registry,
            context,
        }
    }

    /// Install a list of runtimes in order
    ///
    /// Returns the InstallResult of the last installed runtime (typically the primary runtime)
    pub async fn install_runtimes(&self, runtimes: &[String]) -> Result<Option<InstallResult>> {
        let mut last_result = None;
        for runtime in runtimes {
            last_result = self.install_runtime(runtime).await?;
        }
        Ok(last_result)
    }

    /// Install a single runtime
    ///
    /// Returns the InstallResult (including executable_path) if successful.
    /// If the latest version fails verification, automatically falls back to
    /// the next available stable version (up to MAX_FALLBACK_ATTEMPTS times).
    pub async fn install_runtime(&self, runtime_name: &str) -> Result<Option<InstallResult>> {
        info!("Installing: {}", runtime_name);

        // Try using the provider registry first
        if let (Some(registry), Some(context)) = (self.registry, self.context)
            && let Some(runtime) = registry.get_runtime(runtime_name)
        {
            // Check platform support before attempting installation
            if let Err(e) = runtime.check_platform_support() {
                return Err(EnsureError::PlatformNotSupported {
                    runtime: runtime_name.to_string(),
                    reason: e.to_string(),
                }
                .into());
            }

            // Fetch versions to get the latest - show progress spinner
            let spinner =
                ProgressSpinner::new(&format!("Fetching versions for {}...", runtime_name));
            debug!("Fetching versions for {}", runtime_name);
            let versions = match runtime.fetch_versions(context).await {
                Ok(v) => {
                    spinner.finish_and_clear();
                    v
                }
                Err(e) => {
                    spinner.finish_with_error(&format!("Failed to fetch versions: {}", e));
                    return Err(e);
                }
            };

            // Collect stable versions for fallback
            let stable_versions: Vec<String> = versions
                .iter()
                .filter(|v| !v.prerelease)
                .map(|v| v.version.clone())
                .collect();

            let first_version = stable_versions
                .first()
                .cloned()
                .or_else(|| versions.first().map(|v| v.version.clone()))
                .ok_or_else(|| EnsureError::NoVersionsFound {
                    runtime: runtime_name.to_string(),
                })?;

            // Try installing with version fallback
            let mut last_error = None;
            let max_attempts = (MAX_FALLBACK_ATTEMPTS + 1).min(stable_versions.len().max(1));

            for attempt in 0..max_attempts {
                let version = if attempt == 0 {
                    first_version.clone()
                } else if attempt < stable_versions.len() {
                    let fallback = stable_versions[attempt].clone();
                    warn!(
                        "Installation of {} {} failed, falling back to {} (attempt {}/{})",
                        runtime_name,
                        stable_versions.get(attempt - 1).unwrap_or(&first_version),
                        fallback,
                        attempt + 1,
                        max_attempts
                    );
                    fallback
                } else {
                    break;
                };

                info!("Installing {} {} via provider", runtime_name, version);

                match self
                    .try_install_version(runtime_name, &version, context)
                    .await
                {
                    Ok(result) => return Ok(Some(result)),
                    Err(e) => {
                        debug!("Installation of {} {} failed: {}", runtime_name, version, e);
                        last_error = Some(e);
                    }
                }
            }

            if let Some(err) = last_error {
                return Err(err);
            }
        }

        // Fallback: try to install using known methods
        self.install_runtime_fallback(runtime_name).await?;
        Ok(None)
    }

    /// Try to install a specific version, returning an error on failure
    async fn try_install_version(
        &self,
        runtime_name: &str,
        version: &str,
        context: &RuntimeContext,
    ) -> Result<InstallResult> {
        let registry = self.registry.expect("registry must be set");
        let runtime = registry
            .get_runtime(runtime_name)
            .expect("runtime must exist");

        // Run pre-install hook
        runtime.pre_install(version, context).await?;

        // Install the runtime
        debug!("Calling runtime.install() for {} {}", runtime_name, version);
        let result = runtime.install(version, context).await?;
        debug!(
            "Install result: path={}, exe={}, already_installed={}",
            result.install_path.display(),
            result.executable_path.display(),
            result.already_installed
        );

        // Verify the installation actually succeeded
        if !context.fs.exists(&result.executable_path) {
            return Err(EnsureError::PostInstallVerificationFailed {
                runtime: runtime_name.to_string(),
                path: result.executable_path.clone(),
            }
            .into());
        }

        // Run post-install hook (for symlinks, PATH setup, etc.)
        runtime.post_install(version, context).await?;

        info!("Successfully installed {} {}", runtime_name, version);
        Ok(result)
    }

    /// Install a single runtime with a specific version.
    /// If installation fails, tries falling back to previous stable versions.
    pub async fn install_runtime_with_version(
        &self,
        runtime_name: &str,
        version: &str,
    ) -> Result<Option<InstallResult>> {
        info!("Installing: {}@{}", runtime_name, version);

        // Try using the provider registry first
        if let (Some(registry), Some(context)) = (self.registry, self.context)
            && let Some(runtime) = registry.get_runtime(runtime_name)
        {
            // Check platform support before attempting installation
            if let Err(e) = runtime.check_platform_support() {
                return Err(EnsureError::PlatformNotSupported {
                    runtime: runtime_name.to_string(),
                    reason: e.to_string(),
                }
                .into());
            }

            info!(
                "Installing {} {} via provider (explicit version)",
                runtime_name, version
            );

            // Try the requested version first
            match self
                .try_install_version(runtime_name, version, context)
                .await
            {
                Ok(result) => return Ok(Some(result)),
                Err(first_error) => {
                    warn!(
                        "Installation of {} {} failed: {}, trying fallback versions",
                        runtime_name, version, first_error
                    );

                    // Fetch available versions for fallback
                    if let Ok(versions) = runtime.fetch_versions(context).await {
                        let stable_versions: Vec<String> = versions
                            .iter()
                            .filter(|v| !v.prerelease && v.version != version)
                            .map(|v| v.version.clone())
                            .collect();

                        for (i, fallback_version) in stable_versions
                            .iter()
                            .take(MAX_FALLBACK_ATTEMPTS)
                            .enumerate()
                        {
                            warn!(
                                "Falling back to {} {} (attempt {}/{})",
                                runtime_name,
                                fallback_version,
                                i + 1,
                                MAX_FALLBACK_ATTEMPTS
                            );

                            match self
                                .try_install_version(runtime_name, fallback_version, context)
                                .await
                            {
                                Ok(result) => return Ok(Some(result)),
                                Err(e) => {
                                    debug!(
                                        "Fallback {} {} also failed: {}",
                                        runtime_name, fallback_version, e
                                    );
                                }
                            }
                        }
                    }

                    return Err(first_error);
                }
            }
        }

        // Fallback: try to install using known methods
        self.install_runtime_fallback(runtime_name).await?;
        Ok(None)
    }

    /// Ensure a specific version is installed
    ///
    /// This method handles version resolution (e.g., "20" -> "20.18.0") and installation.
    /// Returns the InstallResult (including executable_path) of the installed version.
    pub async fn ensure_version_installed(
        &self,
        runtime_name: &str,
        requested_version: &str,
    ) -> Result<Option<InstallResult>> {
        let (registry, context) = match (self.registry, self.context) {
            (Some(r), Some(c)) => (r, c),
            _ => return Ok(None),
        };

        let runtime = match registry.get_runtime(runtime_name) {
            Some(r) => r,
            None => {
                debug!(
                    "Runtime {} not found in registry, skipping version check",
                    runtime_name
                );
                return Ok(None);
            }
        };

        // Resolve the version constraint to an actual version
        let spinner = ProgressSpinner::new(&format!(
            "Resolving {}@{}...",
            runtime_name, requested_version
        ));
        let resolved_version = match runtime.resolve_version(requested_version, context).await {
            Ok(v) => {
                spinner.finish_and_clear();
                v
            }
            Err(e) => {
                spinner.finish_with_error(&format!("Failed to resolve version: {}", e));
                return Err(e);
            }
        };

        info!(
            "Resolved {}@{} → {}",
            runtime_name, requested_version, resolved_version
        );

        // RFC 0028: Check if this version is proxy-managed (not directly installable)
        if !runtime.is_version_installable(&resolved_version) {
            debug!(
                "{} {} is proxy-managed, skipping direct installation",
                runtime_name, resolved_version
            );
            // For proxy-managed versions, we don't install directly.
            // The prepare_execution() method will handle setting up the proxy.
            // However, we still need to ensure the proxy runtime is installed.
            self.ensure_proxy_runtime_installed(runtime_name, &resolved_version)
                .await?;
            // Return a proxy result without executable_path - the prepare stage
            // will handle proxy execution setup.
            return Ok(Some(InstallResult::proxy(resolved_version)));
        }

        // Check if this version is already installed
        if runtime.is_installed(&resolved_version, context).await? {
            debug!("{} {} is already installed", runtime_name, resolved_version);
            // Find the existing executable path via the resolver
            let exe_path = self
                .resolver
                .find_executable(runtime_name, &resolved_version);
            return Ok(Some(InstallResult::already_installed_with(
                resolved_version,
                exe_path,
            )));
        }

        // Install the specific version
        if !self.config.auto_install {
            return Err(EnsureError::AutoInstallDisabled {
                runtime: runtime_name.to_string(),
                version: resolved_version.clone(),
            }
            .into());
        }

        info!(
            "Auto-installing {} {} (requested: {})",
            runtime_name, resolved_version, requested_version
        );

        // Check for and install dependencies first
        self.install_dependencies_for_version(runtime_name, &resolved_version)
            .await?;

        // Try installing the resolved version with fallback to previous versions
        match self
            .try_install_version(runtime_name, &resolved_version, context)
            .await
        {
            Ok(result) => {
                info!(
                    "Successfully installed {} {}",
                    runtime_name, resolved_version
                );
                Ok(Some(result))
            }
            Err(first_error) => {
                warn!(
                    "Installation of {} {} failed: {}, trying fallback versions",
                    runtime_name, resolved_version, first_error
                );

                // Fetch versions and try previous stable versions
                if let Ok(versions) = runtime.fetch_versions(context).await {
                    let stable_versions: Vec<String> = versions
                        .iter()
                        .filter(|v| !v.prerelease && v.version != resolved_version)
                        .map(|v| v.version.clone())
                        .collect();

                    for (i, fallback_version) in stable_versions
                        .iter()
                        .take(MAX_FALLBACK_ATTEMPTS)
                        .enumerate()
                    {
                        warn!(
                            "Falling back to {} {} (attempt {}/{})",
                            runtime_name,
                            fallback_version,
                            i + 1,
                            MAX_FALLBACK_ATTEMPTS
                        );

                        // Install dependencies for fallback version too
                        if let Err(e) = self
                            .install_dependencies_for_version(runtime_name, fallback_version)
                            .await
                        {
                            debug!("Failed to install dependencies for fallback: {}", e);
                            continue;
                        }

                        match self
                            .try_install_version(runtime_name, fallback_version, context)
                            .await
                        {
                            Ok(result) => {
                                info!(
                                    "Successfully installed {} {} (fallback from {})",
                                    runtime_name, fallback_version, resolved_version
                                );
                                return Ok(Some(result));
                            }
                            Err(e) => {
                                debug!(
                                    "Fallback {} {} also failed: {}",
                                    runtime_name, fallback_version, e
                                );
                            }
                        }
                    }
                }

                Err(first_error)
            }
        }
    }

    /// RFC 0028: Ensure the proxy runtime is installed for proxy-managed tools
    pub async fn ensure_proxy_runtime_installed(
        &self,
        runtime_name: &str,
        _version: &str,
    ) -> Result<()> {
        // Get runtime spec to find dependencies
        if let Some(spec) = self.resolver.get_spec(runtime_name) {
            // Install all required dependencies
            for dep in &spec.dependencies {
                if dep.required {
                    info!(
                        "Installing proxy runtime {} for {} ({})",
                        dep.runtime_name, runtime_name, dep.reason
                    );

                    // Use recommended version if available, otherwise "latest"
                    let dep_version = dep.recommended_version.as_deref().unwrap_or("latest");
                    // Use Box::pin for recursive async call
                    Box::pin(self.ensure_version_installed(&dep.runtime_name, dep_version)).await?;
                }
            }
        }

        Ok(())
    }

    /// Install dependencies for a specific version of a runtime
    pub async fn install_dependencies_for_version(
        &self,
        runtime_name: &str,
        version: &str,
    ) -> Result<()> {
        // Get the runtime spec to check dependencies
        let spec = self.resolver.get_spec(runtime_name);

        if let Some(spec) = spec {
            // Count required dependencies that need installation
            let deps_to_install: Vec<_> = spec
                .dependencies
                .iter()
                .filter(|dep| dep.required)
                .filter(|dep| {
                    let dep_runtime = dep.provided_by.as_deref().unwrap_or(&dep.runtime_name);
                    let resolution = self.resolver.resolve(dep_runtime);
                    resolution.is_err() || resolution.as_ref().unwrap().runtime_needs_install
                })
                .collect();

            if deps_to_install.is_empty() {
                return Ok(());
            }

            // Show progress spinner for dependency installation
            let spinner = ProgressSpinner::new(&format!(
                "Installing {} dependencies for {}...",
                deps_to_install.len(),
                runtime_name
            ));

            for dep in deps_to_install {
                // Get the provider name (the actual runtime to install)
                let dep_runtime = dep.provided_by.as_deref().unwrap_or(&dep.runtime_name);

                // For Rust ecosystem, the dependency should use the same version
                let dep_version = if self.is_rust_ecosystem(runtime_name, dep_runtime) {
                    Some(version)
                } else {
                    dep.recommended_version.as_deref()
                };

                info!(
                    "Installing dependency {}@{} for {} ({})",
                    dep_runtime,
                    dep_version.unwrap_or("latest"),
                    runtime_name,
                    dep.reason
                );
                spinner.set_message(&format!(
                    "Installing dependency {}@{}...",
                    dep_runtime,
                    dep_version.unwrap_or("latest")
                ));

                if let Some(ver) = dep_version {
                    self.install_runtime_with_version(dep_runtime, ver).await?;
                } else {
                    self.install_runtime(dep_runtime).await?;
                }
            }

            spinner.finish_and_clear();
        }

        Ok(())
    }

    /// Check if the runtime and its dependency belong to the Rust ecosystem
    fn is_rust_ecosystem(&self, runtime_name: &str, dep_runtime: &str) -> bool {
        let rust_runtimes = ["cargo", "rustc", "rust", "rustup"];
        rust_runtimes.contains(&runtime_name) && rust_runtimes.contains(&dep_runtime)
    }
}
