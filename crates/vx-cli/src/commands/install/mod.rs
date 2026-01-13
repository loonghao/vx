//! Install command implementation
//!
//! Modular command structure following RFC 0020 Phase 2.

mod args;
mod handler;

pub use args::Args;
pub use handler::handle;
pub use handler::handle_install;
