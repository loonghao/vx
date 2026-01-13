//! Test command - Universal Provider Testing Framework
//!
//! Modular command structure following RFC 0020 Phase 2.

mod args;
mod handler;

pub use args::Args;
pub use handler::handle;

// Re-export TestCommand for backwards compatibility
pub use args::Args as TestCommand;
