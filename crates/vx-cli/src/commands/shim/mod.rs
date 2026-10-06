//! Command shim management (RFC 0042)
//!
//! Exposes any runtime as a directly callable command, so `jq --version` works
//! instead of `vx jq --version`. Shims are generated per platform — Windows
//! needs a batch file and a shell script to cover cmd.exe, PowerShell and Git
//! Bash, while Unix needs a single shell script.
//!
//! Commands:
//! - `vx shim add <runtime> [--as <name>] [--dir <dir>] [--force]`
//! - `vx shim list [--json]`
//! - `vx shim remove <name>`
//! - `vx shim sync`
//! - `vx shim path`

mod args;
mod handler;

pub use args::{AddShimArgs, ListShimArgs, RemoveShimArgs, ShimCommand};
pub use handler::handle;
