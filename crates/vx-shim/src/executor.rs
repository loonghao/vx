//! Shim executor for running globally installed package executables
//!
//! This module handles execution of globally installed packages with proper
//! runtime environment setup, similar to REZ's dynamic environment management.
//!
//! ## Environment Setup
//!
//! When executing a package that has runtime dependencies (e.g., npm packages
//! depending on Node.js), the executor automatically sets up the runtime's
//! bin directory in PATH before execution.
//!
//! ## Example
//!
//! When running `opencode` (an npm package):
//! 1. Load package registry to find runtime dependency (node@20)
//! 2. Build environment with node's bin directory in PATH
//! 3. Execute the shim with the prepared environment

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info, warn};

use vx_env::ToolEnvironment;
use vx_paths::global_packages::{GlobalPackage, PackageRegistry};
use vx_paths::shims;

use crate::error::{ShimError, ShimResult};
use crate::request::PackageRequest;

/// Shim executor that handles execution of globally installed packages
pub struct ShimExecutor {
    /// Path to the package registry file
    registry_path: PathBuf,
    /// Path to the shims directory
    shims_dir: PathBuf,
}

impl ShimExecutor {
    /// Create a new shim executor
    pub fn new(registry_path: PathBuf, shims_dir: PathBuf) -> Self {
        Self {
            registry_path,
            shims_dir,
        }
    }

    /// Try to execute an executable by name
    ///
    /// This looks up the executable in the package registry and runs the shim.
    /// Returns `Ok(Some(exit_code))` if the shim was found and executed,
    /// `Ok(None)` if no matching shim exists.
    pub async fn try_execute(&self, exe_name: &str, args: &[String]) -> ShimResult<Option<i32>> {
        self.try_execute_with_deps(exe_name, args, &[]).await
    }

    /// Try to execute an executable with additional --with dependencies
    ///
    /// This is the same as `try_execute` but allows injecting additional
    /// runtime dependencies via the --with flag.
    pub async fn try_execute_with_deps(
        &self,
        exe_name: &str,
        args: &[String],
        with_deps: &[vx_core::WithDependency],
    ) -> ShimResult<Option<i32>> {
        // Load the package registry
        let registry = match PackageRegistry::load(&self.registry_path) {
            Ok(r) => r,
            Err(_) => {
                debug!("No package registry found, skipping shim execution");
                return Ok(None);
            }
        };

        // Check if this executable is from a globally installed package
        let package = match registry.find_by_executable(exe_name) {
            Some(p) => p,
            None => {
                debug!("Executable '{}' not found in package registry", exe_name);
                return Ok(None);
            }
        };

        self.execute_package_shim_with_deps(package, exe_name, args, with_deps)
            .await
    }

    /// Execute a package request (RFC 0027 syntax)
    ///
    /// This handles the full `ecosystem:package@version::executable` syntax.
    pub async fn execute_request(
        &self,
        request: &PackageRequest,
        args: &[String],
    ) -> ShimResult<i32> {
        self.execute_request_with_deps(request, args, &[]).await
    }

    /// Execute a package request with additional --with dependencies
    ///
    /// This is the same as `execute_request` but allows injecting additional
    /// runtime dependencies via the --with flag.
    pub async fn execute_request_with_deps(
        &self,
        request: &PackageRequest,
        args: &[String],
        with_deps: &[vx_core::WithDependency],
    ) -> ShimResult<i32> {
        debug!("Execute package request: {:?}", request);

        let exe_name = request.executable_name();
        debug!("Executable name: {}", exe_name);

        // Load the package registry
        let registry = PackageRegistry::load(&self.registry_path).map_err(|e| {
            ShimError::Other(anyhow::anyhow!("Failed to load package registry: {}", e))
        })?;

        debug!(
            "Registry loaded, looking for package: {}:{}",
            request.ecosystem, request.package
        );

        // Try to find the package
        if let Some(package) = registry.get(&request.ecosystem, &request.package) {
            debug!("Package found: {:?}", package.name);
            // Package is installed, execute it
            if let Some(exit_code) = self
                .execute_package_shim_with_deps(package, exe_name, args, with_deps)
                .await?
            {
                return Ok(exit_code);
            }
            // Shim not found for this executable
            return Err(ShimError::ExecutableNotFound {
                package: request.package.clone(),
                executable: exe_name.to_string(),
            });
        }

        debug!("Package not found in registry");
        // Package not installed - caller should handle auto-installation
        Err(ShimError::PackageNotInstalled {
            ecosystem: request.ecosystem.clone(),
            package: request.package.clone(),
        })
    }

