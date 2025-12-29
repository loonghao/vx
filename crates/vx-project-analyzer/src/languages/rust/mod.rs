//! Rust project analyzer
//!
//! This module provides analysis for Rust projects, including:
//! - Dependency detection from Cargo.toml
//! - Script detection from common tools (cargo, just, etc.)
//! - Required tool detection

mod analyzer;
mod dependencies;
mod rules;
mod scripts;

pub use analyzer::RustAnalyzer;
