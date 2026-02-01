//! VX Shim - Smart shim system for vx global packages
//!
//! This crate provides intelligent shim execution with runtime dependency resolution.
//! It supports the RFC 0027 implicit package execution syntax:
//!
//! ```text
//! vx <ecosystem>:<package>[@version][::executable] [args...]
//! ```
//!
//! Examples:
//! - `vx npm:typescript::tsc --version`
//! - `vx pip:httpie::http GET example.com`
//! - `vx npm@20:typescript::tsc`

mod error;
mod executor;
mod request;

pub use error::{ShimError, ShimResult};
pub use executor::ShimExecutor;
pub use request::{PackageRequest, RuntimeSpec};

/// Re-export commonly used types
pub mod prelude {
    pub use super::{PackageRequest, RuntimeSpec, ShimError, ShimExecutor, ShimResult};
}
