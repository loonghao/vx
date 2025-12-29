//! Node.js project analyzer
//!
//! This module provides analysis for Node.js projects, including:
//! - Dependency detection from package.json
//! - Script detection from package.json and common tools
//! - Package manager detection (npm, yarn, pnpm, bun)

mod analyzer;
mod dependencies;
pub mod package_manager;
mod rules;
mod scripts;

pub use analyzer::NodeJsAnalyzer;
