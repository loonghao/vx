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
//! - [`DlxInstaller`] - pnpm dlx oneshot runner (like npx)
//!
//! ## Deno
//! - [`DenoInstaller`] - Run npm/JSR packages via deno run
//!
//! ## .NET
//! - [`DotnetToolInstaller`] - Install and run .NET tools via dotnet tool install
//!
//! ## Java
//! - [`JBangInstaller`] - Run Java tools via jbang
//!
//! ## Other Ecosystems
//! - [`CargoInstaller`] - Rust package manager
//! - [`GoInstaller`] - Go package manager
//! - [`GemInstaller`] - Ruby package manager

mod bun;
mod cargo;
mod choco;
mod deno;
mod dlx;
mod dotnet_tool;
mod gem;
mod go;
mod jbang;
mod npm;
mod pip;
mod pnpm;
mod uv;
mod uvx;
mod yarn;

pub use bun::BunInstaller;
pub use cargo::CargoInstaller;
pub use choco::ChocoInstaller;
pub use deno::DenoInstaller;
pub use dlx::DlxInstaller;
pub use dotnet_tool::DotnetToolInstaller;
pub use gem::GemInstaller;
pub use go::GoInstaller;
pub use jbang::JBangInstaller;
pub use npm::NpmInstaller;
pub use pip::PipInstaller;
pub use pnpm::PnpmInstaller;
pub use uv::UvInstaller;
pub use uvx::UvxInstaller;
pub use yarn::YarnInstaller;