    /// Check if an executable exists as a shim
    pub fn has_shim(&self, exe_name: &str) -> bool {
        shims::shim_exists(&self.shims_dir, exe_name)
    }

    /// Execute a shim for a package
    ///
    /// This method handles runtime dependency injection similar to REZ's
    /// dynamic environment management. If the package has a runtime dependency
    /// (e.g., npm packages depend on node), the runtime's bin directory is
    /// automatically added to PATH before execution.
    ///
    /// Note: We execute the target executable directly (not through the shim script)
    /// to ensure our environment variables are properly inherited.
    #[allow(dead_code)]
    async fn execute_package_shim(
        &self,
        package: &GlobalPackage,
        exe_name: &str,
        args: &[String],
    ) -> ShimResult<Option<i32>> {
        self.execute_package_shim_with_deps(package, exe_name, args, &[])
            .await
    }

    /// Execute a shim for a package with additional --with dependencies
    ///
    /// This is the same as `execute_package_shim` but allows injecting additional
    /// runtime dependencies via the --with flag, similar to uvx --with or rez-env.
    ///
    /// The --with dependencies are injected BEFORE the package's own runtime
    /// dependencies, giving them higher priority in PATH resolution.
    async fn execute_package_shim_with_deps(
        &self,
        package: &GlobalPackage,
        exe_name: &str,
        args: &[String],
        with_deps: &[vx_core::WithDependency],
    ) -> ShimResult<Option<i32>> {
        debug!(
            "Execute package shim for: {} (package: {}, with_deps: {:?})",
            exe_name, package.name, with_deps
        );

        // Find the actual executable in the package's install directory
        let target_path = self.find_executable_in_package(package, exe_name);

        debug!(
            "Target path from find_executable_in_package: {:?}",
            target_path
        );

        let target_path = match target_path {
            Some(p) => p,
            None => {
                // Fall back to shim if direct executable not found
                if !shims::shim_exists(&self.shims_dir, exe_name) {
                    warn!(
                        "Package '{}' provides '{}' but neither direct executable nor shim found",
                        package.name, exe_name
                    );
                    return Ok(None);
                }
                shims::get_shim_path(&self.shims_dir, exe_name)
            }
        };

        debug!(
            "Executing: {} (from {}:{}) at {:?}",
            exe_name, package.ecosystem, package.name, target_path
        );

        // Build environment with runtime dependencies (REZ-like dynamic environment)
        // Include --with dependencies
        let env = self.build_runtime_environment_with_deps(package, with_deps)?;

        // Execute the target directly with the prepared environment
        let status = Command::new(&target_path)
            .args(args)
            .envs(&env)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await?;

        Ok(Some(status.code().unwrap_or(1)))
    }

    /// Find the executable in the package's install directory
    fn find_executable_in_package(
        &self,
        package: &GlobalPackage,
        exe_name: &str,
    ) -> Option<PathBuf> {
        let install_dir = &package.install_dir;

        // Try various executable locations and extensions
        let candidates = if cfg!(windows) {
            vec![
                install_dir.join(format!("{}.cmd", exe_name)),
                install_dir.join(format!("{}.exe", exe_name)),
                install_dir.join(format!("{}.bat", exe_name)),
                install_dir.join(exe_name),
                // Also check bin subdirectory
                install_dir.join("bin").join(format!("{}.cmd", exe_name)),
                install_dir.join("bin").join(format!("{}.exe", exe_name)),
                install_dir.join("bin").join(exe_name),
            ]
        } else {
            vec![
                install_dir.join(exe_name),
                install_dir.join("bin").join(exe_name),
            ]
        };

        for candidate in candidates {
            if candidate.exists() {
                debug!("Found executable at: {:?}", candidate);
                return Some(candidate);
            }
        }

        debug!(
            "Executable '{}' not found in install_dir: {:?}",
            exe_name, install_dir
        );
        None
    }

    /// Build runtime environment for package execution
    ///
    /// If the package has runtime dependencies (e.g., node + bun for some npm packages),
    /// this method builds an environment with all runtime bin directories
    /// prepended to PATH.
    ///
    /// For backward compatibility, if `runtime_dependencies` is empty but
    /// `runtime_dependency` is set, we use that. If neither is set but the
    /// ecosystem is known (e.g., "npm"), we infer the runtime automatically.
    #[allow(dead_code)]
    fn build_runtime_environment(
        &self,
        package: &GlobalPackage,
    ) -> ShimResult<HashMap<String, String>> {
        self.build_runtime_environment_with_deps(package, &[])
    }

