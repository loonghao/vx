//! Bridge runner — spawns the target process and forwards exit code.

use std::path::Path;
use std::process::{Command, ExitCode};

/// Run the bridge: spawn the target executable with the given args and return its exit code.
pub fn run_bridge(executable: &Path, prefix_args: &[&str], caller_args: &[String]) -> ExitCode {
    let mut cmd = Command::new(executable);

    // Add prefix args (e.g., "msbuild" for `dotnet msbuild`)
    for arg in prefix_args {
        cmd.arg(arg);
    }

    // Forward all caller args
    cmd.args(caller_args);

    match cmd.status() {
        Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
        Err(e) => {
            eprintln!(
                "vx bridge: failed to execute {}: {}",
                executable.display(),
                e
            );
            ExitCode::from(1)
        }
    }
}
