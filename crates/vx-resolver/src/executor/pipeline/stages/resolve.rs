//! Resolve Stage - version resolution and dependency analysis
//!
//! The first stage of the execution pipeline. Transforms a `ResolveRequest`
//! into an `ExecutionPlan` by:
//!
//! 1. Resolving the requested version with priority:
//!    - explicit (command-line) > vx.lock > vx.toml > latest
//! 2. Calling `Resolver::resolve_with_version()` for dependency analysis
//! 3. Checking platform support via the provider registry
//! 4. Mapping `ResolutionResult` into `PlannedRuntime` entries
//!
//! The ResolveStage is intentionally a thin wrapper that delegates to the existing
//! `Resolver`. This ensures backward compatibility while enabling testability.

use std::path::PathBuf;

use async_trait::async_trait;
use tracing::{debug, trace};
use vx_runtime::{ProviderRegistry, RuntimeContext};

use crate::executor::project_config::ProjectToolsConfig;
use crate::{ResolutionCache, ResolutionCacheKey, ResolutionResult, Resolver, ResolverConfig};

use crate::executor::pipeline::error::ResolveError;
use crate::executor::pipeline::plan::{
    ExecutionConfig, ExecutionPlan, InstallStatus, PlannedRuntime, VersionResolution, VersionSource,
};
use crate::executor::pipeline::stage::Stage;

/// Input to the Resolve stage
#[derive(Debug, Clone)]
pub struct ResolveRequest {
    /// The runtime to execute (e.g., "node", "npm", "go", "msvc")
    pub runtime_name: String,

    /// Explicit version constraint (e.g., "20.0.0", "latest")
    pub version: Option<String>,

    /// Executable override (from `runtime::executable` syntax)
    ///
    /// When set, the resolver will look for this executable name instead of
    /// the runtime's default. For example, `msvc::cl` sets this to "cl",
    /// so the resolver searches for `cl.exe` inside the `msvc` store directory.
    pub executable_override: Option<String>,

    /// Command-line arguments to pass to the runtime
    pub args: Vec<String>,

    /// Additional runtimes to inject via `--with`
    pub with_deps: Vec<WithDepRequest>,

    /// Whether to inherit parent environment
    pub inherit_env: bool,

    /// Whether auto-install is enabled
    pub auto_install: bool,

    /// Whether to inherit vx-managed PATH
    pub inherit_vx_path: bool,

    /// Working directory override
    pub working_dir: Option<PathBuf>,
}

/// A `--with` dependency request
#[derive(Debug, Clone)]
pub struct WithDepRequest {
    /// Runtime name
    pub runtime: String,
    /// Optional version constraint
    pub version: Option<String>,
}

impl From<&vx_core::WithDependency> for WithDepRequest {
    fn from(dep: &vx_core::WithDependency) -> Self {
        Self {
            runtime: dep.runtime.clone(),
            version: dep.version.clone(),
        }
    }
}

impl ResolveRequest {
    /// Create a minimal resolve request
    pub fn new(runtime_name: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            runtime_name: runtime_name.into(),
            version: None,
            executable_override: None,
            args,
            with_deps: Vec::new(),
            inherit_env: false,
            auto_install: true,
            inherit_vx_path: true,
            working_dir: None,
        }
    }

    /// Set explicit version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Add `--with` dependencies
    pub fn with_deps(mut self, deps: Vec<WithDepRequest>) -> Self {
        self.with_deps = deps;
        self
    }
}

/// The Resolve stage: `ResolveRequest` → `ExecutionPlan`
///
/// This stage handles:
/// - Version resolution (explicit → project config → latest installed)
/// - Dependency resolution via `Resolver::resolve_with_version()`
/// - Platform support checking
/// - Mapping results to `ExecutionPlan`
///
/// It intentionally delegates to the existing `Resolver` to avoid duplicating logic.
pub struct ResolveStage<'a> {
    /// The underlying resolver for dependency analysis
    resolver: &'a Resolver,

    /// Resolver configuration
    config: &'a ResolverConfig,

    /// Optional disk-backed resolution cache for skipping repeated resolver calls
    resolution_cache: Option<&'a ResolutionCache>,

    /// Optional project config for version fallback
    project_config: Option<&'a ProjectToolsConfig>,

    /// Optional provider registry/runtime context for version-aware deps(version)
    registry: Option<&'a ProviderRegistry>,
    runtime_context: Option<&'a RuntimeContext>,

    /// Optional runtime store base path for scanning installed versions
    #[allow(dead_code)]
    store_base: Option<PathBuf>,
}

