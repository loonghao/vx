//! Command shim arguments (RFC 0042)

use clap::{Args as ClapArgs, Subcommand};

/// Command shim management subcommand
#[derive(Subcommand, Clone, Debug)]
pub enum ShimCommand {
    /// Expose a runtime as a directly callable command
    Add(AddShimArgs),

    /// List command shims created by vx
    #[command(alias = "ls")]
    List(ListShimArgs),

    /// Remove a command shim created by vx
    #[command(alias = "rm")]
    Remove(RemoveShimArgs),

    /// Rewrite every shim against the current vx executable
    Sync,

    /// Show the directories shims are written to and whether they are on PATH
    Path,
}

/// Arguments for `vx shim add`
#[derive(ClapArgs, Clone, Debug)]
pub struct AddShimArgs {
    /// Runtime to expose (e.g., jq, git@2.53.0, node@22)
    ///
    /// The shim forwards to `vx <runtime>`, so versioned specs work and the
    /// runtime is installed on first use.
    #[arg(required = true)]
    pub runtime: String,

    /// Command name to create (defaults to the runtime name)
    ///
    /// Use this to give the shim a different name than the runtime, for
    /// example `vx shim add jq --as jqp`.
    #[arg(long = "as", value_name = "NAME")]
    pub name: Option<String>,

    /// Directory to write the shim into (repeatable)
    ///
    /// Defaults to the vx bin directory and the directory containing the `vx`
    /// executable. The directory must be on PATH for the command to resolve.
    #[arg(long = "dir", value_name = "DIR")]
    pub dir: Vec<std::path::PathBuf>,

    /// Overwrite an existing command with the same name
    ///
    /// Required when the name already resolves to a binary that vx did not
    /// create, so `vx shim add git` cannot silently shadow the system git.
    #[arg(short, long)]
    pub force: bool,
}

/// Arguments for `vx shim list`
#[derive(ClapArgs, Clone, Debug)]
pub struct ListShimArgs {
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

/// Arguments for `vx shim remove`
#[derive(ClapArgs, Clone, Debug)]
pub struct RemoveShimArgs {
    /// Name of the shim to remove
    #[arg(required = true)]
    pub name: String,

    /// Remove even if the shim files are missing
    #[arg(short, long)]
    pub force: bool,
}
