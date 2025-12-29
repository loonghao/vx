//! CI environment detection and support
//!
//! This module provides CI provider detection and path export functionality.
//!
//! # Supported CI Providers
//!
//! - GitHub Actions
//! - GitLab CI
//! - Azure Pipelines
//! - CircleCI
//! - Jenkins
//! - Generic CI (CI=true)

mod exporter;
mod provider;

pub use exporter::{ExportResult, PathExporter};
pub use provider::CiProvider;
