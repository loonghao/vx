//! # vx-args
//!
//! Argument parsing and variable interpolation for vx scripts and extensions.
//!
//! This crate provides:
//! - Declarative argument definitions
//! - Positional and named argument parsing
//! - Variable interpolation with `{{var}}` syntax
//! - Built-in variables and command interpolation
//! - Automatic help generation
//!
//! ## Example
//!
//! ```rust
//! use vx_args::{ArgDef, ArgParser, ArgType};
//!
//! let mut parser = ArgParser::new("deploy");
//! parser.positional(ArgDef::new("environment")
//!     .required(true)
//!     .choices(vec!["dev", "staging", "prod"])
//!     .help("Target environment"));
//! parser.add_arg(ArgDef::new("region")
//!     .default("us-east-1")
//!     .help("Cloud region"));
//! parser.add_arg(ArgDef::new("verbose")
//!     .arg_type(ArgType::Flag)
//!     .short('v')
//!     .help("Enable verbose output"));
//!
//! let args = vec!["prod", "--region", "us-west-2", "-v"];
//! let parsed = parser.parse(&args).unwrap();
//!
//! assert_eq!(parsed.get_string("environment"), Some("prod"));
//! assert_eq!(parsed.get_string("region"), Some("us-west-2"));
//! assert_eq!(parsed.get_bool("verbose"), Some(true));
//! ```

mod error;
mod help;
mod interpolation;
mod parser;
mod types;

pub use error::{ArgError, ArgResult};
pub use help::HelpFormatter;
pub use interpolation::{Interpolator, VarSource};
pub use parser::{ArgParser, ParsedArgs};
pub use types::{ArgDef, ArgType, ArgValue};
