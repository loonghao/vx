//! # vx-bridge — Generic command bridge/forwarder for vx-managed tools
//!
//! Some third-party tools (e.g., node-gyp) expect specific executables (e.g., `MSBuild.exe`)
//! at specific paths. When vx manages these tools differently (e.g., via `dotnet msbuild`),
//! we need stub executables that **look like** the expected tool but **delegate** to the
//! actual vx-managed implementation.
//!
//! This crate provides a generic, cross-platform framework for creating such bridges.
//!
//! ## Quick Start
//!
//! Create a bridge executable in just a few lines:
//!
//! ```rust,no_run
//! use vx_bridge::BridgeConfig;
//!
//! fn main() -> std::process::ExitCode {
//!     // MSBuild.exe → dotnet msbuild <args>
//!     BridgeConfig::new("MSBuild")
//!         .target_vx_runtime("dotnet")
//!         .prefix_args(&["msbuild"])
//!         .system_search_paths(&[
//!             r"C:\Program Files\dotnet\dotnet.exe",
//!         ])
//!         .run()
//! }
//! ```
//!
//! ## How It Works
//!
//! 1. **Find the target executable** — searches vx store first, then system paths, then PATH
//! 2. **Build the command** — prepends any prefix args, then appends the caller's args
//! 3. **Execute** — spawns the process with inherited stdio, returns its exit code
//!
//! ## Cross-Platform
//!
//! - On Windows, searches for `.exe` files
//! - On Unix, searches for files without extension (or with platform-appropriate extension)
//! - PATH separator is handled automatically (`;` on Windows, `:` on Unix)

mod config;
mod deployer;
mod embedded;
mod finder;
mod runner;

pub use config::BridgeConfig;
pub use deployer::{DeployError, deploy_bridge};
pub use embedded::{deploy_embedded_bridge, register_embedded_bridge};
pub use finder::ExecutableFinder;
pub use runner::run_bridge;
