//! Version Solver Module
//!
//! This module provides a universal version resolver that supports:
//! - Partial version matching (e.g., "3.11" → "3.11.11")
//! - Version constraint expressions (e.g., ">=3.9,<3.12")
//! - Lock file mechanism for reproducible environments
//! - Multi-ecosystem support with different version semantics
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      Version Solver                          │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────────────┐    ┌─────────────────┐                 │
//! │  │  VersionSolver  │───▶│  SolverStatus   │                 │
//! │  └────────┬────────┘    └─────────────────┘                 │
//! │           │                                                  │
//! │           ▼                                                  │
//! │  ┌─────────────────┐    ┌─────────────────┐                 │
//! │  │ VersionRequest  │───▶│ ResolvedVersion │                 │
//! │  └────────┬────────┘    └─────────────────┘                 │
//! │           │                                                  │
//! │           ▼                                                  │
//! │  ┌─────────────────┐    ┌─────────────────┐                 │
//! │  │VersionStrategy  │───▶│   Ecosystem     │                 │
//! │  │  (per ecosystem)│    └─────────────────┘                 │
//! │  └─────────────────┘                                        │
//! └─────────────────────────────────────────────────────────────┘
//! ```

mod constraint;
mod request;
mod resolved;
mod solver;
mod strategy;

pub use constraint::{RangeConstraint, RangeOp, Version, VersionConstraint};
pub use request::VersionRequest;
pub use resolved::ResolvedVersion;
pub use solver::{SolverConfig, SolverError, SolverResult, SolverStatus, VersionSolver};
pub use strategy::{SemverStrategy, VersionStrategy};