impl<'a> ResolveStage<'a> {
    /// Create a new ResolveStage
    pub fn new(resolver: &'a Resolver, config: &'a ResolverConfig) -> Self {
        Self {
            resolver,
            config,
            resolution_cache: None,
            project_config: None,
            registry: None,
            runtime_context: None,
            store_base: None,
        }
    }

    /// Enable the disk-backed resolution cache for this stage
    pub fn with_resolution_cache(mut self, cache: &'a ResolutionCache) -> Self {
        self.resolution_cache = Some(cache);
        self
    }

    /// Set the project configuration for version fallback
    pub fn with_project_config(mut self, config: &'a ProjectToolsConfig) -> Self {
        self.project_config = Some(config);
        self
    }

    /// Enable version-aware dependency lookup via the runtime registry.
    pub fn with_runtime_access(
        mut self,
        registry: &'a ProviderRegistry,
        runtime_context: &'a RuntimeContext,
    ) -> Self {
        self.registry = Some(registry);
        self.runtime_context = Some(runtime_context);
        self
    }

    /// Set the runtime store base path (for resolving "latest" to an installed version)
    pub fn with_store_base(mut self, path: PathBuf) -> Self {
        self.store_base = Some(path);
        self
    }

    /// Resolve version from explicit argument or project config.
    ///
    /// Priority: explicit > vx.lock > vx.toml > None (let resolver decide)
    /// If the result is "latest", try to resolve to the actual latest installed version.
    fn resolve_version(&self, runtime_name: &str, explicit: Option<&str>) -> Option<String> {
        // Pass "latest" through to EnsureStage so that
        // `ensure_version_installed("latest")` → `runtime.resolve_version("latest", ctx)`
        // can resolve it against the cached remote version list (not just local installs).
        // This ensures that if a newer version is available remotely, it will be installed.
        if let Some(v) = explicit {
            Some(v.to_string())
        } else if let Some(project_config) = self.project_config {
            project_config
                .get_version_with_fallback(runtime_name)
                .map(|s| s.to_string())
        } else {
            None
        }
    }

    /// Determine the `VersionSource` based on how the version was obtained
    ///
    /// Priority: explicit > locked > project config > installed latest
    fn determine_source(&self, runtime_name: &str, explicit: Option<&str>) -> VersionSource {
        if explicit.is_some() {
            VersionSource::Explicit
        } else if let Some(project_config) = self.project_config {
            // Check if the version comes from vx.lock
            if project_config.is_locked(runtime_name) {
                VersionSource::Locked
            } else if project_config
                .get_version_with_fallback(runtime_name)
                .is_some()
            {
                VersionSource::ProjectConfig
            } else {
                VersionSource::InstalledLatest
            }
        } else {
            VersionSource::InstalledLatest
        }
    }

    async fn enrich_with_versioned_dependencies(
        &self,
        runtime_name: &str,
        resolved_version: Option<&str>,
        resolution: &mut ResolutionResult,
    ) -> Result<(), ResolveError> {
        let (Some(registry), Some(runtime_context)) = (self.registry, self.runtime_context) else {
            return Ok(());
        };

        let Some(runtime) = registry.get_runtime(runtime_name) else {
            return Ok(());
        };

        let Some(version) = self
            .resolve_dependency_version(
                runtime_name,
                resolved_version,
                resolution,
                runtime.as_ref(),
                runtime_context,
            )
            .await?
        else {
            return Ok(());
        };

        let deps = runtime
            .versioned_dependencies(&version, runtime_context)
            .await
            .map_err(|e| ResolveError::ResolutionFailed {
                runtime: runtime_name.to_string(),
                reason: format!(
                    "failed to resolve version-aware dependencies for {}@{}: {}",
                    runtime_name, version, e
                ),
            })?;

        if deps.is_empty() {
            return Ok(());
        }

        self.resolver.merge_additional_dependencies(
            runtime_name,
            resolution,
            deps.into_iter()
                .map(|dep| self.runtime_dependency_to_resolver(dep, runtime_name)),
        );

        Ok(())
    }

