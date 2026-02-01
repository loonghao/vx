//! Shim executor for running globally installed package executables

use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, warn};

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

        self.execute_package_shim(&package, exe_name, args).await
    }

    /// Execute a package request (RFC 0027 syntax)
    ///
    /// This handles the full `ecosystem:package@version::executable` syntax.
    pub async fn execute_request(
        &self,
        request: &PackageRequest,
        args: &[String],
    ) -> ShimResult<i32> {
        let exe_name = request.executable_name();

        // Load the package registry
        let registry = PackageRegistry::load(&self.registry_path).map_err(|e| {
            ShimError::Other(anyhow::anyhow!("Failed to load package registry: {}", e))
        })?;

        // Try to find the package
        if let Some(package) = registry.get(&request.ecosystem, &request.package) {
            // Package is installed, execute it
            if let Some(exit_code) = self.execute_package_shim(&package, exe_name, args).await? {
                return Ok(exit_code);
            }
            // Shim not found for this executable
            return Err(ShimError::ExecutableNotFound {
                package: request.package.clone(),
                executable: exe_name.to_string(),
            });
        }

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
    async fn execute_package_shim(
        &self,
        package: &GlobalPackage,
        exe_name: &str,
        args: &[String],
    ) -> ShimResult<Option<i32>> {
        // Check if shim exists
        if !shims::shim_exists(&self.shims_dir, exe_name) {
            warn!(
                "Package '{}' provides '{}' but shim is missing",
                package.name, exe_name
            );
            return Ok(None);
        }

        // Get the shim path and execute it
        let shim_path = shims::get_shim_path(&self.shims_dir, exe_name);

        debug!(
            "Executing shim: {} (from {}:{})",
            exe_name, package.ecosystem, package.name
        );

        // Execute the shim
        let status = Command::new(&shim_path)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await?;

        Ok(Some(status.code().unwrap_or(1)))
    }
}

