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

                if installed_version.is_some() {
                    dep.status = InstallStatus::Installed;
                    info!("[EnsureStage] Dependency {} installed", dep.name);
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

            if installed_version.is_some() {
                plan.primary.status = InstallStatus::Installed;
                info!("[EnsureStage] Primary {} installed", plan.primary.name);
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

                if installed_version.is_some() {
                    injected.status = InstallStatus::Installed;
                    info!("[EnsureStage] --with dep {} installed", injected.name);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::pipeline::plan::{ExecutionConfig, PlannedRuntime};
    use std::path::PathBuf;

    #[test]
    fn test_ensure_stage_creation() {
        let config = ResolverConfig::default();
        let runtime_map = crate::RuntimeMap::empty();
        let resolver = Resolver::new(config.clone(), runtime_map).unwrap();
        let stage = EnsureStage::new(&resolver, &config, None, None);
        assert!(stage.registry.is_none());
        assert!(stage.context.is_none());
    }

    #[tokio::test]
    async fn test_ensure_stage_already_installed() {
        let config = ResolverConfig::default();
        let runtime_map = crate::RuntimeMap::empty();
        let resolver = Resolver::new(config.clone(), runtime_map).unwrap();
        let stage = EnsureStage::new(&resolver, &config, None, None);

        let primary = PlannedRuntime::installed(
            "node",
            "20.0.0".to_string(),
            PathBuf::from("/usr/local/bin/node"),
        );
        let plan = ExecutionPlan::new(primary, ExecutionConfig::default());

        let result = stage.execute(plan).await;
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(!plan.needs_install());
    }

    #[tokio::test]
    async fn test_ensure_stage_auto_install_disabled() {
        let config = ResolverConfig::default();
        let runtime_map = crate::RuntimeMap::empty();
        let resolver = Resolver::new(config.clone(), runtime_map).unwrap();
        let stage = EnsureStage::new(&resolver, &config, None, None);

        let primary = PlannedRuntime::needs_install("node", "20.0.0".to_string());
        let mut exec_config = ExecutionConfig::default();
        exec_config.auto_install = false;
        let plan = ExecutionPlan::new(primary, exec_config);

        let result = stage.execute(plan).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, EnsureError::AutoInstallDisabled { .. }));
    }

    #[tokio::test]
    async fn test_ensure_stage_platform_unsupported_logged() {
        let config = ResolverConfig::default();
        let runtime_map = crate::RuntimeMap::empty();
        let resolver = Resolver::new(config.clone(), runtime_map).unwrap();
        let stage = EnsureStage::new(&resolver, &config, None, None);

        let primary = PlannedRuntime::installed(
            "node",
            "20.0.0".to_string(),
            PathBuf::from("/usr/local/bin/node"),
        );
        let unsupported = PlannedRuntime::unsupported("msvc", "Windows only".to_string());
        let plan =
            ExecutionPlan::new(primary, ExecutionConfig::default()).with_injected(unsupported);

        // Should succeed (unsupported injected dep is just a warning)
        let result = stage.execute(plan).await;
        assert!(result.is_ok());
    }
}
