//! # vx-core
//!
//! Core abstractions and interfaces for the vx universal tool manager.
//!
//! This crate provides essential abstractions following SOLID principles:
//! - **Single Responsibility**: Each module has one clear purpose
//! - **Open/Closed**: Extensible through traits, closed for modification
//! - **Interface Segregation**: Small, focused interfaces
//! - **Dependency Inversion**: Depend on abstractions, not concretions
//!
//! ## Design Philosophy
//!
//! Following the principle of "interfaces over implementations", vx-core provides
//! only the essential abstractions. Concrete implementations live in separate crates.
//!
//! ## Example
//!
//! ```rust,no_run
//! use vx_core::{ToolManager, ExecutionContext, VxResult};
//!
//! async fn example(manager: &dyn ToolManager) -> VxResult<()> {
//!     let available = manager.is_available("node").await?;
//!     if available {
//!         let context = ExecutionContext::default();
//!         manager.execute("node", &context).await?;
//!     }
//!     Ok(())
//! }
//! ```

// Core abstractions - the only module we need
pub mod core;

// Cross-platform command execution utilities
pub mod command;

// Version parsing and comparison utilities
pub mod version_utils;

// Re-export everything from core for convenience
pub use core::*;
