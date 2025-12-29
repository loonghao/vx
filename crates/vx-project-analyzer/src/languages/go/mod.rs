//! Go project analyzer
//!
//! This module provides analysis for Go projects, including:
//! - Dependency detection from go.mod
//! - Script detection from common tools (go, make, etc.)
//! - Required tool detection

mod analyzer;
mod dependencies;
mod rules;

pub use analyzer::GoAnalyzer;
