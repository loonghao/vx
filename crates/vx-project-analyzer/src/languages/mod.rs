//! Language-specific analyzers

mod nodejs;
mod python;
mod rust;

pub use nodejs::NodeJsAnalyzer;
pub use python::PythonAnalyzer;
pub use rust::RustAnalyzer;

use crate::dependency::Dependency;
use crate::error::AnalyzerResult;
use crate::types::{RequiredTool, Script};
use async_trait::async_trait;
use std::path::Path;

/// Trait for language-specific analyzers
#[async_trait]
pub trait LanguageAnalyzer: Send + Sync {
    /// Check if this analyzer applies to the project
    fn detect(&self, root: &Path) -> bool;

    /// Get the analyzer name
    fn name(&self) -> &'static str;

    /// Analyze project dependencies
    async fn analyze_dependencies(&self, root: &Path) -> AnalyzerResult<Vec<Dependency>>;

    /// Analyze project scripts
    async fn analyze_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>>;

    /// Get required tools based on analysis
    fn required_tools(&self, deps: &[Dependency], scripts: &[Script]) -> Vec<RequiredTool>;

    /// Generate install command for a dependency
    fn install_command(&self, dep: &Dependency) -> Option<String>;
}

/// Get all available language analyzers
pub fn all_analyzers() -> Vec<Box<dyn LanguageAnalyzer>> {
    vec![
        Box::new(PythonAnalyzer::new()),
        Box::new(NodeJsAnalyzer::new()),
        Box::new(RustAnalyzer::new()),
    ]
}
