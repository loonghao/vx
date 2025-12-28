//! # vx-project-analyzer
//!
//! Project analyzer for vx - detects dependencies, scripts, and tools from project configuration files.
//!
//! This crate provides:
//! - **Project Analysis**: Detect project type, dependencies, and scripts
//! - **Script Parsing**: Parse script commands to identify required tools
//! - **Dependency Detection**: Identify missing dependencies and suggest installations
//! - **Configuration Sync**: Sync project configuration with `.vx.toml`
//!
//! ## Example
//!
//! ```rust,no_run
//! use vx_project_analyzer::{ProjectAnalyzer, AnalyzerConfig};
//! use std::path::Path;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let analyzer = ProjectAnalyzer::new(AnalyzerConfig::default());
//!     let analysis = analyzer.analyze(Path::new(".")).await?;
//!
//!     println!("Detected ecosystems: {:?}", analysis.ecosystems);
//!     println!("Required tools: {:?}", analysis.required_tools);
//!
//!     Ok(())
//! }
//! ```

mod analyzer;
mod dependency;
mod ecosystem;
mod error;
mod languages;
mod script_parser;
mod sync;
mod types;

pub use analyzer::{AnalyzerConfig, ProjectAnalyzer};
pub use dependency::{Dependency, DependencySource, InstallMethod};
pub use ecosystem::Ecosystem;
pub use error::{AnalyzerError, AnalyzerResult};
pub use languages::LanguageAnalyzer;
pub use script_parser::{ScriptParser, ScriptTool, ToolInvocation};
pub use sync::{ConflictResolution, SyncAction, SyncConfig, SyncManager, VxConfigSnapshot};
pub use types::{ProjectAnalysis, RequiredTool, Script, ScriptSource};