    async fn resolve_dependency_version(
        &self,
        runtime_name: &str,
        resolved_version: Option<&str>,
        resolution: &ResolutionResult,
        runtime: &dyn vx_runtime::Runtime,
        runtime_context: &RuntimeContext,
    ) -> Result<Option<String>, ResolveError> {
        if let Some(version) = resolved_version.filter(|version| !version.is_empty()) {
            return Ok(Some(version.to_string()));
        }

        if !resolution.runtime_needs_install {
            return runtime
                .resolve_installed_version("latest", runtime_context)
                .await
                .map_err(|e| ResolveError::ResolutionFailed {
                    runtime: runtime_name.to_string(),
                    reason: format!(
                        "failed to detect installed version for {}: {}",
                        runtime_name, e
                    ),
                });
        }

        runtime
            .resolve_version("latest", runtime_context)
            .await
            .map(Some)
            .map_err(|e| ResolveError::ResolutionFailed {
                runtime: runtime_name.to_string(),
                reason: format!(
                    "failed to resolve latest version for {}: {}",
                    runtime_name, e
                ),
            })
    }

    fn runtime_dependency_to_resolver(
        &self,
        dep: vx_runtime::RuntimeDependency,
        runtime_name: &str,
    ) -> crate::RuntimeDependency {
        let dep_name = dep.name.clone();
        let reason = dep
            .reason
            .clone()
            .unwrap_or_else(|| format!("{} requires {}", runtime_name, dep_name));

        let mut resolver_dep = if dep.optional {
            crate::RuntimeDependency::optional(dep_name, reason)
        } else {
            crate::RuntimeDependency::required(dep_name, reason)
        };

        if let Some(version_req) = dep
            .version_req
            .as_deref()
            .filter(|version| !version.is_empty() && *version != "*")
        {
            let (min, max) = crate::RuntimeMap::parse_version_bounds(version_req);
            if let Some(min) = min {
                resolver_dep = resolver_dep.with_min_version(min);
            }
            if let Some(max) = max {
                resolver_dep = resolver_dep.with_max_version(max);
            }
        }

        if let Some(min) = dep.min_version {
            resolver_dep = resolver_dep.with_min_version(min);
        }
        if let Some(max) = dep.max_version {
            resolver_dep = resolver_dep.with_max_version(max);
        }
        if let Some(recommended) = dep.recommended_version {
            resolver_dep = resolver_dep.with_recommended_version(recommended);
        }

        resolver_dep
    }

    fn ensure_compatible_dependencies(
        &self,
        resolution: &ResolutionResult,
    ) -> Result<(), ResolveError> {
        if resolution.incompatible_dependencies.is_empty() {
            return Ok(());
        }

        let reasons: Vec<String> = resolution
            .incompatible_dependencies
            .iter()
            .map(|ic| {
                format!(
                    "{}: current={}, recommended={:?}",
                    ic.runtime_name,
                    ic.current_version.as_deref().unwrap_or("?"),
                    ic.recommended_version
                )
            })
            .collect();
        trace!("[ResolveStage] incompatible deps: {:?}", reasons);

        Err(ResolveError::IncompatibleDependencies {
            details: reasons.join("; "),
        })
    }

    /// Map a `ResolutionResult` into an `ExecutionPlan`
    fn build_plan(
        &self,
        request: &ResolveRequest,
        resolution: &ResolutionResult,
        resolved_version: Option<&str>,
        source: VersionSource,
    ) -> ExecutionPlan {
        // Build the primary PlannedRuntime
        let primary = self.build_primary_runtime(resolution, resolved_version, source);

        // Build dependency PlannedRuntimes
        let dependencies = self.build_dependency_runtimes(resolution);

        // Build injected (--with) PlannedRuntimes
        let injected = self.build_injected_runtimes(&request.with_deps);

        // Build execution config
        let config = ExecutionConfig {
            args: request.args.clone(),
            working_dir: request.working_dir.clone(),
            extra_env: std::collections::HashMap::new(),
            inherit_vx_path: request.inherit_vx_path,
            inherit_parent_env: request.inherit_env,
            auto_install: request.auto_install,
            show_progress: true,
        };

        let mut plan = ExecutionPlan::new(primary, config);
        plan.dependencies = dependencies;
        plan.injected = injected;
        plan
    }

