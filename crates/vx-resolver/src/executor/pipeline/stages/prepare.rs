//! Prepare Stage - environment preparation
//!
//! The third stage of the execution pipeline. Takes an `ExecutionPlan` (with all
//! runtimes installed) and produces a `PreparedExecution` ready to be spawned.
//!
//! This stage handles:
//! - Environment variable preparation via `EnvironmentManager`
//! - `--with` dependency PATH injection
//! - Proxy execution setup (RFC 0028) for bundled runtimes
//! - Executable path verification

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use tracing::{debug, info};

use crate::executor::environment::EnvironmentManager;
use crate::executor::pipeline::error::PrepareError;
use crate::executor::pipeline::plan::ExecutionPlan;
use crate::executor::pipeline::stage::Stage;
use crate::executor::project_config::ProjectToolsConfig;
use crate::{Resolver, ResolverConfig};
use vx_runtime::{ProviderRegistry, RuntimeContext};

/// The output of the Prepare stage — a fully-prepared command ready to execute
#[derive(Debug, Clone)]
pub struct PreparedExecution {
    /// The resolved executable path
    pub executable: PathBuf,

    /// Command prefix args (before user args)
    pub command_prefix: Vec<String>,

    /// User-provided arguments
    pub args: Vec<String>,

    /// Complete environment variables for the subprocess
    pub env: HashMap<String, String>,

    /// Whether to inherit vx-managed PATH
    pub inherit_vx_path: bool,

    /// Optional vx tools PATH string
    pub vx_tools_path: Option<String>,

    /// Working directory
    pub working_dir: Option<PathBuf>,

    /// The original plan (for reference by ExecuteStage)
    pub plan: ExecutionPlan,
}

/// The Prepare stage: `ExecutionPlan` → `PreparedExecution`
///
/// Prepares the environment, injects `--with` dependencies into PATH,
/// and verifies the executable is ready.
pub struct PrepareStage<'a> {
    /// Resolver for environment preparation
    resolver: &'a Resolver,

    /// Resolver config
    config: &'a ResolverConfig,

    /// Provider registry
    registry: Option<&'a ProviderRegistry>,

    /// Runtime context
    context: Option<&'a RuntimeContext>,

    /// Project config
    project_config: Option<&'a ProjectToolsConfig>,
}

impl<'a> PrepareStage<'a> {
    /// Create a new PrepareStage
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
            project_config: None,
        }
    }

    /// Set project configuration
    pub fn with_project_config(mut self, config: &'a ProjectToolsConfig) -> Self {
        self.project_config = Some(config);
        self
    }

    /// Create an environment manager
    fn environment_manager(&self) -> EnvironmentManager<'_> {
        EnvironmentManager::new(
            self.config,
            self.resolver,
            self.registry,
            self.context,
            self.project_config,
        )
    }

    /// Try proxy execution for bundled runtimes (RFC 0028)
    ///
    /// For runtimes that are bundled with another runtime (e.g., msbuild with dotnet),
    /// the executable is not directly available. Instead, we call the runtime's
    /// `prepare_execution()` method which returns the proxy executable and command prefix.
    async fn try_proxy_execution(
        &self,
        plan: &ExecutionPlan,
        runtime_env: &HashMap<String, String>,
    ) -> Result<Option<(PathBuf, Vec<String>)>, PrepareError> {
        let registry = match self.registry {
            Some(r) => r,
            None => return Ok(None),
        };

        let runtime = match registry.get_runtime(&plan.primary.name) {
            Some(r) => r,
            None => return Ok(None),
        };

        let version_str = plan
            .primary
            .version_string()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "latest".to_string());

        // Only apply proxy for non-installable (bundled) runtimes
        if runtime.is_version_installable(&version_str) {
            return Ok(None);
        }

        debug!(
            "[PrepareStage] Attempting proxy execution for bundled runtime {}@{}",
            plan.primary.name, version_str
        );

        let exec_ctx = vx_runtime::ExecutionContext {
            working_dir: std::env::current_dir().ok(),
            env: runtime_env.clone(),
            capture_output: false,
            timeout: self.config.execution_timeout,
            executor: Arc::new(vx_runtime::RealCommandExecutor),
        };

        match runtime.prepare_execution(&version_str, &exec_ctx).await {
            Ok(prep) => {
                if let Some(ref msg) = prep.message {
                    info!("{}", msg);
                }

                if let Some(exe) = prep.executable_override {
                    debug!(
                        "[PrepareStage] Proxy resolved: executable={}, prefix={:?}",
                        exe.display(),
                        prep.command_prefix
                    );
                    Ok(Some((exe, prep.command_prefix)))
                } else if prep.use_system_path {
                    // Try system PATH
                    if let Ok(system_exe) = which::which(&plan.primary.name) {
                        debug!(
                            "[PrepareStage] Using system executable: {}",
                            system_exe.display()
                        );
                        Ok(Some((system_exe, prep.command_prefix)))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                debug!(
                    "[PrepareStage] Proxy execution failed for {}: {}",
                    plan.primary.name, e
                );
                // Return ProxyNotAvailable error with context
                Err(PrepareError::ProxyNotAvailable {
                    runtime: plan.primary.name.clone(),
                    proxy: "bundled".to_string(),
                    reason: e.to_string(),
                })
            }
        }
    }
}

