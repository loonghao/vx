//! Executor - the core command forwarding engine
//!
//! This module implements the main execution logic:
//! 1. Resolve runtime and dependencies
//! 2. Auto-install missing components
//! 3. Forward command to the appropriate executable
//!
//! ## Project Configuration Support
//!
//! When a `vx.toml` file is found in the current directory or parent directories,
//! the executor will prioritize using the tool versions specified in the project
//! configuration when building the subprocess PATH. This ensures that:
//!
//! - Subprocesses use the same tool versions as defined in `vx.toml`
//! - Environment isolation is maintained per-project
//! - No global pollution from globally installed tool versions

use crate::{ResolutionCache, Resolver, ResolverConfig, Result, RuntimeMap};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::process::{ExitStatus, Stdio};
use std::sync::Mutex;
use tokio::process::Command;
use tracing::{debug, info, info_span, trace, warn};
use vx_config::parse_config;
use vx_console::ProgressSpinner;
use vx_paths::find_config_file_upward;
use vx_paths::project::{find_vx_config, PROJECT_VX_DIR};
use vx_runtime::{CacheMode, ProviderRegistry, RuntimeContext};

// Re-export from vx_core for convenience
pub use vx_core::{exit_code_from_status, is_ctrl_c_exit};

/// Track which tools have been warned about missing versions
/// This prevents duplicate warnings when building PATH
static WARNED_TOOLS: Mutex<Option<HashSet<String>>> = Mutex::new(None);

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
    /// This stores the tool versions specified in the project configuration.
    project_config: Option<ProjectToolsConfig>,
}

/// Project tools configuration extracted from vx.toml
#[derive(Debug, Clone)]
struct ProjectToolsConfig {
    /// Tool versions from vx.toml (tool_name -> version)
    tools: HashMap<String, String>,
}

impl ProjectToolsConfig {
    /// Load project configuration from vx.toml in current directory or parent directories
    fn load() -> Option<Self> {
        let cwd = std::env::current_dir().ok()?;
        let config_path = find_config_file_upward(&cwd)?;
        let config = parse_config(&config_path).ok()?;
        let tools = config.tools_as_hashmap();

        if tools.is_empty() {
            debug!("No tools defined in vx.toml at {}", config_path.display());
            None
        } else {
            debug!(
                "Loaded {} tool(s) from vx.toml at {}",
                tools.len(),
                config_path.display()
            );
            Some(Self { tools })
        }
    }

    /// Get the version for a specific tool
    fn get_version(&self, tool: &str) -> Option<&str> {
        self.tools.get(tool).map(|s| s.as_str())
    }

    /// Get the version for a tool with ecosystem fallback
    ///
    /// First tries to find the tool directly. If not found, it checks if the tool
    /// belongs to a known ecosystem and tries to use the primary runtime's version.
    ///
    /// **Important**: Only "bundled tools" (tools that are part of the primary runtime)
    /// should fall back to the primary runtime's version. Independent tools like pnpm,
    /// yarn, and bun have their own version schemes and should NOT inherit the Node.js
    /// version.
    ///
    /// Examples of valid fallbacks:
    /// - `cargo` -> checks `cargo` then `rust` (cargo is bundled with Rust)
    /// - `rustc` -> checks `rustc` then `rust` (rustc is bundled with Rust)
    /// - `npm` -> checks `npm` then `node` (npm is bundled with Node.js)
    /// - `pip` -> checks `pip` then `python` (pip is often bundled with Python)
    ///
    /// Examples that should NOT fall back:
    /// - `pnpm` -> only checks `pnpm` (pnpm has its own version scheme: 9.x, 10.x)
    /// - `yarn` -> only checks `yarn` (yarn has its own version scheme: 1.x, 2.x, 3.x, 4.x)
    /// - `bun` -> only checks `bun` (bun has its own version scheme)
    fn get_version_with_fallback(&self, tool: &str) -> Option<&str> {
        // First, try direct lookup
        if let Some(version) = self.get_version(tool) {
            return Some(version);
        }

        // Fallback to primary runtime for the ecosystem (only for bundled tools)
        let primary = self.bundled_tool_runtime(tool)?;
        self.get_version(primary)
    }

    /// Get the primary runtime name for a bundled tool
    ///
    /// Returns Some(runtime) only for tools that are **bundled with** their primary runtime
    /// and share the same version. Independent package managers (pnpm, yarn, bun) are NOT
    /// included because they have their own independent version schemes.
    ///
    /// # Bundled vs Independent Tools
    ///
    /// **Bundled tools** (should fall back):
    /// - `cargo`, `rustc`, `rustup` -> bundled with `rust`
    /// - `npm`, `npx` -> bundled with `node`
    /// - `pip`, `pip3` -> often bundled with `python`
    /// - `gofmt` -> bundled with `go`
    ///
    /// **Independent tools** (should NOT fall back):
    /// - `pnpm` -> has versions like 9.0.1, 10.28.2 (NOT node versions)
    /// - `yarn` -> has versions like 1.22.0, 2.4.3, 4.0.0 (NOT node versions)
    /// - `bun` -> has versions like 1.0.0, 1.1.0 (NOT node versions)
    fn bundled_tool_runtime(&self, tool: &str) -> Option<&'static str> {
        match tool {
            // Rust ecosystem - all are bundled with rustup/rust
            "rustc" | "cargo" | "rustup" => Some("rust"),

            // Node.js ecosystem - ONLY npm/npx are bundled with Node.js
            // pnpm, yarn, bun are INDEPENDENT tools with their own version schemes
            "npm" | "npx" => Some("node"),

            // Python ecosystem - pip is often bundled with Python
            // uv is independent and should not fall back
            "pip" | "pip3" => Some("python"),

            // Go ecosystem - gofmt is bundled with Go
            "gofmt" => Some("go"),

            // Everything else (including pnpm, yarn, bun, uv, etc.) should NOT fall back
            _ => None,
        }
    }
}

