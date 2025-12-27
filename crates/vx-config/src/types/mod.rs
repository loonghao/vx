//! Configuration type definitions
//!
//! This module defines all configuration structures for `.vx.toml` files.
//! All fields are optional to maintain backward compatibility.
//!
//! ## Module Structure
//!
//! Types are organized by feature area:
//! - `config`: Root `VxConfig` struct
//! - `project`: Project metadata
//! - `tool`: Tool version and configuration
//! - `python`: Python environment configuration
//! - `env`: Environment variables and secrets
//! - `script`: Script definitions
//! - `settings`: Behavior settings
//! - `hooks`: Lifecycle hooks
//! - `service`: Service definitions
//! - `dependencies`: Dependency management
//! - `ai`: AI integration
//! - `docs`: Documentation generation
//! - `team`: Team collaboration
//! - `remote`: Remote development
//! - `security`: Security scanning
//! - `test`: Test pipeline
//! - `telemetry`: Telemetry configuration
//! - `container`: Container deployment
//! - `versioning`: Versioning strategy

mod ai;
mod config;
mod container;
mod dependencies;
mod docs;
mod env;
mod hooks;
mod project;
mod python;
mod remote;
mod script;
mod security;
mod service;
mod settings;
mod team;
mod telemetry;
mod test;
mod tool;
mod versioning;

// Re-export all types
pub use ai::*;
pub use config::*;
pub use container::*;
pub use dependencies::*;
pub use docs::*;
pub use env::*;
pub use hooks::*;
pub use project::*;
pub use python::*;
pub use remote::*;
pub use script::*;
pub use security::*;
pub use service::*;
pub use settings::*;
pub use team::*;
pub use telemetry::*;
pub use test::*;
pub use tool::*;
pub use versioning::*;
