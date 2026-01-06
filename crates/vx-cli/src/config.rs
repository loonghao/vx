//! VX Configuration Re-exports
//!
//! This module re-exports configuration types from `vx-config` crate.
//! All configuration parsing and types are centralized in `vx-config`.
//!
//! ## Migration Note
//!
//! Previously, this module contained a separate `VxConfig` implementation.
//! As part of the architecture improvements (Phase 1: Config Unification),
//! all configuration types are now defined in `vx-config` crate.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use vx_cli::config::VxConfig;
//! // or directly:
//! use vx_config::VxConfig;
//! ```

// Re-export all configuration types from vx-config
pub use vx_config::*;
