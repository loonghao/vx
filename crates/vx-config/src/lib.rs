//! VX Configuration Management
//!
//! This crate provides typed configuration parsing for `vx.toml` files.
//! It supports both v1 (legacy) and v2 (enhanced) configuration formats.
//!
//! # Example
//!
//! ```rust,ignore
//! use vx_config::VxConfig;
//!
//! let config = VxConfig::from_file("vx.toml")?;
//! println!("Tools: {:?}", config.tools);
//! ```
//!
//! ## Config Manager
//!
//! For format-preserving edits that maintain comments and formatting:
//!
//! ```rust,ignore
//! use vx_config::config_manager::{ConfigManager, TomlDocument};
//!
//! // Load and edit while preserving formatting
//! let mut doc = TomlDocument::parse(content)?;
//! doc.set_string("tools.node", "22");
//! ```

pub mod config_manager;
mod container;
mod dependencies;
mod error;
mod hooks;
mod inheritance;
mod migration;
mod parser;
mod remote;
mod security;
mod setup_pipeline;
mod team;
mod telemetry;
mod testing;
mod types;
mod validation;

pub use container::{
    ContainerManager, DockerfileGenerator, GitInfo, GoDockerConfig, NodejsDockerConfig,
    PythonDockerConfig, RustDockerConfig, generate_dockerfile,
};
pub use dependencies::{AuditResult, AutoUpdateStrategy, DependencyManager, RegistryPresets};
pub use error::{ConfigError, ConfigResult};
pub use hooks::{EnterHookManager, GitHookInstaller, HookExecutor, HookResult};
pub use inheritance::{InheritanceManager, LockEntry, LockFile, MergeStrategy, PresetSource};
pub use migration::{ConfigMigrator, ConfigVersion, MigrationOptions, MigrationResult};
pub use parser::{parse_config, parse_config_str};
pub use remote::{RemoteGenerator, generate_devcontainer_json, generate_gitpod_yml};
pub use security::{
    LicenseViolation, ScanStatus, SecretFinding, SecurityScanResult, SecurityScanner, Severity,
    Vulnerability, generate_report as generate_security_report, patterns,
};
pub use setup_pipeline::{SetupHookResult, SetupPipeline, SetupPipelineResult};
pub use team::{TeamManager, generate_codeowners};
pub use telemetry::{BuildTiming, BuildTracker, Metric, OtlpExporter, Span, TelemetryCollector};
pub use testing::{CoverageReporter, TestFramework, TestResult, TestRunner};
pub use types::*;
pub use validation::{ValidationResult, validate_config};

/// Re-export for convenience (only available with "schema" feature)
#[cfg(feature = "schema")]
pub use schemars;