#[async_trait]
impl<'a> Stage<ExecutionPlan, PreparedExecution> for PrepareStage<'a> {
    type Error = PrepareError;

    async fn execute(&self, plan: ExecutionPlan) -> Result<PreparedExecution, PrepareError> {
        debug!("[PrepareStage] Preparing execution for {}", plan.primary.name);

        // Step 1: Prepare environment variables (needed before proxy execution)
        let version = plan.primary.version_string().map(|s| s.to_string());
        let env_mgr = self.environment_manager();
        let runtime_env = env_mgr
            .prepare_runtime_environment(
                &plan.primary.name,
                version.as_deref(),
                plan.config.inherit_parent_env,
            )
            .await
            .map_err(|e| PrepareError::EnvironmentFailed {
                runtime: plan.primary.name.clone(),
                reason: e.to_string(),
            })?;

        debug!(
            "[PrepareStage] Environment prepared: {} variables",
            runtime_env.len()
        );

        // Step 2: Resolve executable — try direct path first, then proxy execution (RFC 0028)
        let (executable, command_prefix) = if let Some(exe) = plan.primary.executable.clone() {
            // Executable already resolved (normal runtimes)
            (exe, Vec::new())
        } else {
            // No executable path — this is expected for bundled runtimes (e.g., msbuild).
            // Try proxy execution: the runtime's prepare_execution() can provide
            // an executable override (e.g., msbuild → dotnet msbuild).
            debug!(
                "[PrepareStage] No executable for {}, trying proxy execution (RFC 0028)",
                plan.primary.name
            );
            self.try_proxy_execution(&plan, &runtime_env)
                .await?
                .ok_or_else(|| {
                    // Distinguish between unknown runtime and known-but-no-executable
                    if let Some(registry) = self.registry {
                        if registry.get_runtime(&plan.primary.name).is_none() {
                            return PrepareError::UnknownRuntime {
                                runtime: plan.primary.name.clone(),
                            };
                        }
                    }
                    PrepareError::NoExecutable {
                        runtime: plan.primary.name.clone(),
                    }
                })?
        };

        // Step 3: Build vx tools PATH
        let vx_tools_path = if plan.config.inherit_vx_path {
            let env_mgr = self.environment_manager();
            env_mgr.build_vx_tools_path()
        } else {
            None
        };

        Ok(PreparedExecution {
            executable,
            command_prefix,
            args: plan.config.args.clone(),
            env: runtime_env,
            inherit_vx_path: plan.config.inherit_vx_path,
            vx_tools_path,
            working_dir: plan.config.working_dir.clone(),
            plan,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::pipeline::plan::{ExecutionConfig, PlannedRuntime};

    #[test]
    fn test_prepared_execution_fields() {
        let prepared = PreparedExecution {
            executable: PathBuf::from("/usr/local/bin/node"),
            command_prefix: vec![],
            args: vec!["--version".to_string()],
            env: HashMap::new(),
            inherit_vx_path: true,
            vx_tools_path: None,
            working_dir: None,
            plan: ExecutionPlan::new(
                PlannedRuntime::installed(
                    "node",
                    "20.0.0".to_string(),
                    PathBuf::from("/usr/local/bin/node"),
                ),
                ExecutionConfig::default(),
            ),
        };

        assert_eq!(prepared.executable, PathBuf::from("/usr/local/bin/node"));
        assert_eq!(prepared.args, vec!["--version"]);
    }

    #[tokio::test]
    async fn test_prepare_stage_no_executable() {
        let config = ResolverConfig::default();
        let runtime_map = crate::RuntimeMap::empty();
        let resolver = Resolver::new(config.clone(), runtime_map).unwrap();
        let stage = PrepareStage::new(&resolver, &config, None, None);

        // Primary has no executable path
        let primary = PlannedRuntime::needs_install("node", "20.0.0".to_string());
        let plan = ExecutionPlan::new(primary, ExecutionConfig::default());

        let result = stage.execute(plan).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PrepareError::NoExecutable { .. }
        ));
    }
}