    /// Build runtime environment for package execution with additional --with dependencies
    ///
    /// This is similar to `build_runtime_environment` but also includes additional
    /// runtime dependencies specified via the --with flag.
    ///
    /// The --with dependencies are added FIRST in PATH order, giving them higher priority.
    /// This allows users to override or supplement the package's default runtime dependencies.
    ///
    /// # Example
    ///
    /// ```text
    /// vx --with bun npm:opencode-ai@latest::opencode
    /// ```
    ///
    /// This will inject bun's bin directory into PATH before node's bin directory,
    /// allowing opencode to use bun as a runtime.
    fn build_runtime_environment_with_deps(
        &self,
        package: &GlobalPackage,
        with_deps: &[vx_core::WithDependency],
    ) -> ShimResult<HashMap<String, String>> {
        debug!(
            "Build runtime environment for package: {} (ecosystem: {}, with_deps: {:?})",
            package.name, package.ecosystem, with_deps
        );

        // Get runtime dependencies (explicit or inferred from ecosystem)
        let mut runtime_deps = package.get_runtime_dependencies();

        // If no explicit dependencies, infer all applicable runtimes from ecosystem
        if runtime_deps.is_empty() {
            runtime_deps = self.infer_all_runtimes_from_ecosystem(&package.ecosystem);
        }

        debug!("Package runtime dependencies: {:?}", runtime_deps);

        // Build environment with all dependencies
        // --with dependencies are added first (higher priority in PATH)
        let mut tool_env = ToolEnvironment::new();

        // Add --with dependencies first (they take priority in PATH)
        for dep in with_deps {
            let version = dep.version.as_deref().unwrap_or("latest");
            info!("Injecting --with runtime: {}@{}", dep.runtime, version);
            tool_env = tool_env.tool(&dep.runtime, version);
        }

        // Add package's own runtime dependencies
        for dep in &runtime_deps {
            // Skip if already added via --with (avoid duplicates)
            if with_deps.iter().any(|w| w.runtime == dep.runtime) {
                debug!(
                    "Skipping package runtime dependency {} (already in --with)",
                    dep.runtime
                );
                continue;
            }

            info!(
                "Package '{}' requires runtime: {}@{}",
                package.name, dep.runtime, dep.version
            );
            tool_env = tool_env.tool(&dep.runtime, &dep.version);
        }

        let env = tool_env
            .include_vx_bin(true)
            .inherit_path(true)
            .warn_missing(false) // Don't warn for optional runtimes like bun
            .build()
            .map_err(|e| {
                ShimError::Other(anyhow::anyhow!(
                    "Failed to build runtime environment: {}",
                    e
                ))
            })?;

        let total_deps = with_deps.len() + runtime_deps.len();
        debug!(
            "Built environment with {} entries, PATH includes {} runtime bin dirs ({} from --with, {} from package)",
            env.len(),
            total_deps,
            with_deps.len(),
            runtime_deps.len()
        );

        // Log PATH for debugging
        if let Some(path) = env.get("PATH") {
            debug!("Environment PATH: {}", path);
        }

        Ok(env)
    }

    /// Get all inferred runtime dependencies for an ecosystem
    ///
    /// Some ecosystems may have multiple common dependencies.
    /// For example, npm packages may need both node and bun.
    fn infer_all_runtimes_from_ecosystem(
        &self,
        ecosystem: &str,
    ) -> Vec<vx_paths::global_packages::RuntimeDependency> {
        use vx_paths::global_packages::RuntimeDependency;

        match ecosystem.to_lowercase().as_str() {
            // Node.js ecosystem - some packages also use bun
            "npm" | "yarn" | "pnpm" => {
                let mut deps = vec![RuntimeDependency {
                    runtime: "node".to_string(),
                    version: "latest".to_string(),
                }];
                // Check if bun is installed and add it
                if vx_paths::PathManager::new()
                    .map(|pm| !pm.list_store_versions("bun").unwrap_or_default().is_empty())
                    .unwrap_or(false)
                {
                    deps.push(RuntimeDependency {
                        runtime: "bun".to_string(),
                        version: "latest".to_string(),
                    });
                }
                deps
            }
            // Python ecosystem
            "pip" | "pypi" => vec![RuntimeDependency {
                runtime: "uv".to_string(),
                version: "latest".to_string(),
            }],
            // Go ecosystem
            "go" => vec![RuntimeDependency {
                runtime: "go".to_string(),
                version: "latest".to_string(),
            }],
            // Cargo ecosystem
            "cargo" | "crates" => vec![RuntimeDependency {
                runtime: "cargo".to_string(),
                version: "latest".to_string(),
            }],
            // Ruby ecosystem
            "gem" | "rubygems" => vec![RuntimeDependency {
                runtime: "ruby".to_string(),
                version: "latest".to_string(),
            }],
            _ => vec![],
        }
    }
}
