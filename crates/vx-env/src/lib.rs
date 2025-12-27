//! VX Environment Management
//!
//! This crate provides environment management capabilities for vx,
//! inspired by rez's shell execution model.
//!
//! # Features
//!
//! - **Script Execution**: Generate and execute platform-specific wrapper scripts
//! - **Environment Building**: Build environment variable sets for tool execution
//! - **Tool Environment**: Configure PATH with vx-managed tools
//! - **Shell Integration**: Support for multiple shells (bash, PowerShell, cmd)
//! - **Safe Command Parsing**: Parse and quote shell commands safely using `shell-words`
//!
//! # Example
//!
//! ```rust,no_run
//! use vx_env::{EnvBuilder, ToolEnvironment, execute_with_env};
//! use std::collections::HashMap;
//!
//! // Build environment with tools
//! let env = ToolEnvironment::new()
//!     .tool("node", "20.0.0")
//!     .tool("go", "1.21.0")
//!     .env_var("NODE_ENV", "production")
//!     .build()
//!     .unwrap();
//!
//! // Execute command with environment
//! let status = execute_with_env("node --version", &env).unwrap();
//! ```
//!
//! # Architecture
//!
//! ```text
//! vx-env/
//! ├── builder.rs      # Environment builder (EnvBuilder)
//! ├── executor.rs     # Script execution (execute_with_env)
//! ├── tool_env.rs     # Tool environment (ToolEnvironment)
//! ├── words.rs        # Shell command parsing (shell-words wrapper)
//! ├── error.rs        # Error types
//! └── shell/          # Shell-specific implementations
//!     ├── bash.rs
//!     ├── powershell.rs
//!     └── cmd.rs
//! ```

mod builder;
mod error;
mod executor;
pub mod shell;
mod tool_env;
mod words;

pub use builder::EnvBuilder;
pub use error::EnvError;
pub use executor::{execute_with_env, generate_wrapper_script};
pub use tool_env::{ToolEnvironment, ToolSpec};
pub use words::{join_args, parse_command, quote_arg};

/// Result type for vx-env operations
pub type Result<T> = std::result::Result<T, EnvError>;
