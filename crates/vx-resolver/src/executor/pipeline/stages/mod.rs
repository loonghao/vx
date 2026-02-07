//! Concrete pipeline stage implementations (RFC 0029)
//!
//! Each stage is in its own submodule:
//! - `resolve` - Version resolution and dependency analysis
//! - `ensure` - Installation of missing runtimes
//! - `prepare` - Environment preparation
//! - `execute` - Command execution

pub mod ensure;
pub mod execute;
pub mod prepare;
pub mod resolve;

// Re-export stage types
pub use ensure::EnsureStage;
pub use execute::ExecuteStage;
pub use prepare::{PrepareStage, PreparedExecution};
pub use resolve::{ResolveRequest, ResolveStage, WithDepRequest};
