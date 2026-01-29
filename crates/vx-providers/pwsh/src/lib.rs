//! PowerShell provider for vx
//!
//! This provider manages PowerShell 7+ (pwsh) installation and execution.
//! PowerShell is a cross-platform shell and scripting language.
//! https://github.com/PowerShell/PowerShell

pub mod config;
mod provider;
mod runtime;

pub use config::PwshUrlBuilder;
pub use provider::{create_provider, PwshProvider};
pub use runtime::PwshRuntime;
