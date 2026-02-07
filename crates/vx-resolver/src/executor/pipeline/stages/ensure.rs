//! Ensure Stage - install missing runtimes
//!
//! The second stage of the execution pipeline. Takes an `ExecutionPlan` and ensures
//! all runtimes marked as `NeedsInstall` are installed. The plan is updated in-place
//! with resolved executable paths.
//!
//! This stage wraps the existing `InstallationManager` to maintain backward compatibility.

use async_trait::async_trait;
use tracing::{debug, info};

use crate::executor::installation::InstallationManager;
use crate::executor::pipeline::error::EnsureError;
use crate::executor::pipeline::plan::{ExecutionPlan, InstallStatus};
use crate::executor::pipeline::stage::Stage;
use crate::{Resolver, ResolverConfig};
use vx_runtime::{ProviderRegistry, RuntimeContext};

/// The Ensure stage: `ExecutionPlan` → `ExecutionPlan` (with installations completed)
///
/// Iterates over all runtimes in the plan and installs any that are missing.
/// After installation, the plan's `PlannedRuntime` entries are updated with
/// their executable paths and `InstallStatus::Installed`.
pub struct EnsureStage<'a> {
    /// Resolver for re-resolution after installation
    resolver: &'a Resolver,

    /// Resolver config
    config: &'a ResolverConfig,

    /// Provider registry for runtime installation
    registry: Option<&'a ProviderRegistry>,

    /// Runtime context for installation
    context: Option<&'a RuntimeContext>,
}

impl<'a> EnsureStage<'a> {
    /// Create a new EnsureStage
    pub fn new(
        resolver: &'a Resolver,
        config: &'a ResolverConfig,
        registry: Option<&'a ProviderRegistry>,
        context: Option<&'a RuntimeContext>,
    ) -> Self {
        Self {
            resolver,
            config,
            registry,
            context,
        }
    }

    /// Create an InstallationManager (delegates to existing logic)
    fn installation_manager(&self) -> InstallationManager<'_> {
        InstallationManager::new(self.config, self.resolver, self.registry, self.context)
    }
}

#[async_trait]
impl<'a> Stage<ExecutionPlan, ExecutionPlan> for EnsureStage<'a> {
    type Error = EnsureError;

    async fn execute(&self, mut plan: ExecutionPlan) -> Result<ExecutionPlan, EnsureError> {
        // Check if auto-install is enabled
        if !plan.config.auto_install && plan.needs_install() {
            let missing: Vec<String> = plan
                .runtimes_needing_install()
                .iter()
                .map(|r| r.name.clone())
                .collect();
            return Err(EnsureError::AutoInstallDisabled {
                runtime: missing.join(", "),
                version: "required".to_string(),
            });
        }

        // Check for platform-unsupported runtimes first
        let unsupported = plan.unsupported_runtimes();
        if !unsupported.is_empty() {
            let reasons: Vec<String> = unsupported
                .iter()
                .filter_map(|r| {
                    if let InstallStatus::PlatformUnsupported { reason } = &r.status {
                        Some(format!("{}: {}", r.name, reason))
                    } else {
                        None
                    }
                })
                .collect();
            // Log as warning but don't fail — unsupported deps might be optional
            for reason in &reasons {
                tracing::warn!("Platform unsupported: {}", reason);
            }
        }

        if !plan.needs_install() {
            debug!("[EnsureStage] All runtimes already installed");
            return Ok(plan);
        }

        let install_mgr = self.installation_manager();

        // Install dependencies first (they're in topological order)
        for dep in &mut plan.dependencies {
            if dep.status == InstallStatus::NeedsInstall {
                debug!("[EnsureStage] Installing dependency: {}", dep.name);
                let version = dep.version_string().map(|s| s.to_string());

                let installed_version = if let Some(ver) = &version {
                    install_mgr
                        .install_runtime_with_version(&dep.name, ver)
                        .await
                        .map_err(|e| EnsureError::DependencyInstallFailed {
                            runtime: plan.primary.name.clone(),
                            dep: dep.name.clone(),
                            reason: e.to_string(),
                        })?
                } else {
                    install_mgr.install_runtime(&dep.name).await.map_err(|e| {
                        EnsureError::DependencyInstallFailed {
                            runtime: plan.primary.name.clone(),
                            dep: dep.name.clone(),
                            reason: e.to_string(),
                        }
                    })?
                };

                if let Some(ref actual_version) = installed_version {
                    dep.mark_installed_with_version(actual_version.clone(), None);
                    info!(
                        "[EnsureStage] Dependency {} installed (version: {})",
                        dep.name, actual_version
                    );
                }
            }
        }

        // Install the primary runtime
        if plan.primary.status == InstallStatus::NeedsInstall {
            debug!("[EnsureStage] Installing primary: {}", plan.primary.name);
            let version = plan.primary.version_string().map(|s| s.to_string());

            let installed_version = if let Some(ver) = &version {
                install_mgr
                    .ensure_version_installed(&plan.primary.name, ver)
                    .await
                    .map_err(|e| EnsureError::InstallFailed {
                        runtime: plan.primary.name.clone(),
                        version: ver.clone(),
                        reason: e.to_string(),
                    })?
            } else {
                install_mgr
                    .install_runtime(&plan.primary.name)
                    .await
                    .map_err(|e| EnsureError::InstallFailed {
                        runtime: plan.primary.name.clone(),
                        version: "latest".to_string(),
                        reason: e.to_string(),
                    })?
            };

            if let Some(ref actual_version) = installed_version {
                // Update both status and version to the actual installed version.
                // This is critical: the requested version may be "latest" or a range,
                // but the store directory uses the concrete version (e.g., "0.10.0").
                // Without this update, re-resolution would search for a directory
                // named "latest" which doesn't exist.
                plan.primary
                    .mark_installed_with_version(actual_version.clone(), None);
                info!(
                    "[EnsureStage] Primary {} installed (version: {})",
                    plan.primary.name, actual_version
                );
            }
        }

        // Install injected (--with) runtimes
        for injected in &mut plan.injected {
            if injected.status == InstallStatus::NeedsInstall {
                debug!("[EnsureStage] Installing --with dep: {}", injected.name);
                let version = injected.version_string().map(|s| s.to_string());

                let installed_version = if let Some(ver) = &version {
                    install_mgr
                        .install_runtime_with_version(&injected.name, ver)
                        .await
                        .map_err(|e| EnsureError::InstallFailed {
                            runtime: injected.name.clone(),
                            version: ver.clone(),
                            reason: e.to_string(),
                        })?
                } else {
                    install_mgr
                        .install_runtime(&injected.name)
                        .await
                        .map_err(|e| EnsureError::InstallFailed {
                            runtime: injected.name.clone(),
                            version: "latest".to_string(),
                            reason: e.to_string(),
                        })?
                };

                if let Some(ref actual_version) = installed_version {
                    injected.mark_installed_with_version(actual_version.clone(), None);
                    info!(
                        "[EnsureStage] --with dep {} installed (version: {})",
                        injected.name, actual_version
                    );
                }
            }
        }

        // Re-resolve to get updated executable paths
        // (This mirrors the re-resolve logic in the current executor)
        if let Ok(re_resolved) = self
            .resolver
            .resolve_with_version(&plan.primary.name, plan.primary.version_string())
        {
            if re_resolved.executable.is_absolute() {
                plan.primary.executable = Some(re_resolved.executable);
                plan.primary.status = InstallStatus::Installed;
            }
        }

        debug!(
            "[EnsureStage] Complete. primary={:?}, needs_install={}",
            plan.primary.executable,
            plan.needs_install()
        );

        Ok(plan)
    }
}