    /// Build the primary `PlannedRuntime` from a `ResolutionResult`
    fn build_primary_runtime(
        &self,
        resolution: &ResolutionResult,
        resolved_version: Option<&str>,
        source: VersionSource,
    ) -> PlannedRuntime {
        if resolution.runtime_needs_install {
            // Runtime needs installation
            let version = resolved_version.unwrap_or("latest").to_string();
            PlannedRuntime {
                name: resolution.runtime.clone(),
                version: VersionResolution::NeedsInstall {
                    version: version.clone(),
                },
                status: InstallStatus::NeedsInstall,
                executable: None,
                install_dir: None,
                command_prefix: resolution.command_prefix.clone(),
            }
        } else {
            // Runtime is available
            let exe_path = resolution.executable.clone();
            let version_str = resolved_version.unwrap_or("unknown").to_string();

            PlannedRuntime {
                name: resolution.runtime.clone(),
                version: VersionResolution::Installed {
                    version: version_str,
                    source,
                },
                status: InstallStatus::Installed,
                executable: Some(exe_path),
                install_dir: None,
                command_prefix: resolution.command_prefix.clone(),
            }
        }
    }

    /// Build `PlannedRuntime` entries for missing dependencies from install_order
    ///
    /// For bundled runtimes (e.g., npm bundled with node), when the primary
    /// runtime's version comes from vx.lock via ecosystem fallback, we propagate
    /// that version to the parent dependency. This ensures `vx npm ci` with
    /// `node = "22.22.0"` in vx.lock installs node 22.22.0 — not "latest".
    fn build_dependency_runtimes(&self, resolution: &ResolutionResult) -> Vec<PlannedRuntime> {
        resolution
            .install_order
            .iter()
            .filter(|name| *name != &resolution.runtime)
            .map(|name| {
                // Try to resolve the dependency's version from project config (vx.lock > vx.toml)
                let dep_version = self
                    .project_config
                    .and_then(|pc| pc.get_version_with_fallback(name))
                    .map(|v| v.to_string());

                if resolution.missing_dependencies.contains(name) {
                    if let Some(ver) = dep_version {
                        PlannedRuntime::needs_install(name.clone(), ver)
                    } else {
                        PlannedRuntime {
                            name: name.clone(),
                            version: VersionResolution::Unresolved,
                            status: InstallStatus::NeedsInstall,
                            executable: None,
                            install_dir: None,
                            command_prefix: Vec::new(),
                        }
                    }
                } else {
                    // Dependency is in install_order but not missing — it's available
                    PlannedRuntime {
                        name: name.clone(),
                        version: VersionResolution::Unresolved,
                        status: InstallStatus::Installed,
                        executable: None,
                        install_dir: None,
                        command_prefix: Vec::new(),
                    }
                }
            })
            .collect()
    }

    /// Build `PlannedRuntime` entries for `--with` injected dependencies
    fn build_injected_runtimes(&self, with_deps: &[WithDepRequest]) -> Vec<PlannedRuntime> {
        with_deps
            .iter()
            .map(|dep| {
                if let Some(ref ver) = dep.version {
                    PlannedRuntime::needs_install(dep.runtime.clone(), ver.clone())
                } else {
                    PlannedRuntime {
                        name: dep.runtime.clone(),
                        version: VersionResolution::Unresolved,
                        status: InstallStatus::NeedsInstall,
                        executable: None,
                        install_dir: None,
                        command_prefix: Vec::new(),
                    }
                }
            })
            .collect()
    }
}

#[async_trait]
impl<'a> Stage<ResolveRequest, ExecutionPlan> for ResolveStage<'a> {
    type Error = ResolveError;

