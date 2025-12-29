//! C++ project analyzer
//!
//! This module provides analysis for C++ projects, including:
//! - Detection via CMakeLists.txt, Makefile, meson.build
//! - Dependency detection from CMake find_package/FetchContent
//! - Script detection from common build commands

mod analyzer;
mod dependencies;
mod rules;

pub use analyzer::CppAnalyzer;
