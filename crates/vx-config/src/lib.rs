//! VX Configuration Management
//!
//! This crate provides typed configuration parsing for `.vx.toml` files.
//! It supports both v1 (legacy) and v2 (enhanced) configuration formats.
//!
//! # Example
//!
//! ```rust,ignore
//! use vx_config::VxConfig;
//!
//! let config = VxConfig::from_file(".vx.toml")?;
//! println!("Tools: {:?}", config.tools);
//! ```

mod container;
mod dependencies;
mod error;
mod hooks;
mod inheritance;
mod migration;
mod parser;
mod remote;
mod security;
mod team;
mod telemetry;
mod testing;
mod types;
mod validation;

pub use container::{
    ContainerManager, DockerfileGenerator, GitInfo, GoDockerConfig, NodejsDockerConfig,
    PythonDockerConfig, RustDockerConfig,
};
pub use dependencies::{AuditResult, AutoUpdateStrategy, DependencyManager, RegistryPresets};
pub use error::{ConfigError, ConfigResult};
pub use hooks::{EnterHookManager, GitHookInstaller, HookExecutor, HookResult};
pub use inheritance::{InheritanceManager, LockEntry, LockFile, MergeStrategy, PresetSource};
pub use migration::{ConfigMigrator, ConfigVersion, MigrationOptions, MigrationResult};
pub use parser::{parse_config, parse_config_str};
pub use remote::RemoteGenerator;
pub use security::{
    generate_report as generate_security_report, LicenseViolation, SecretFinding,
    SecurityScanResult, SecurityScanner, Severity, Vulnerability,
};
pub use team::TeamManager;
pub use telemetry::{BuildTiming, BuildTracker, Metric, OtlpExporter, Span, TelemetryCollector};
pub use testing::{CoverageReporter, TestFramework, TestResult, TestRunner};
pub use types::*;
pub use validation::{validate_config, ValidationResult};

/// Re-export for convenience
pub use schemars;