    async fn execute(&self, input: ResolveRequest) -> Result<ExecutionPlan, ResolveError> {
        debug!(
            "[ResolveStage] runtime={}, version={:?}, executable_override={:?}",
            input.runtime_name, input.version, input.executable_override
        );

        // Step 1: Resolve version (explicit → project config → latest installed)
        let resolved_version = self.resolve_version(&input.runtime_name, input.version.as_deref());
        let source = self.determine_source(&input.runtime_name, input.version.as_deref());

        debug!(
            "[ResolveStage] resolved_version={:?}, source={:?}",
            resolved_version, source
        );

        // Step 2: Check resolution cache (only when no executable override, since overrides
        // are rare and their cache keys would be harder to invalidate correctly)
        let cache_key = if self.resolution_cache.is_some() && input.executable_override.is_none() {
            Some(ResolutionCacheKey::from_context(
                &input.runtime_name,
                input.version.as_deref(),
                &input.args,
                self.config,
            ))
        } else {
            None
        };

        if let (Some(cache), Some(key)) = (self.resolution_cache, &cache_key)
            && let Some(mut cached) = cache.get(key)
        {
            debug!(
                "[ResolveStage] Resolution cache hit for {}",
                input.runtime_name
            );
            // Run the same platform-support check so cache hits are consistent with misses
            if let Some(unsupported) = cached
                .unsupported_platform_runtimes
                .iter()
                .find(|u| u.is_primary)
            {
                return Err(ResolveError::PlatformNotSupported {
                    runtime: unsupported.runtime_name.clone(),
                    required: unsupported.supported_platforms.clone(),
                    current: unsupported.current_platform.clone(),
                });
            }
            self.enrich_with_versioned_dependencies(
                &input.runtime_name,
                resolved_version.as_deref(),
                &mut cached,
            )
            .await?;
            self.ensure_compatible_dependencies(&cached)?;
            return Ok(self.build_plan(&input, &cached, resolved_version.as_deref(), source));
        }

        // Step 3: Resolve dependencies via the Resolver
        // If an executable override is provided (e.g., msvc::cl), use it for resolution
        let mut resolution = if let Some(ref exe_override) = input.executable_override {
            self.resolver
                .resolve_with_executable(
                    &input.runtime_name,
                    resolved_version.as_deref(),
                    exe_override,
                )
                .map_err(|e| ResolveError::ResolutionFailed {
                    runtime: input.runtime_name.clone(),
                    reason: e.to_string(),
                })?
        } else {
            self.resolver
                .resolve_with_version(&input.runtime_name, resolved_version.as_deref())
                .map_err(|e| ResolveError::ResolutionFailed {
                    runtime: input.runtime_name.clone(),
                    reason: e.to_string(),
                })?
        };

        self.enrich_with_versioned_dependencies(
            &input.runtime_name,
            resolved_version.as_deref(),
            &mut resolution,
        )
        .await?;

        debug!(
            "[ResolveStage] executable={}, needs_install={}, missing_deps={:?}",
            resolution.executable.display(),
            resolution.runtime_needs_install,
            resolution.missing_dependencies
        );

        // Step 3: Check for unsupported platform runtimes
        if !resolution.unsupported_platform_runtimes.is_empty() {
            let primary_unsupported = resolution
                .unsupported_platform_runtimes
                .iter()
                .find(|u| u.is_primary);

            if let Some(unsupported) = primary_unsupported {
                return Err(ResolveError::PlatformNotSupported {
                    runtime: unsupported.runtime_name.clone(),
                    required: unsupported.supported_platforms.clone(),
                    current: unsupported.current_platform.clone(),
                });
            }
        }

        // Step 4: Check for incompatible dependencies
        self.ensure_compatible_dependencies(&resolution)?;

        // Step 5: Cache the resolution result.
        //
        // Only cache results where the primary runtime is already installed and
        // all dependencies are satisfied.  A `NeedsInstall` result is a transient,
        // pre-installation snapshot: once the EnsureStage installs the tool the
        // cached answer would be stale and every subsequent run would still enter
        // the (now fast-path) EnsureStage unnecessarily.
        //
        // By not caching NeedsInstall results we ensure that:
        //   run 1  (tool absent)  → cache miss → resolve → NeedsInstall → install
        //   run 2  (tool present) → cache miss → resolve → Installed    → cache ✅
        //   run 3+ (tool present) → cache hit  → Installed              → skip EnsureStage ✅
        let is_fully_installed =
            !resolution.runtime_needs_install && resolution.missing_dependencies.is_empty();

        if is_fully_installed {
            if let (Some(cache), Some(key)) = (self.resolution_cache, &cache_key) {
                if let Err(e) = cache.set(key, &resolution) {
                    debug!(
                        "[ResolveStage] Failed to write resolution cache for {}: {}",
                        input.runtime_name, e
                    );
                } else {
                    debug!(
                        "[ResolveStage] Resolution cached for {} (Installed)",
                        input.runtime_name
                    );
                }
            }
        } else {
            debug!(
                "[ResolveStage] Skipping cache write for {} — runtime or deps need install",
                input.runtime_name
            );
        }

        // Step 6: Build the ExecutionPlan
        let plan = self.build_plan(&input, &resolution, resolved_version.as_deref(), source);

        debug!(
            "[ResolveStage] plan: primary={}, deps={}, injected={}, needs_install={}",
            plan.primary.name,
            plan.dependencies.len(),
            plan.injected.len(),
            plan.needs_install()
        );

        Ok(plan)
    }
}

