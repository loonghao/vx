//! Common/cross-language analyzers
//!
//! This module contains analyzers for tools that are language-agnostic,
//! such as `just`, `make`, etc.

mod justfile;

pub use justfile::JustfileAnalyzer;
