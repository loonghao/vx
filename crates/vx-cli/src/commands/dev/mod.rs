//! Dev command implementation
//!
//! Modular command structure following RFC 0020 Phase 2.
//!
//! This command creates a shell environment with all project tools available.
//! It reads the vx.toml configuration and sets up PATH to include all
//! managed tool versions.
//!
//! ## Isolation Mode
//!
//! By default, `vx dev` runs in isolation mode where only vx-managed tools
//! are available in PATH. System tools are NOT inherited unless:
//! - `--inherit-system` flag is used
//! - `isolation = false` is set in `vx.toml` settings
//!
//! ## Environment Variable Passthrough
//!
//! In isolation mode, only essential system variables and those matching
//! `passenv` patterns are available. Configure in `vx.toml`:
//!
//! ```toml
//! [settings]
//! passenv = ["GITHUB_*", "CI", "SSH_*"]
//! ```

mod args;
mod export;
mod handler;
mod info;
mod shell;
mod tools;

pub use args::Args;
pub use export::{generate_env_export, ExportFormat};
pub use handler::build_script_environment;
pub use handler::handle;
pub use tools::get_registry;
