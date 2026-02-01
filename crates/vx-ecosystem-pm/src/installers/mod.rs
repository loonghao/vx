//! Ecosystem-specific package installers
//!
//! This module provides installers for various development ecosystem package managers:
//!
//! ## Python
//! - [`PipInstaller`] - Standard Python package manager using venv
//! - [`UvInstaller`] - Fast Python package manager (recommended)
//!
//! ## Node.js
//! - [`NpmInstaller`] - Node Package Manager
//! - [`BunInstaller`] - Fast all-in-one JavaScript runtime
//! - [`YarnInstaller`] - Fast, reliable dependency management
//! - [`PnpmInstaller`] - Disk space efficient package manager
//!
//! ## Other Ecosystems
//! - [`CargoInstaller`] - Rust package manager
//! - [`GoInstaller`] - Go package manager
//! - [`GemInstaller`] - Ruby package manager

mod bun;
mod cargo;
mod gem;
mod go;
mod npm;
mod pip;
mod pnpm;
mod uv;
mod yarn;

pub use bun::BunInstaller;
pub use cargo::CargoInstaller;
pub use gem::GemInstaller;
pub use go::GoInstaller;
pub use npm::NpmInstaller;
pub use pip::PipInstaller;
pub use pnpm::PnpmInstaller;
pub use uv::UvInstaller;
pub use yarn::YarnInstaller;

