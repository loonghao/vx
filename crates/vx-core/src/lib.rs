//! # vx-core
//!
//! Core utilities and types for the vx universal tool manager.
//!
//! This crate provides foundational utilities shared across the vx codebase:
//!
//! - `WithDependency` — Runtime dependency spec for the `--with` flag
//! - Process exit status utilities (`is_ctrl_c_exit`, `exit_code_from_status`)
//! - Version resolution helpers (`is_latest_version`, `resolve_latest_version`)
//! - Cross-platform command execution (`command` module)
//! - Version parsing and comparison (`version_utils` module)
//!
//! ## Design Philosophy
//!
//! Runtime and Provider abstractions live in `vx-runtime` (the `Runtime` trait)
//! and `vx-starlark` (the provider.star DSL engine). This crate only contains
//! lightweight, dependency-free utilities.

// Core types and utilities
pub mod core;

// Cross-platform command execution utilities
pub mod command;

// Version parsing and comparison utilities
pub mod version_utils;

// Re-export everything from core for convenience
pub use core::*;
