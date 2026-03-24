//! # vx-ecosystem-pm
//!
//! Development ecosystem package managers for vx.
//!
//! This crate provides abstractions for installing packages from development
//! ecosystem package managers (npm, pip, cargo, uv, bun, yarn, pnpm, go, gem)
//! with isolation support.
//!
//! ## RFC 0025: Cross-Language Global Package Isolation
//!
//! This implements the package installation portion of RFC 0025, providing:
//! - Isolated package installations (no global pollution)
//! - Content-addressable storage structure
//! - Executable detection for shim creation
//! - Support for alternative package managers (uv for pip, bun/yarn/pnpm for npm)
//!
//! ## Supported Ecosystems
//!
//! ### Python
//! - `pip` - Standard Python package manager
//! - `uv` - Fast Python package manager (recommended)
//!
//! ### Node.js
//! - `npm` - Node Package Manager
//! - `bun` - Fast all-in-one JavaScript runtime
//! - `yarn` - Fast, reliable, and secure dependency management
//! - `pnpm` - Fast, disk space efficient package manager
//!
//! ### Windows
//! - `choco` - Chocolatey package manager
//!
//! ### Other
//! - `cargo` - Rust package manager
//! - `go` - Go package manager
//! - `gem` - Ruby package manager
//!
//! ## Example
//!
//! ```rust,ignore
//! use vx_ecosystem_pm::{get_installer, InstallOptions};
//!
//! async fn install_typescript() -> anyhow::Result<()> {
//!     let install_dir = std::path::PathBuf::from("/path/to/install");
//!     let installer = get_installer("npm")?;
//!     let options = InstallOptions::default();
//!     let result = installer.install(&install_dir, "typescript", "5.3.0", &options).await?;
//!     println!("Installed executables: {:?}", result.executables);
//!     Ok(())
//! }
//! ```

mod error;
mod traits;
mod types;
mod utils;

pub mod installers;

pub use error::{EcosystemPmError, Result};
pub use traits::EcosystemInstaller;
pub use types::{EcosystemInstallResult, InstallEnv, InstallOptions};
pub use utils::{detect_executables_in_dir, run_command};

use anyhow::bail;

/// Get an ecosystem installer for the specified ecosystem
///
/// # Arguments
/// * `ecosystem` - The ecosystem name
///
/// # Supported ecosystems
/// - Python: `pip`, `uv`, `python`, `pypi`
/// - Node.js: `npm`, `node`, `bun`, `yarn`, `pnpm`
/// - Rust: `cargo`, `rust`, `crates`
/// - Go: `go`, `golang`
/// - Ruby: `gem`, `ruby`, `rubygems`
///
/// # Returns
/// A boxed trait object implementing `EcosystemInstaller`
///
/// # Example
/// ```rust,ignore
/// let installer = get_installer("uv")?;
/// ```
pub fn get_installer(ecosystem: &str) -> anyhow::Result<Box<dyn EcosystemInstaller>> {
    use installers::*;

    match ecosystem.to_lowercase().as_str() {
        // Python ecosystem
        "pip" | "python" | "pypi" => Ok(Box::new(PipInstaller::new())),
        "uv" => Ok(Box::new(UvInstaller::new())),
        // uvx: run Python CLI tools in isolated, ephemeral uv-managed environments
        "uvx" => Ok(Box::new(UvxInstaller::new())),

        // Node.js ecosystem
        // npx is a package runner bundled with npm, treat it as npm ecosystem
        "npm" | "node" | "npx" => Ok(Box::new(NpmInstaller::new())),
        // bunx is bun's package runner, treat it as bun ecosystem
        "bun" | "bunx" => Ok(Box::new(BunInstaller::new())),
        "yarn" => Ok(Box::new(YarnInstaller::new())),
        "pnpm" => Ok(Box::new(PnpmInstaller::new())),
        // dlx: pnpm's oneshot runner (like npx)
        "dlx" => Ok(Box::new(DlxInstaller::new())),

        // Deno ecosystem
        // deno: run npm/JSR packages via deno run
        "deno" => Ok(Box::new(DenoInstaller::new())),

        // .NET ecosystem
        // dotnet-tool: install and run .NET tools via dotnet tool install
        "dotnet-tool" | "dotnet" => Ok(Box::new(DotnetToolInstaller::new())),

        // Java ecosystem
        // jbang: run Java tools in isolated, cached environments
        "jbang" | "java" => Ok(Box::new(JBangInstaller::new())),

        // Rust ecosystem
        "cargo" | "rust" | "crates" => Ok(Box::new(CargoInstaller::new())),

        // Go ecosystem
        "go" | "golang" => Ok(Box::new(GoInstaller::new())),

        // Ruby ecosystem
        "gem" | "ruby" | "rubygems" => Ok(Box::new(GemInstaller::new())),

        // Windows ecosystem
        "choco" | "chocolatey" => Ok(Box::new(ChocoInstaller::new())),

        _ => bail!(
            "Unsupported ecosystem: {}. Supported: pip, uv, uvx, npm, npx, bun, bunx, yarn, pnpm, dlx, deno, dotnet-tool, jbang, cargo, go, gem, choco",
            ecosystem
        ),
    }
}

/// Get the preferred installer for an ecosystem
///
/// This returns the recommended/faster installer for each ecosystem:
/// - Python: uv (if available), falls back to pip
/// - Node.js/npm: bun (if available), falls back to npm
///
/// The preference order follows the same pattern as Python's uv→pip fallback:
/// prefer the faster, more modern tool, with automatic fallback to the standard tool.
///
/// # Arguments
/// * `ecosystem` - The base ecosystem name (python, node, rust, go, ruby)
pub fn get_preferred_installer(ecosystem: &str) -> anyhow::Result<Box<dyn EcosystemInstaller>> {
    use installers::*;

    match ecosystem.to_lowercase().as_str() {
        "python" | "pip" | "pypi" => {
            // Prefer uv if available
            let uv = UvInstaller::new();
            if uv.is_available() {
                Ok(Box::new(uv))
            } else {
                Ok(Box::new(PipInstaller::new()))
            }
        }
        "npm" | "node" | "npx" => {
            // Prefer bun if available (faster), fall back to npm
            let bun = BunInstaller::new();
            if bun.is_available() {
                Ok(Box::new(bun))
            } else {
                Ok(Box::new(NpmInstaller::new()))
            }
        }
        // For other ecosystems, use the default
        other => get_installer(other),
    }
}
