//! # vx-system-pm
//!
//! System package manager integration for vx.
//!
//! This crate provides abstractions for interacting with system package managers
//! (Chocolatey, winget, Homebrew, APT, etc.) and managing system-level dependencies
//! (VCRedist, .NET Framework, Windows KB updates).
//!
//! ## Features
//!
//! - **Package Manager Abstraction**: Unified interface for multiple package managers
//! - **System Dependency Resolution**: Detect and install system prerequisites
//! - **Cross-Platform Support**: Windows, macOS, and Linux
//! - **Automatic Installation**: Install package managers if not present
//!
//! ## Example
//!
//! ```rust,ignore
//! use vx_system_pm::{PackageManagerRegistry, PackageInstallSpec};
//!
//! async fn install_git() -> anyhow::Result<()> {
//!     let registry = PackageManagerRegistry::new();
//!     
//!     // Get the best available package manager
//!     let pm = registry.get_preferred()?;
//!     
//!     // Install git with custom parameters
//!     let spec = PackageInstallSpec {
//!         package: "git".to_string(),
//!         params: Some("/GitAndUnixToolsOnPath".to_string()),
//!         ..Default::default()
//!     };
//!     
//!     pm.install_package(&spec).await?;
//!     Ok(())
//! }
//! ```

mod dependency;
mod detector;
mod error;
mod registry;
mod resolver;
mod strategy;

pub mod managers;

pub use dependency::{SystemDepType, SystemDependency, SystemDepsConfig};
pub use detector::PackageManagerDetector;
pub use error::{Result, SystemPmError};
pub use managers::{InstallResult, PackageInstallSpec, SystemPackageManager};
pub use registry::PackageManagerRegistry;
pub use resolver::{
    DependencyResolution, ResolvedDependency, SystemDependencyResolver, UnresolvedDependency,
};
pub use strategy::InstallStrategy;
