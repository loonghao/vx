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
//! - **Shell Spawning**: Unified shell spawning for dev and env commands
//! - **Session Management**: Unified session context for shell environments
//! - **Embedded Assets**: Shell init scripts embedded at compile time via rust-embed
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
//! # Shell Spawning Example
//!
//! ```rust,no_run
//! use vx_env::{SessionContext, ShellSpawner, IsolationConfig};
//!
//! // Create a session context
//! let session = SessionContext::new("my-project")
//!     .tool("node", "20.0.0")
//!     .tool("go", "1.21.0")
//!     .isolated(true);
//!
//! // Spawn a shell
//! let spawner = ShellSpawner::new(session).unwrap();
//! spawner.spawn_interactive(None).unwrap();
//! ```
//!
//! # Architecture
//!
//! ```text
//! vx-env/
//! ├── assets.rs       # Embedded shell scripts (rust-embed)
//! ├── builder.rs      # Environment builder (EnvBuilder)
//! ├── executor.rs     # Script execution (execute_with_env)
//! ├── tool_env.rs     # Tool environment (ToolEnvironment)
//! ├── session.rs      # Session context (SessionContext)
//! ├── spawner.rs      # Shell spawner (ShellSpawner)
//! ├── words.rs        # Shell command parsing (shell-words wrapper)
//! ├── error.rs        # Error types
//! └── shell/          # Shell-specific implementations
//!     ├── bash.rs
//!     ├── powershell.rs
//!     └── cmd.rs
//! ```

pub mod assets;
mod builder;
pub mod context;
pub mod env_assembler;
mod error;
mod executor;
pub mod session;
pub mod shell;
pub mod spawner;
mod tool_env;
mod words;

pub use assets::ShellScript;
pub use builder::EnvBuilder;
pub use context::{ContextOverride, EnvContext};
pub use env_assembler::{EnvAssembler, EnvOperation, EnvVar, priority};
pub use error::EnvError;
pub use executor::{execute_with_env, generate_wrapper_script};
pub use session::{IsolationConfig, SessionContext, SessionSource};
pub use spawner::{ExportFormat, ShellSpawner, detect_shell, print_exit, print_welcome};
pub use tool_env::{ToolEnvironment, ToolSpec};
pub use words::{join_args, parse_command, quote_arg};

/// Result type for vx-env operations
pub type Result<T> = std::result::Result<T, EnvError>;
