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
use crate::executor::pipeline::plan::{ExecutionPlan, VersionResolution};
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

    /// Optional output filter config (compact mode).
    /// Passed through from `ExecutionConfig::output_filter`.
    pub output_filter: Option<vx_output_filter::OutputFilterConfig>,
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

    fn resolve_system_executable(
        runtime: &dyn vx_runtime::Runtime,
        runtime_env: &HashMap<String, String>,
    ) -> Option<PathBuf> {
        let executable_name = runtime.executable_name();
        let working_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let runtime_path = runtime_env
            .iter()
            .find_map(|(key, value)| key.eq_ignore_ascii_case("PATH").then_some(value));

        if let Some(path) = runtime_path
            && let Ok(system_exe) = which::which_in(executable_name, Some(path), &working_dir)
        {
            return Some(system_exe);
        }

        which::which(executable_name).ok()
    }

    /// Ask the runtime for execution-specific preparation (RFC 0028).
    ///
    /// For runtimes that are bundled with another runtime (e.g., msbuild with dotnet),
    /// the executable is not directly available. Instead, we call the runtime's
    /// `prepare_execution()` method which returns the proxy executable, extra
    /// environment variables, and command prefix.
    async fn try_execution_preparation(
        &self,
        plan: &ExecutionPlan,
        runtime_env: &HashMap<String, String>,
    ) -> Result<Option<vx_runtime::ExecutionPrep>, PrepareError> {
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

        debug!(
            "[PrepareStage] Attempting proxy/system-path execution for runtime {}@{}",
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
            Ok(mut prep) => {
                if let Some(ref msg) = prep.message {
                    info!("{}", msg);
                }

                if let Some(ref exe) = prep.executable_override {
                    debug!(
                        "[PrepareStage] Proxy resolved: executable={}, prefix={:?}",
                        exe.display(),
                        prep.command_prefix
                    );
                    Ok(Some(prep))
                } else if prep.use_system_path {
                    // Try system PATH using the runtime's actual executable name.
                    if let Some(system_exe) =
                        Self::resolve_system_executable(runtime.as_ref(), runtime_env)
                    {
                        debug!(
                            "[PrepareStage] Using system executable: {}",
                            system_exe.display()
                        );
                        prep.executable_override = Some(system_exe);
                        Ok(Some(prep))
                    } else {
                        Ok(None)
                    }
                } else if prep.proxy_ready
                    || !prep.env_vars.is_empty()
                    || !prep.path_prepend.is_empty()
                    || !prep.command_prefix.is_empty()
                {
                    Ok(Some(prep))
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
        debug!(
            "[PrepareStage] Preparing execution for {}",
            plan.primary.name
        );

        // Step 1: Prepare environment variables (needed before proxy execution)
        let version = plan.primary.version_string().map(|s| s.to_string());
        let env_mgr = self.environment_manager();

        // A `SystemAvailable` primary is owned by an external toolchain manager
        // (rustup, via `rust-toolchain.toml` / `RUSTUP_TOOLCHAIN`). Injecting the
        // provider environment here would overwrite `RUSTUP_HOME` / `CARGO_HOME`
        // with vx's store paths and silently redirect the command to vx's toolchain
        // — the exact override the pin exists to prevent.
        //
        // Passing an empty map is safe: the child inherits the parent environment
        // and only the entries we set explicitly are overridden.
        let mut runtime_env = if matches!(
            plan.primary.version,
            VersionResolution::SystemAvailable { .. }
        ) {
            debug!(
                "[PrepareStage] {} is externally managed; not injecting vx environment",
                plan.primary.name
            );
            HashMap::new()
        } else {
            env_mgr
                .prepare_runtime_environment(
                    &plan.primary.name,
                    version.as_deref(),
                    plan.config.inherit_parent_env,
                )
                .await
                .map_err(|e| PrepareError::EnvironmentFailed {
                    runtime: plan.primary.name.clone(),
                    reason: e.to_string(),
                })?
        };

        debug!(
            "[PrepareStage] Environment prepared: {} variables",
            runtime_env.len()
        );

        // Step 2: Allow the runtime to replace a direct executable and augment
        // the environment before falling back to the resolved path. This is
        // required by providers whose installed payload must be activated
        // before its executable becomes meaningful (for example Rez bundles).
        let execution_prep = self.try_execution_preparation(&plan, &runtime_env).await?;
        let mut executable_override = None;
        let mut prepared_command_prefix = None;
        if let Some(prep) = execution_prep {
            runtime_env.extend(prep.env_vars);
            if !prep.path_prepend.is_empty() {
                prepend_environment_path(&mut runtime_env, &prep.path_prepend).map_err(
                    |reason| PrepareError::EnvironmentFailed {
                        runtime: plan.primary.name.clone(),
                        reason,
                    },
                )?;
            }
            executable_override = prep.executable_override;
            prepared_command_prefix = Some(prep.command_prefix);
        }

        // Step 3: Resolve executable — direct path first, then proxy execution (RFC 0028)
        let (executable, command_prefix) = if let Some(executable) = executable_override {
            (
                executable,
                prepared_command_prefix.unwrap_or_else(|| plan.primary.command_prefix.clone()),
            )
        } else if let Some(exe) = plan.primary.executable.clone() {
            // Safety net: verify the executable filename matches the requested runtime.
            // This prevents silent misresolution where a bundled tool (npm) gets the
            // parent runtime's binary (node), which would execute `node ci` instead of `npm ci`.
            let exe_stem = exe.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let runtime_name = &plan.primary.name;

            if exe_stem.eq_ignore_ascii_case(runtime_name) {
                // Normal case: executable matches runtime name
                (exe, plan.primary.command_prefix.clone())
            } else if !plan.primary.command_prefix.is_empty() {
                // Command prefix runtimes (e.g., bunx -> bun x) are expected to have
                // a different executable name, so this is fine.
                debug!(
                    "[PrepareStage] Executable {} has prefix {:?} for runtime {}",
                    exe.display(),
                    plan.primary.command_prefix,
                    runtime_name
                );
                (exe, plan.primary.command_prefix.clone())
            } else {
                // Mismatch without command_prefix.
                //
                // Two sub-cases:
                //
                // A) `exe` is an absolute path that was found by the resolver
                //    (e.g. `vx msvc::ildasm` resolved to `ildasm.exe` via _ILDASM_PATHS).
                //    The path is correct — use it directly.
                //
                // B) `exe` is a bare/relative name (resolver couldn't find the binary).
                //    This means the executable_override doesn't exist.  Report a clear
                //    error instead of silently falling back to the parent runtime's binary
                //    (which would execute `cl.exe --help` instead of `sigtools --help`).
                if exe.is_absolute() {
                    // Case A: resolver found the right binary — trust it.
                    tracing::debug!(
                        "[PrepareStage] Executable mismatch but path is absolute: {} (stem={}) for runtime {}, using directly",
                        exe.display(),
                        exe_stem,
                        runtime_name
                    );
                    (exe, plan.primary.command_prefix.clone())
                } else {
                    // Case B: bare name — executable_override was not found.
                    tracing::warn!(
                        "[PrepareStage] Executable override '{}' not found for runtime {}",
                        exe_stem,
                        runtime_name
                    );
                    return Err(PrepareError::NoExecutable {
                        runtime: format!("{}::{}", runtime_name, exe_stem),
                    });
                }
            }
        } else {
            // No executable path — this is expected for bundled runtimes (e.g., msbuild).
            // Execution preparation already ran in step 2 and supplied an override
            // when the runtime had one; distinguish the two failure modes here.
            if let Some(registry) = self.registry
                && registry.get_runtime(&plan.primary.name).is_none()
            {
                return Err(PrepareError::UnknownRuntime {
                    runtime: plan.primary.name.clone(),
                });
            }
            return Err(PrepareError::NoExecutable {
                runtime: plan.primary.name.clone(),
            });
        };

        // Step 3: Build vx tools PATH
        //
        // Runtimes delegated to an external toolchain manager are excluded: a
        // `~/.vx/store/rust/*/cargo/bin` left over from an earlier vx-managed install
        // would otherwise shadow the rustup toolchain for every child process.
        let vx_tools_path = if plan.config.inherit_vx_path {
            let env_mgr = self.environment_manager();
            env_mgr.build_vx_tools_path_excluding(&plan.config.delegated_to_system)
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
            output_filter: plan.config.output_filter.clone(),
            plan,
        })
    }
}

/// Prepend `entries` to the `PATH` variable of `environment`.
///
/// This mirrors how a Rez package prepends its payload directories, so an
/// activated bundle wins over any earlier entry without discarding it.
fn prepend_environment_path(
    environment: &mut HashMap<String, String>,
    entries: &[PathBuf],
) -> Result<(), String> {
    if entries.is_empty() {
        return Ok(());
    }

    let key = environment
        .keys()
        .find(|key| key.eq_ignore_ascii_case("PATH"))
        .cloned();
    let separator = if cfg!(windows) { ";" } else { ":" };
    let prepended = entries
        .iter()
        .map(|entry| {
            entry
                .to_str()
                .map(str::to_string)
                .ok_or_else(|| format!("non-UTF-8 PATH entry: {}", entry.display()))
        })
        .collect::<Result<Vec<_>, String>>()?
        .join(separator);

    let (key, value) = match key {
        Some(ref key) => match environment.remove(key) {
            Some(existing) if !existing.is_empty() => {
                (key.clone(), format!("{prepended}{separator}{existing}"))
            }
            _ => (key.clone(), prepended),
        },
        None => ("PATH".to_string(), prepended),
    };
    environment.insert(key, value);
    Ok(())
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
            output_filter: None,
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
