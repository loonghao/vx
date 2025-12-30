//! Framework detection for desktop and cross-platform applications
//!
//! This module provides detection for application frameworks like:
//! - Electron (JavaScript/TypeScript desktop apps)
//! - Tauri (Rust + Web desktop apps)
//!
//! Frameworks are detected on top of language ecosystems and provide
//! additional context about project structure and required tools.

mod electron;
mod tauri;
mod types;

pub use electron::ElectronDetector;
pub use tauri::TauriDetector;
pub use types::{FrameworkInfo, ProjectFramework};

use crate::dependency::Dependency;
use crate::error::AnalyzerResult;
use crate::types::{RequiredTool, Script};
use async_trait::async_trait;
use std::path::Path;

/// Trait for framework detectors
#[async_trait]
pub trait FrameworkDetector: Send + Sync {
    /// Check if this framework is present in the project
    fn detect(&self, root: &Path) -> bool;

    /// Get the framework type
    fn framework(&self) -> ProjectFramework;

    /// Get detailed framework information
    async fn get_info(&self, root: &Path) -> AnalyzerResult<FrameworkInfo>;

    /// Get additional required tools for this framework
    fn required_tools(&self, deps: &[Dependency], scripts: &[Script]) -> Vec<RequiredTool>;

    /// Get additional scripts provided by this framework
    async fn additional_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>>;
}

/// Get all available framework detectors
pub fn all_framework_detectors() -> Vec<Box<dyn FrameworkDetector>> {
    vec![
        Box::new(ElectronDetector::new()),
        Box::new(TauriDetector::new()),
    ]
}
