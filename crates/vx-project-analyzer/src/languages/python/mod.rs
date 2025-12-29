//! Python project analyzer
//!
//! This module provides analysis for Python projects, including:
//! - Dependency detection from pyproject.toml and requirements.txt
//! - Script detection from pyproject.toml and common tools (nox, pytest, etc.)
//! - Required tool detection

mod analyzer;
mod dependencies;
mod rules;
mod scripts;

pub use analyzer::PythonAnalyzer;