/// List installed versions from a runtime store directory (sorted)
#[cfg(test)]
fn list_installed_versions(runtime_dir: &std::path::Path) -> std::io::Result<Vec<String>> {
    if !runtime_dir.exists() {
        return Ok(Vec::new());
    }

    let mut versions: Vec<String> = std::fs::read_dir(runtime_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();

    versions.sort_by(|a, b| {
        // Try semver sort, fall back to string sort
        match (semver::Version::parse(a), semver::Version::parse(b)) {
            (Ok(va), Ok(vb)) => va.cmp(&vb),
            _ => a.cmp(b),
        }
    });

    Ok(versions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ResolverConfig, RuntimeMap};
    use std::path::PathBuf;

    fn test_resolver() -> Resolver {
        let config = ResolverConfig::default();
        let runtime_map = RuntimeMap::empty();
        Resolver::new(config, runtime_map).unwrap()
    }

    #[test]
    fn test_resolve_request_new() {
        let req = ResolveRequest::new("node", vec!["--version".into()]);
        assert_eq!(req.runtime_name, "node");
        assert!(req.version.is_none());
        assert_eq!(req.args, vec!["--version"]);
        assert!(req.auto_install);
        assert!(req.with_deps.is_empty());
    }

    #[test]
    fn test_resolve_request_with_version() {
        let req = ResolveRequest::new("node", vec![]).with_version("20.0.0");
        assert_eq!(req.version, Some("20.0.0".to_string()));
    }

    #[test]
    fn test_resolve_request_with_deps() {
        let deps = vec![
            WithDepRequest {
                runtime: "bun".to_string(),
                version: Some("1.0.0".to_string()),
            },
            WithDepRequest {
                runtime: "deno".to_string(),
                version: None,
            },
        ];
        let req = ResolveRequest::new("node", vec![]).with_deps(deps);
        assert_eq!(req.with_deps.len(), 2);
        assert_eq!(req.with_deps[0].runtime, "bun");
        assert_eq!(req.with_deps[1].version, None);
    }

    #[test]
    fn test_with_dep_request_from_core() {
        let core_dep = vx_core::WithDependency::new("bun", Some("1.0.0".to_string()));
        let req: WithDepRequest = (&core_dep).into();
        assert_eq!(req.runtime, "bun");
        assert_eq!(req.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_resolve_stage_creation() {
        let resolver = test_resolver();
        let config = ResolverConfig::default();
        let stage = ResolveStage::new(&resolver, &config);
        assert!(stage.project_config.is_none());
        assert!(stage.store_base.is_none());
    }

    #[test]
    fn test_resolve_stage_with_store_base() {
        let resolver = test_resolver();
        let config = ResolverConfig::default();
        let stage =
            ResolveStage::new(&resolver, &config).with_store_base(PathBuf::from("/tmp/store"));
        assert_eq!(stage.store_base, Some(PathBuf::from("/tmp/store")));
    }

    #[test]
    fn test_resolve_version_explicit() {
        let resolver = test_resolver();
        let config = ResolverConfig::default();
        let stage = ResolveStage::new(&resolver, &config);

        let version = stage.resolve_version("node", Some("20.0.0"));
        assert_eq!(version, Some("20.0.0".to_string()));
    }

    #[test]
    fn test_resolve_version_none() {
        let resolver = test_resolver();
        let config = ResolverConfig::default();
        let stage = ResolveStage::new(&resolver, &config);

        let version = stage.resolve_version("node", None);
        assert_eq!(version, None);
    }

    #[test]
    fn test_determine_source_explicit() {
        let resolver = test_resolver();
        let config = ResolverConfig::default();
        let stage = ResolveStage::new(&resolver, &config);

        let source = stage.determine_source("node", Some("20.0.0"));
        assert_eq!(source, VersionSource::Explicit);
    }

    #[test]
    fn test_determine_source_no_version() {
        let resolver = test_resolver();
        let config = ResolverConfig::default();
        let stage = ResolveStage::new(&resolver, &config);

        let source = stage.determine_source("node", None);
        assert_eq!(source, VersionSource::InstalledLatest);
    }

    #[test]
    fn test_build_injected_runtimes_with_version() {
        let resolver = test_resolver();
        let config = ResolverConfig::default();
        let stage = ResolveStage::new(&resolver, &config);

        let deps = vec![WithDepRequest {
            runtime: "bun".to_string(),
            version: Some("1.0.0".to_string()),
        }];

        let runtimes = stage.build_injected_runtimes(&deps);
        assert_eq!(runtimes.len(), 1);
        assert_eq!(runtimes[0].name, "bun");
        assert_eq!(runtimes[0].version_string(), Some("1.0.0"));
        assert_eq!(runtimes[0].status, InstallStatus::NeedsInstall);
    }

    #[test]
    fn test_build_injected_runtimes_no_version() {
        let resolver = test_resolver();
        let config = ResolverConfig::default();
        let stage = ResolveStage::new(&resolver, &config);

        let deps = vec![WithDepRequest {
            runtime: "deno".to_string(),
            version: None,
        }];

        let runtimes = stage.build_injected_runtimes(&deps);
        assert_eq!(runtimes.len(), 1);
        assert_eq!(runtimes[0].name, "deno");
        assert_eq!(runtimes[0].status, InstallStatus::NeedsInstall);
    }

    #[test]
    fn test_build_primary_runtime_needs_install() {
        let resolver = test_resolver();
        let config = ResolverConfig::default();
        let stage = ResolveStage::new(&resolver, &config);

        let resolution = ResolutionResult {
            runtime: "node".to_string(),
            executable: PathBuf::from("node"),
            command_prefix: vec![],
            missing_dependencies: vec![],
            install_order: vec!["node".to_string()],
            runtime_needs_install: true,
            incompatible_dependencies: vec![],
            dependency_requirements: vec![],
            unsupported_platform_runtimes: vec![],
        };

        let primary =
            stage.build_primary_runtime(&resolution, Some("20.0.0"), VersionSource::Explicit);
        assert_eq!(primary.name, "node");
        assert_eq!(primary.status, InstallStatus::NeedsInstall);
        assert_eq!(primary.version_string(), Some("20.0.0"));
        assert!(primary.executable.is_none());
    }

    #[test]
    fn test_build_primary_runtime_installed() {
        let resolver = test_resolver();
        let config = ResolverConfig::default();
        let stage = ResolveStage::new(&resolver, &config);

        let resolution = ResolutionResult {
            runtime: "node".to_string(),
            executable: PathBuf::from("/home/user/.vx/store/node/20.0.0/bin/node"),
            command_prefix: vec![],
            missing_dependencies: vec![],
            install_order: vec![],
            runtime_needs_install: false,
            incompatible_dependencies: vec![],
            dependency_requirements: vec![],
            unsupported_platform_runtimes: vec![],
        };

        let primary =
            stage.build_primary_runtime(&resolution, Some("20.0.0"), VersionSource::ProjectConfig);
        assert_eq!(primary.name, "node");
        assert_eq!(primary.status, InstallStatus::Installed);
        assert!(primary.executable.is_some());
        assert_eq!(
            primary.version,
            VersionResolution::Installed {
                version: "20.0.0".to_string(),
                source: VersionSource::ProjectConfig,
            }
        );
    }

    #[test]
    fn test_build_dependency_runtimes() {
        let resolver = test_resolver();
        let config = ResolverConfig::default();
        let stage = ResolveStage::new(&resolver, &config);

        let resolution = ResolutionResult {
            runtime: "npm".to_string(),
            executable: PathBuf::from("npm"),
            command_prefix: vec![],
            missing_dependencies: vec!["node".to_string()],
            install_order: vec!["node".to_string(), "npm".to_string()],
            runtime_needs_install: true,
            incompatible_dependencies: vec![],
            dependency_requirements: vec![],
            unsupported_platform_runtimes: vec![],
        };

        let deps = stage.build_dependency_runtimes(&resolution);
        assert_eq!(deps.len(), 1); // "npm" is filtered out (same as primary)
        assert_eq!(deps[0].name, "node");
        assert_eq!(deps[0].status, InstallStatus::NeedsInstall);
    }

    #[test]
    fn test_list_installed_versions_nonexistent_dir() {
        let versions = list_installed_versions(&PathBuf::from("/nonexistent/path")).unwrap();
        assert!(versions.is_empty());
    }

    #[tokio::test]
    async fn test_resolve_stage_unknown_runtime() {
        let resolver = test_resolver();
        let config = ResolverConfig::default();
        let stage = ResolveStage::new(&resolver, &config);

        let request = ResolveRequest::new("completely-unknown-xyz", vec![]);

        // With an empty RuntimeMap, the resolver should still return a result
        // (the runtime just won't be found as installed)
        let result = stage.execute(request).await;

        // The resolver returns Ok with runtime_needs_install=true for unknown runtimes
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.primary.name, "completely-unknown-xyz");
    }

    #[tokio::test]
    async fn test_resolve_stage_with_explicit_version() {
        let resolver = test_resolver();
        let config = ResolverConfig::default();
        let stage = ResolveStage::new(&resolver, &config);

        let request = ResolveRequest::new("node", vec!["--version".into()]).with_version("20.0.0");

        let result = stage.execute(request).await;
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.primary.name, "node");
        assert_eq!(plan.config.args, vec!["--version"]);
    }

    #[tokio::test]
    async fn test_resolve_stage_with_injected_deps() {
        let resolver = test_resolver();
        let config = ResolverConfig::default();
        let stage = ResolveStage::new(&resolver, &config);

        let request = ResolveRequest::new("node", vec![]).with_deps(vec![
            WithDepRequest {
                runtime: "bun".to_string(),
                version: Some("1.0.0".to_string()),
            },
            WithDepRequest {
                runtime: "deno".to_string(),
                version: None,
            },
        ]);

        let result = stage.execute(request).await;
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.injected.len(), 2);
        assert_eq!(plan.injected[0].name, "bun");
        assert_eq!(plan.injected[1].name, "deno");
    }

    #[tokio::test]
    async fn test_resolve_stage_config_propagation() {
        let resolver = test_resolver();
        let config = ResolverConfig::default();
        let stage = ResolveStage::new(&resolver, &config);

        let mut request = ResolveRequest::new("node", vec!["script.js".into()]);
        request.inherit_env = true;
        request.auto_install = false;
        request.inherit_vx_path = false;
        request.working_dir = Some(PathBuf::from("/tmp/project"));

        let result = stage.execute(request).await;
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.config.args, vec!["script.js"]);
        assert!(plan.config.inherit_parent_env);
        assert!(!plan.config.auto_install);
        assert!(!plan.config.inherit_vx_path);
        assert_eq!(plan.config.working_dir, Some(PathBuf::from("/tmp/project")));
    }

    // =============================================================================
    // VersionSource::Locked tests
    // =============================================================================

    #[test]
    fn test_determine_source_locked() {
        use crate::executor::project_config::ProjectToolsConfig;

        let resolver = test_resolver();
        let config = ResolverConfig::default();

        // Create a ProjectToolsConfig with locked version
        let project_config = ProjectToolsConfig::from_tools_with_locked(
            std::collections::HashMap::from([("node".to_string(), "22".to_string())]),
            std::collections::HashMap::from([("node".to_string(), "20.18.0".to_string())]),
        );

        let stage = ResolveStage::new(&resolver, &config).with_project_config(&project_config);

        // When no explicit version and tool is locked, source should be Locked
        let source = stage.determine_source("node", None);
        assert_eq!(source, VersionSource::Locked);
    }

    #[test]
    fn test_determine_source_project_config_when_not_locked() {
        use crate::executor::project_config::ProjectToolsConfig;

        let resolver = test_resolver();
        let config = ResolverConfig::default();

        // Create a ProjectToolsConfig with only vx.toml version (no lock)
        let project_config = ProjectToolsConfig::from_tools(std::collections::HashMap::from([(
            "node".to_string(),
            "22".to_string(),
        )]));

        let stage = ResolveStage::new(&resolver, &config).with_project_config(&project_config);

        // When no explicit version and tool is in config but not locked
        let source = stage.determine_source("node", None);
        assert_eq!(source, VersionSource::ProjectConfig);
    }

    #[test]
    fn test_determine_source_priority() {
        use crate::executor::project_config::ProjectToolsConfig;

        let resolver = test_resolver();
        let config = ResolverConfig::default();

        // Create a ProjectToolsConfig with both locked and config versions
        let project_config = ProjectToolsConfig::from_tools_with_locked(
            std::collections::HashMap::from([
                ("node".to_string(), "22".to_string()), // vx.toml
                ("go".to_string(), "1.21".to_string()), // vx.toml only
            ]),
            std::collections::HashMap::from([
                ("node".to_string(), "20.18.0".to_string()), // vx.lock
            ]),
        );

        let stage = ResolveStage::new(&resolver, &config).with_project_config(&project_config);

        // Explicit takes highest priority
        assert_eq!(
            stage.determine_source("node", Some("18.0.0")),
            VersionSource::Explicit
        );

        // Locked takes priority over config
        assert_eq!(stage.determine_source("node", None), VersionSource::Locked);

        // Config when not locked
        assert_eq!(
            stage.determine_source("go", None),
            VersionSource::ProjectConfig
        );

        // InstalledLatest when not in config or lock
        assert_eq!(
            stage.determine_source("unknown", None),
            VersionSource::InstalledLatest
        );
    }
}