impl<'a> Executor<'a> {
    /// Create an executor with a provider registry, runtime context, and runtime map
    ///
    /// The RuntimeMap should be built from provider manifests using
    /// `RuntimeMap::from_manifests()`. See RFC 0017.
    ///
    /// This constructor automatically loads project configuration from `vx.toml`
    /// if present in the current directory or parent directories. When project
    /// configuration is available, the executor will prioritize using the tool
    /// versions specified in `vx.toml` for subprocess PATH construction.
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
        // Create a span for the entire execution
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
        // Check if we should use a bundled version (offline mode)
        // This happens BEFORE any network operations to fail fast when offline
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
            return execute_bundle(&bundle_ctx, args).await;
        }
        let network_offline = !is_online();
        if force_offline || network_offline {
            // Try to find if there's a bundle at all
            let cwd = std::env::current_dir().ok();
            let has_project_bundle = cwd
                .and_then(|cwd| {
                    find_vx_config(&cwd)
                        .ok()
                        .and_then(|p| p.parent().map(has_bundle))
                })
                .unwrap_or(false);

            if has_project_bundle {
                // Bundle exists but tool not found in it
                return Err(anyhow::anyhow!(
                    "Offline mode: tool '{}' not found in bundle. \
                     Run 'vx bundle create' while online to add it.",
                    runtime_name
                ));
            } else if network_offline {
                // No network and no bundle
                return Err(anyhow::anyhow!(
                    "Offline mode: no bundle available and network is offline. \
                     Run 'vx bundle create' while online to create one.",
                ));
            }
            // If force_offline but network is available and no bundle,
            // we can still try the global store, so continue...
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
        debug!("[RESOLVE]");

        // Determine the version to use:
        // 1. Explicit version from command line (e.g., "cargo@1.83")
        // 2. Version from project config (vx.toml)
        // 3. None (use latest installed)
        let resolved_version = if let Some(v) = version {
            Some(v.to_string())
        } else if let Some(ref project_config) = self.project_config {
            // Use project config version with ecosystem fallback
            project_config
                .get_version_with_fallback(runtime_name)
                .map(|s| s.to_string())
        } else {
            None
        };

        debug!(
            "  version: {:?}",
            resolved_version.as_ref().unwrap_or(&"latest".to_string())
        );

        let mut resolution = self
            .resolver
            .resolve_with_version(runtime_name, resolved_version.as_deref())?;
        debug!("  executable: {}", resolution.executable.display(),);
        debug!("  needs_install: {}", resolution.runtime_needs_install);
        debug!("[/RESOLVE]");

        // -------------------------
        // Ensure Installed
        // -------------------------
        debug!("[INSTALL_CHECK]");
        let mut installed_version: Option<String> = None;
        let dependency_env_overrides: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        // Track if we need to re-resolve after installation
        let needs_re_resolve =
            resolution.runtime_needs_install || !resolution.executable.is_absolute();

        // If a specific version is requested (from command line or project config), ensure it's installed first
        if let Some(requested_version) = resolved_version.clone() {
            installed_version = self
                .ensure_version_installed(runtime_name, &requested_version)
                .await?;

            debug!("  installed_version: {:?}", installed_version);
        }

        // Install missing runtimes/dependencies (if any)
        if !resolution.install_order.is_empty() {
            if self.config.auto_install {
                info!("  auto-installing: {:?}", resolution.install_order);

                self.install_runtimes(&resolution.install_order).await?;
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

        // Re-resolve after installation to get the correct executable path
        // This is necessary because the initial resolution may have been done before
        // the runtime was installed, resulting in a relative executable path
        if needs_re_resolve {
            debug!("[RE-RESOLVE] Re-resolving after installation to get correct executable path");
            // Use the installed version if available, otherwise use the original version
            let re_resolve_version = installed_version.as_deref().or(resolved_version.as_deref());
            resolution = self
                .resolver
                .resolve_with_version(runtime_name, re_resolve_version)?;
            debug!(
                "  executable (after re-resolve): {}",
                resolution.executable.display()
            );
        }

        debug!("[/INSTALL_CHECK]");

        // -------------------------
        // Handle incompatible dependencies
        // -------------------------
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
        let mut runtime_env = self
            .prepare_runtime_environment(runtime_name, installed_version.as_deref(), inherit_env)
            .await?;

        // Apply dependency environment overrides
        runtime_env.extend(dependency_env_overrides);
        debug!("  env_vars: {} variables set", runtime_env.len());
        debug!("[/PREPARE_ENV]");

        // -------------------------
        // Execute
        // -------------------------
        debug!("[EXECUTE]");
        // Verify executable exists before attempting to run
        if resolution.executable.is_absolute() && !resolution.executable.exists() {
            return Err(anyhow::anyhow!(
                "Executable not found at '{}'. The runtime may not have been installed correctly. \
                 Try running 'vx install {}' to reinstall.",
                resolution.executable.display(),
                runtime_name
            ));
        }

        // For bundled tools (like npx, npm), ensure the executable's parent directory
        // is in PATH so the runtime (node) can be found
        if resolution.executable.is_absolute() {
            if let Some(exe_dir) = resolution.executable.parent() {
                let exe_dir_str = exe_dir.to_string_lossy().to_string();
                let path_sep = vx_paths::path_separator();

                // Also add the grandparent directory in case the executable is in a subdirectory
                // This handles cases like node-v20.20.0-win-x64/npx.cmd where node.exe is in the same dir
                let grandparent_dir = exe_dir.parent().map(|p| p.to_string_lossy().to_string());

                let current_path = runtime_env
                    .get("PATH")
                    .cloned()
                    .or_else(|| std::env::var("PATH").ok())
                    .unwrap_or_default();

                // Build new PATH: exe_dir + grandparent_dir (if different) + current_path
                let mut new_path = exe_dir_str;
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

        // Build command with environment variables
        let mut cmd = self.build_command(&resolution, args, &runtime_env)?;

        debug!("  cmd: {} {:?}", resolution.executable.display(), args);
        debug!("--- tool output below ---");

        // Execute
        let status = self.run_command(&mut cmd).await?;
        debug!("--- tool output above ---");
        debug!("[/EXECUTE] exit={}", exit_code_from_status(&status));

        Ok(exit_code_from_status(&status))
    }

    /// Expand template variables in environment values
    fn expand_template(
        &self,
        template: &str,
        runtime_name: &str,
        version: Option<&str>,
    ) -> Result<String> {
        let mut result = template.to_string();

        // Get runtime spec for template expansion
        if let Some(_spec) = self.resolver.get_spec(runtime_name) {
            // Replace {install_dir} using PathProvider
            // The install_dir is: ~/.vx/store/<runtime>/<version>/<platform>
            if result.contains("{install_dir}") {
                if let (Some(ctx), Some(ver)) = (self.context, version) {
                    // Get the version store directory and add platform subdirectory
                    let version_dir = ctx.paths.version_store_dir(runtime_name, ver);
                    let platform = vx_paths::manager::CurrentPlatform::current();
                    let install_dir = version_dir.join(platform.as_str());
                    result = result.replace("{install_dir}", &install_dir.to_string_lossy());
                    debug!(
                        "  expand_template: {{install_dir}} -> {}",
                        install_dir.display()
                    );
                }
            }

            // Replace {version}
            if let Some(ver) = version {
                result = result.replace("{version}", ver);
            }

            // Replace {executable} using PathProvider
            if result.contains("{executable}") {
                if let (Some(ctx), Some(ver)) = (self.context, version) {
                    let exe_path = ctx.paths.executable_path(runtime_name, ver);
                    result = result.replace("{executable}", &exe_path.to_string_lossy());
                }
            }

            // Replace {PATH} with current PATH
            if let Ok(path) = std::env::var("PATH") {
                result = result.replace("{PATH}", &path);
            }

            // Replace shell-style variables: $HOME, $USER, etc.
            if result.contains("$HOME") {
                if let Ok(home) = std::env::var("HOME").or_else(|_| {
                    // Fallback to USERPROFILE on Windows
                    std::env::var("USERPROFILE")
                }) {
                    result = result.replace("$HOME", &home);
                }
            }

            if result.contains("$CARGO_HOME") {
                if let Ok(cargo_home) = std::env::var("CARGO_HOME") {
                    result = result.replace("$CARGO_HOME", &cargo_home);
                }
            }

            if result.contains("$RUSTUP_HOME") {
                if let Ok(rustup_home) = std::env::var("RUSTUP_HOME") {
                    result = result.replace("$RUSTUP_HOME", &rustup_home);
                }
            }

            if result.contains("$USER") {
                if let Ok(user) = std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
                    result = result.replace("$USER", &user);
                }
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

    /// Prepare environment variables to use a specific version of a dependency
    #[allow(dead_code)]
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
        // IMPORTANT: Use runtime.store_name() which handles aliases and bundled runtimes
        let store_name = runtime.store_name();
        let store_dir = context.paths.version_store_dir(store_name, version);
        let bin_dir = self.find_bin_dir(&store_dir, store_name);

        if let Some(bin) = bin_dir {
            // Prepend the bin directory to PATH
            let current_path = std::env::var("PATH").unwrap_or_default();
            let new_path = vx_paths::prepend_to_path(&current_path, &[bin.display().to_string()]);
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

        let exe_name = vx_paths::with_executable_extension(tool_name);

        // Common bin directory patterns
        let patterns = [
            store_dir.join("bin"),
            store_dir.to_path_buf(),
            store_dir.join(tool_name).join("bin"),
            store_dir.join("Scripts"), // Windows Python pattern
            store_dir.join(tool_name), // Check tool-name subdirectory (e.g., python/)
        ];

        for pattern in &patterns {
            if pattern.exists() && pattern.is_dir() {
                // Check if this directory contains executables
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

        // Check if this runtime is provided by another runtime
        // For example, cargo and rustc are provided by rustup
        if let Some(spec) = self.resolver.get_spec(runtime_name) {
            for dep in &spec.dependencies {
                if dep.required {
                    if let Some(ref provider) = dep.provided_by {
                        // This runtime is provided by another runtime
                        info!(
                            "{} is provided by {}, installing {}@{} instead",
                            runtime_name, provider, provider, requested_version
                        );

                        // Install the provider with the requested version
                        return Box::pin(
                            self.ensure_version_installed(provider, requested_version),
                        )
                        .await;
                    }
                }
            }
        }

        let runtime = registry
            .get_runtime(runtime_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown runtime: {}", runtime_name))?;

        // Resolve the version constraint to an actual version - show progress spinner
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
            "Resolved {}@{} 鈫?{}",
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
        // Note: We don't show a spinner here because runtime.install() will show
        // its own download progress, and having two progress indicators causes flickering
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

                // For Rust ecosystem (cargo, rustc depend on rustup), the dependency
                // should use the same version as the parent runtime since rustup manages
                // Rust toolchain versions (e.g., 1.90.0, stable, nightly).
                // This ensures that when installing cargo@1.90.0, rustup also installs
                // toolchain 1.90.0 instead of defaulting to "stable".
                let dep_version = if self.is_rust_ecosystem(runtime_name, dep_runtime) {
                    Some(version)
                } else {
                    // For other ecosystems, use the version from RuntimeDependency if specified
                    dep.recommended_version.as_deref()
                };

                // Install the dependency with version if available
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
    /// where version consistency is important (rustup manages toolchain versions)
    fn is_rust_ecosystem(&self, runtime_name: &str, dep_runtime: &str) -> bool {
        let rust_runtimes = ["cargo", "rustc", "rust", "rustup"];
        rust_runtimes.contains(&runtime_name) && rust_runtimes.contains(&dep_runtime)
    }

    /// Get the provider runtime for a bundled tool
    ///
    /// For bundled tools (e.g., npx bundled with node), returns the provider runtime
    /// name and version. This is used to set up the correct environment.
    ///
    /// Returns (Some(provider_name), Some(version)) if this is a bundled tool,
    /// or (None, None) if it's a standalone runtime.
    fn get_provider_runtime(
        &self,
        runtime_name: &str,
        version: Option<&str>,
    ) -> (Option<String>, Option<String>) {
        if let Some(spec) = self.resolver.get_spec(runtime_name) {
            for dep in &spec.dependencies {
                if dep.required {
                    if let Some(ref provider) = dep.provided_by {
                        // This is a bundled tool, return the provider
                        return (Some(provider.clone()), version.map(|v| v.to_string()));
                    }
                }
            }
        }
        (None, None)
    }

    /// Prepare environment variables for a runtime
    ///
    /// This combines environment variables from:
    /// 1. Runtime's prepare_environment method
    /// 2. Manifest's env_config (including advanced configuration)
    ///
    /// For bundled tools (e.g., npx bundled with node), this uses the provider
    /// runtime's environment configuration to ensure correct PATH setup.
    async fn prepare_runtime_environment(
        &self,
        runtime_name: &str,
        version: Option<&str>,
        inherit_env: bool,
    ) -> Result<std::collections::HashMap<String, String>> {
        use std::collections::HashMap;

        let mut env = HashMap::new();

        // For bundled tools, use the provider runtime's environment configuration
        // e.g., for npx (bundled with node), use node's env_config
        let (effective_runtime, effective_version) =
            self.get_provider_runtime(runtime_name, version);
        let effective_runtime_name = effective_runtime.as_deref().unwrap_or(runtime_name);
        let effective_version_ref = effective_version.as_deref().or(version);

        debug!(
            "  prepare_env for {} (effective: {}@{:?})",
            runtime_name, effective_runtime_name, effective_version_ref
        );

        // Get environment from manifest's env_config
        if let Some(spec) = self.resolver.get_spec(effective_runtime_name) {
            if let Some(env_config) = &spec.env_config {
                // Handle advanced environment configuration
                if let Some(advanced) = &env_config.advanced {
                    // Handle PATH manipulation
                    let mut path_parts = Vec::new();

                    // Prepend entries - use effective runtime for template expansion
                    for entry in &advanced.path_prepend {
                        let expanded = self.expand_template(
                            entry,
                            effective_runtime_name,
                            effective_version_ref,
                        )?;
                        path_parts.push(expanded);
                    }

                    // Get current PATH if not isolated or if inheriting
                    let isolate_env = if inherit_env { false } else { advanced.isolate };

                    // Get effective inherit_system_vars (defaults + provider-specific)
                    let inherit_vars = env_config.effective_inherit_system_vars();

                    let current_path = if !isolate_env {
                        std::env::var("PATH").unwrap_or_default()
                    } else {
                        // When isolated, filter PATH to only include system directories
                        // This ensures child processes can find essential tools (sh, bash, etc.)
                        // while excluding user-specific paths for isolation
                        if let Ok(full_path) = std::env::var("PATH") {
                            vx_manifest::filter_system_path(&full_path)
                        } else {
                            String::new()
                        }
                    };

                    // Split current_path and add each directory separately
                    // This is necessary because std::env::join_paths expects individual paths,
                    // not a single string containing the path separator
                    if !current_path.is_empty() {
                        for part in vx_paths::split_path(&current_path) {
                            path_parts.push(part.to_string());
                        }
                    }

                    // Append entries - use effective runtime for template expansion
                    for entry in &advanced.path_append {
                        let expanded = self.expand_template(
                            entry,
                            effective_runtime_name,
                            effective_version_ref,
                        )?;
                        path_parts.push(expanded);
                    }

                    // Set PATH
                    if !path_parts.is_empty() {
                        env.insert(
                            "PATH".to_string(),
                            std::env::join_paths(path_parts)?
                                .to_string_lossy()
                                .to_string(),
                        );
                    }

                    // Handle advanced env vars
                    // Use effective runtime for all template expansions
                    for (var_name, var_config) in &advanced.env_vars {
                        match var_config {
                            vx_manifest::EnvVarConfig::Simple(value) => {
                                let expanded = self.expand_template(
                                    value,
                                    effective_runtime_name,
                                    effective_version_ref,
                                )?;
                                env.insert(var_name.clone(), expanded);
                            }
                            vx_manifest::EnvVarConfig::Advanced {
                                value,
                                prepend,
                                append,
                                replace,
                            } => {
                                let mut final_value = String::new();

                                if *replace {
                                    if let Some(v) = value {
                                        final_value = self.expand_template(
                                            v,
                                            effective_runtime_name,
                                            effective_version_ref,
                                        )?;
                                    }
                                } else {
                                    // Prepend
                                    if let Some(pre) = prepend {
                                        for item in pre {
                                            let expanded = self.expand_template(
                                                item,
                                                effective_runtime_name,
                                                effective_version_ref,
                                            )?;
                                            final_value.push_str(&expanded);
                                            final_value.push(vx_paths::path_separator());
                                        }
                                    }

                                    // Current value
                                    if let Ok(current) = std::env::var(var_name) {
                                        final_value.push_str(&current);
                                        if !final_value.ends_with(vx_paths::path_separator()) {
                                            final_value.push(vx_paths::path_separator());
                                        }
                                    }

                                    // Append
                                    if let Some(app) = append {
                                        for item in app {
                                            let expanded = self.expand_template(
                                                item,
                                                effective_runtime_name,
                                                effective_version_ref,
                                            )?;
                                            final_value.push_str(&expanded);
                                            final_value.push(vx_paths::path_separator());
                                        }
                                    }

                                    // Remove trailing separator
                                    final_value = final_value
                                        .trim_end_matches(vx_paths::path_separator())
                                        .to_string();
                                }

                                if !final_value.is_empty() {
                                    env.insert(var_name.clone(), final_value);
                                }
                            }
                        }
                    }

                    // Inherit system vars (excluding PATH which is handled above)
                    // Uses effective_inherit_system_vars which combines defaults + provider-specific
                    // This ensures variables like SHELL, TERM, HOME, etc. are available to child processes
                    // which may spawn shell scripts (e.g., npm postinstall scripts)
                    for var_pattern in &inherit_vars {
                        if var_pattern == "PATH" {
                            continue; // PATH is handled separately above
                        }

                        // Handle glob patterns like "LC_*"
                        if var_pattern.contains('*') {
                            let prefix = var_pattern.trim_end_matches('*');
                            for (key, value) in std::env::vars() {
                                if key.starts_with(prefix) && !env.contains_key(&key) {
                                    env.insert(key, value);
                                }
                            }
                        } else if let Ok(value) = std::env::var(var_pattern) {
                            // Only insert if not already set
                            if !env.contains_key(var_pattern) {
                                env.insert(var_pattern.clone(), value);
                            }
                        }
                    }
                }

                // Add basic vars - use effective runtime for template expansion
                for (key, value) in &spec.env_vars {
                    let expanded =
                        self.expand_template(value, effective_runtime_name, effective_version_ref)?;
                    env.insert(key.clone(), expanded);
                }
            }
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

        // Call prepare_environment and merge
        match runtime.prepare_environment(&version, context).await {
            Ok(runtime_env) => {
                env.extend(runtime_env);
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
                Ok(env)
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
                let version = versions
                    .iter()
                    .find(|v| !v.prerelease)
                    .map(|v| v.version.clone())
                    .or_else(|| versions.first().map(|v| v.version.clone()))
                    .ok_or_else(|| anyhow::anyhow!("No versions found for {}", runtime_name))?;

                info!("Installing {} {} via provider", runtime_name, version);

                // Run pre-install hook
                runtime.pre_install(&version, context).await?;

                // Install the runtime
                // Note: We don't show a spinner here because runtime.install() will show
                // its own download progress, and having two progress indicators causes flickering
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

    /// Install a single runtime with a specific version
    ///
    /// This is used when installing dependencies that need to match the parent runtime's version,
    /// such as rustup needing to install the same toolchain version as cargo.
    ///
    /// Returns the installed version if successful
    async fn install_runtime_with_version(
        &self,
        runtime_name: &str,
        version: &str,
    ) -> Result<Option<String>> {
        info!("Installing: {}@{}", runtime_name, version);

        // Try using the provider registry first
        if let (Some(registry), Some(context)) = (self.registry, self.context) {
            if let Some(runtime) = registry.get_runtime(runtime_name) {
                // Check platform support before attempting installation
                if let Err(e) = runtime.check_platform_support() {
                    return Err(anyhow::anyhow!("{}", e));
                }

                info!(
                    "Installing {} {} via provider (explicit version)",
                    runtime_name, version
                );

                // Run pre-install hook
                runtime.pre_install(version, context).await?;

                // Install the runtime with the specified version
                debug!(
                    "Calling runtime.install() for {} {} (explicit version)",
                    runtime_name, version
                );
                let result = runtime.install(version, context).await?;
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
                runtime.post_install(version, context).await?;

                info!("Successfully installed {} {}", runtime_name, version);
                return Ok(Some(version.to_string()));
            }
        }

        // Fallback: try to install using known methods
        // Note: Fallback doesn't support version specification
        warn!(
            "No provider found for {}, falling back to system installation (version {} will be ignored)",
            runtime_name, version
        );
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

        // Build the final environment with optional vx PATH inheritance
        let mut final_env = runtime_env.clone();

        // If inherit_vx_path is enabled, prepend all vx-managed tool bin directories to PATH
        if self.config.inherit_vx_path {
            if let Some(vx_path) = self.build_vx_tools_path() {
                let current_path = final_env
                    .get("PATH")
                    .cloned()
                    .or_else(|| std::env::var("PATH").ok())
                    .unwrap_or_default();

                let new_path = if current_path.is_empty() {
                    vx_path
                } else {
                    vx_paths::prepend_to_path(&current_path, &[vx_path])
                };

                final_env.insert("PATH".to_string(), new_path);
                trace!("PATH includes vx-managed tools for {}", resolution.runtime);
            }
        }

        // Inject environment variables
        if !final_env.is_empty() {
            trace!(
                "injecting {} env vars for {}",
                final_env.len(),
                resolution.runtime
            );
            for (key, value) in &final_env {
                cmd.env(key, value);
            }
        }

        // Inherit stdio
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        Ok(cmd)
    }

    /// Build PATH string containing all vx-managed tool bin directories
    ///
    /// This allows subprocesses to access vx-managed tools without the `vx` prefix.
    /// For example, when running `vx just lint`, the justfile can use `uvx nox -s lint`
    /// directly instead of `vx uvx nox -s lint`.
    ///
    /// ## Project Configuration Priority
    ///
    /// When a `vx.toml` file is present, the executor prioritizes using the tool
    /// versions specified in the project configuration. This ensures:
    ///
    /// - **Environment Isolation**: Each project uses its own tool versions
    /// - **No Global Pollution**: Globally installed newer versions don't affect projects
    /// - **Reproducibility**: Same tool versions across all team members
    ///
    /// ### Version Selection Order
    ///
    /// 1. If `vx.toml` specifies a version for the tool → use that version
    /// 2. If the specified version is not installed → fall back to latest installed
    /// 3. If no `vx.toml` exists → use latest installed version (existing behavior)
    fn build_vx_tools_path(&self) -> Option<String> {
        let context = self.context?;
        let registry = self.registry?;

        let mut paths: Vec<String> = Vec::new();

        // Add vx bin directory first (for shims)
        let vx_bin = context.paths.bin_dir();
        if vx_bin.exists() {
            paths.push(vx_bin.to_string_lossy().to_string());
        }

        // Collect all installed runtime bin directories by scanning the store directory
        // We use synchronous filesystem operations to avoid runtime issues with block_in_place
        for runtime in registry.supported_runtimes() {
            let runtime_name = runtime.store_name();
            let runtime_store_dir = context.paths.runtime_store_dir(runtime_name);

            // Skip if the runtime store directory doesn't exist
            if !runtime_store_dir.exists() {
                continue;
            }

            // Get installed versions by reading directory entries
            if let Ok(entries) = std::fs::read_dir(&runtime_store_dir) {
                let installed_versions: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect();

                // Determine which version to use:
                // 1. Priority: version from vx.toml (project configuration)
                // 2. Fallback: latest installed version
                let version_to_use =
                    self.select_version_for_runtime(runtime_name, &installed_versions);

                if let Some(version) = version_to_use {
                    let store_dir = context.paths.version_store_dir(runtime_name, &version);

                    if let Some(bin_dir) = self.find_bin_dir(&store_dir, runtime_name) {
                        if bin_dir.exists() {
                            let bin_path = bin_dir.to_string_lossy().to_string();
                            // Avoid duplicates
                            if !paths.contains(&bin_path) {
                                paths.push(bin_path);
                            }
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
    ///
    /// This method implements the version selection logic:
    /// 1. If `vx.toml` specifies a version and it's installed 鈫?use it
    /// 2. If `vx.toml` specifies a version but it's not installed 鈫?log warning, use latest
    /// 3. If no `vx.toml` or no version specified 鈫?use latest installed
    fn select_version_for_runtime(
        &self,
        runtime_name: &str,
        installed_versions: &[String],
    ) -> Option<String> {
        if installed_versions.is_empty() {
            return None;
        }

        // Check if project configuration specifies a version for this runtime
        // Uses ecosystem fallback: e.g., "cargo" falls back to "rust" configuration
        if let Some(ref project_config) = self.project_config {
            if let Some(requested_version) = project_config.get_version_with_fallback(runtime_name)
            {
                // Try to find exact match or compatible version
                let matching_version =
                    self.find_matching_version(runtime_name, requested_version, installed_versions);

                if let Some(version) = matching_version {
                    trace!("Using {} version {} from vx.toml", runtime_name, version);
                    return Some(version);
                } else {
                    // Requested version not installed, warn and fall back to latest
                    // Only warn once per tool to avoid flooding the output
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
        }

        // Fall back to latest installed version
        let mut versions = installed_versions.to_vec();
        versions.sort_by(|a, b| {
            // Try semver comparison, fall back to string comparison
            self.compare_versions(a, b)
        });

        versions.last().cloned()
    }

    /// Find a matching version from installed versions
    ///
    /// Supports:
    /// - Exact version match (e.g., "20.0.0" matches "20.0.0")
    /// - Major version prefix (e.g., "20" matches "20.0.0", "20.1.0")
    /// - Major.minor prefix (e.g., "20.0" matches "20.0.0", "20.0.1")
    fn find_matching_version(
        &self,
        _runtime_name: &str,
        requested: &str,
        installed: &[String],
    ) -> Option<String> {
        // First try exact match
        if installed.contains(&requested.to_string()) {
            return Some(requested.to_string());
        }

        // Try prefix match for partial versions (e.g., "20" matches "20.0.0")
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
    ///
    /// Attempts semver comparison, falls back to string comparison
    fn compare_versions(&self, a: &str, b: &str) -> std::cmp::Ordering {
        // Try to parse as semver (handling potential 'v' prefix)
        let a_clean = a.trim_start_matches('v');
        let b_clean = b.trim_start_matches('v');

        // Split into parts and compare numerically
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

        // Compare each part
        for (ap, bp) in a_parts.iter().zip(b_parts.iter()) {
            match ap.cmp(bp) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            }
        }

        // If all compared parts are equal, longer version is "greater"
        a_parts.len().cmp(&b_parts.len())
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

    Ok(exit_code_from_status(&status))
}

// ============================================================================
// Offline Bundle Support
// ============================================================================

/// Bundle directory name within .vx
pub const BUNDLE_DIR: &str = "bundle";

/// Bundle manifest file name
pub const BUNDLE_MANIFEST: &str = "manifest.json";

/// Context for using a bundled tool
#[derive(Debug, Clone)]
pub struct BundleContext {
    /// Project root directory
    pub project_root: PathBuf,
    /// Tool name
    pub tool_name: String,
    /// Tool version
    pub version: String,
    /// Full path to executable
    pub executable: PathBuf,
}

/// Bundle manifest containing metadata about the bundled environment
#[derive(Debug, Clone, serde::Deserialize)]
pub struct BundleManifest {
    /// Manifest version (1 = legacy single-platform, 2 = multi-platform)
    #[serde(default = "default_manifest_version")]
    pub version: u32,
    /// All platforms included in this bundle
    #[serde(default)]
    pub platforms: Vec<String>,
    /// Bundled tools with their versions
    pub tools: HashMap<String, BundledToolInfo>,
}

fn default_manifest_version() -> u32 {
    1
}

/// Information about a bundled tool
#[derive(Debug, Clone, serde::Deserialize)]
pub struct BundledToolInfo {
    /// Resolved version
    pub version: String,
    /// Legacy path (for v1 manifests)
    #[serde(default)]
    pub path: String,
    /// Platform-specific paths (for v2 manifests)
    #[serde(default)]
    pub platform_paths: HashMap<String, String>,
}

impl BundledToolInfo {
    /// Get the path for a specific platform
    pub fn path_for_platform(&self, platform: &str) -> Option<&str> {
        // First try platform-specific path
        if let Some(path) = self.platform_paths.get(platform) {
            return Some(path.as_str());
        }
        // Fall back to legacy single path (for v1 manifests)
        if !self.path.is_empty() {
            return Some(&self.path);
        }
        None
    }
}

impl BundleManifest {
    /// Check if this bundle supports a specific platform
    pub fn supports_platform(&self, platform: &str) -> bool {
        if self.platforms.is_empty() {
            // v1 manifest - assume it supports current platform
            true
        } else {
            self.platforms.contains(&platform.to_string())
        }
    }
}

/// Quick network connectivity check
///
/// Uses a fast DNS lookup to determine if the system has internet access.
/// Returns true if online, false if offline.
pub fn is_online() -> bool {
    use std::net::ToSocketAddrs;

    // Try multiple targets for reliability
    let targets = ["github.com:443", "nodejs.org:443", "pypi.org:443"];

    for target in targets {
        if let Ok(mut addrs) = target.to_socket_addrs() {
            if addrs.next().is_some() {
                return true;
            }
        }
    }

    false
}

/// Check if a bundle exists for the given project root
pub fn has_bundle(project_root: &std::path::Path) -> bool {
    let manifest_path = project_root
        .join(PROJECT_VX_DIR)
        .join(BUNDLE_DIR)
        .join(BUNDLE_MANIFEST);
    manifest_path.exists()
}

/// Load bundle manifest from the project
fn load_bundle_manifest(project_root: &std::path::Path) -> Option<BundleManifest> {
    let manifest_path = project_root
        .join(PROJECT_VX_DIR)
        .join(BUNDLE_DIR)
        .join(BUNDLE_MANIFEST);

    let content = std::fs::read_to_string(&manifest_path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Get current platform string (same format as bundle.rs)
fn current_platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

/// Get bundle store path for a tool
/// Supports both v1 (single platform) and v2 (multi-platform) bundle structures
fn get_bundle_tool_path(
    project_root: &std::path::Path,
    tool_name: &str,
    version: &str,
) -> Option<PathBuf> {
    let base_path = project_root
        .join(PROJECT_VX_DIR)
        .join(BUNDLE_DIR)
        .join("store")
        .join(tool_name)
        .join(version);

    // v2 structure: store/{tool}/{version}/{platform}/
    let platform = current_platform();
    let platform_path = base_path.join(&platform);
    if platform_path.exists() {
        return Some(platform_path);
    }

    // v1 structure (legacy): store/{tool}/{version}/
    if base_path.exists() {
        // Check if this is actually a v1 layout (no platform subdirectories)
        // by verifying it's not just an empty directory or a different platform
        if let Ok(entries) = std::fs::read_dir(&base_path) {
            let subdirs: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .collect();

            // If subdirs look like platform names (e.g., "windows-x86_64"),
            // this is v2 format and current platform is not available
            let has_platform_subdirs = subdirs.iter().any(|d| {
                let name = d.file_name().to_string_lossy().to_string();
                name.contains('-')
                    && (name.contains("windows")
                        || name.contains("linux")
                        || name.contains("macos"))
            });

            if has_platform_subdirs {
                // v2 structure, but current platform not available
                debug!(
                    "Bundle has platform-specific subdirectories but not for current platform: {}",
                    platform
                );
                return None;
            }

            // v1 structure
            return Some(base_path);
        }
    }

    None
}

/// Find executable in bundle for a tool
///
/// Returns the full path to the executable if found in the bundle.
fn find_bundle_executable(
    project_root: &std::path::Path,
    tool_name: &str,
    version: &str,
) -> Option<PathBuf> {
    let bundle_tool_path = get_bundle_tool_path(project_root, tool_name, version)?;

    // Common executable search paths within a tool directory
    let search_paths = [
        "bin",     // Most tools
        "Scripts", // Windows Python
        "",        // Root directory
    ];

    #[cfg(windows)]
    let exe_names = [
        format!("{}.exe", tool_name),
        format!("{}.cmd", tool_name),
        format!("{}.bat", tool_name),
        tool_name.to_string(),
    ];

    #[cfg(not(windows))]
    let exe_names = [tool_name.to_string()];

    for search_path in &search_paths {
        let base = if search_path.is_empty() {
            bundle_tool_path.clone()
        } else {
            bundle_tool_path.join(search_path)
        };

        for exe_name in &exe_names {
            let exe_path = base.join(exe_name);
            if exe_path.exists() && exe_path.is_file() {
                return Some(exe_path);
            }
        }
    }

    // Also search in subdirectories (e.g., node-v20.0.0-win-x64/)
    if let Ok(entries) = std::fs::read_dir(&bundle_tool_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Check bin subdirectory
                let bin_path = path.join("bin");
                for exe_name in &exe_names {
                    let exe_path = bin_path.join(exe_name);
                    if exe_path.exists() && exe_path.is_file() {
                        return Some(exe_path);
                    }
                }
                // Check root of subdirectory
                for exe_name in &exe_names {
                    let exe_path = path.join(exe_name);
                    if exe_path.exists() && exe_path.is_file() {
                        return Some(exe_path);
                    }
                }
            }
        }
    }

    None
}

/// Try to get bundle context for offline execution
///
/// Returns bundle information if:
/// 1. Network is offline (or force_offline is true)
/// 2. Bundle exists in the project
/// 3. The requested tool is available in the bundle
pub fn try_get_bundle_context(tool_name: &str, force_offline: bool) -> Option<BundleContext> {
    // Check if we should use bundle
    if !force_offline && is_online() {
        return None;
    }

    // Find project root
    let cwd = std::env::current_dir().ok()?;
    let config_path = find_vx_config(&cwd).ok()?;
    let project_root = config_path.parent()?;

    // Load bundle manifest
    let manifest = load_bundle_manifest(project_root)?;

    // Check if tool is in bundle
    let bundled_tool = manifest.tools.get(tool_name)?;

    // Find executable
    let executable = find_bundle_executable(project_root, tool_name, &bundled_tool.version)?;

    info!(
        "Using bundled {} {} (offline mode)",
        tool_name, bundled_tool.version
    );

    Some(BundleContext {
        project_root: project_root.to_path_buf(),
        tool_name: tool_name.to_string(),
        version: bundled_tool.version.clone(),
        executable,
    })
}

/// Execute a bundled tool directly
///
/// This bypasses the normal resolution/installation flow and runs
/// the executable directly from the bundle.
pub async fn execute_bundle(bundle: &BundleContext, args: &[String]) -> Result<i32> {
    debug!(
        "Executing bundled tool: {} {} ({})",
        bundle.tool_name,
        bundle.version,
        bundle.executable.display()
    );

    // On Windows, .cmd and .bat files need to be executed via cmd.exe
    #[cfg(windows)]
    let mut cmd = {
        let ext = bundle
            .executable
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if ext == "cmd" || ext == "bat" {
            let mut c = Command::new("cmd.exe");
            c.arg("/c").arg(&bundle.executable);
            c
        } else {
            Command::new(&bundle.executable)
        }
    };

    #[cfg(not(windows))]
    let mut cmd = Command::new(&bundle.executable);

    cmd.args(args);

    // Set up environment with bundle bin directories in PATH
    let bundle_bin = bundle.executable.parent().map(|p| p.to_path_buf());
    if let Some(bin_dir) = bundle_bin {
        let current_path = std::env::var("PATH").unwrap_or_default();
        let new_path = vx_paths::prepend_to_path(&current_path, &[bin_dir.display().to_string()]);
        cmd.env("PATH", new_path);
    }

    // Inherit stdio
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    let status = cmd
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to execute bundled '{}': {}", bundle.tool_name, e))?;

    Ok(exit_code_from_status(&status))
}
