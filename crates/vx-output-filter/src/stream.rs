//! Async stream runner — spawns a child with piped stdout/stderr
//! and applies an [`OutputFilter`] to each stream.
//!
//! stdin is always inherited so interactive tools keep working.

use crate::filter::{OutputFilter, OutputFilterConfig};
use anyhow::Result;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tracing::debug;

/// Run a child process whose stdout/stderr are piped, applying output filtering.
///
/// # Errors
/// Returns an error if the `stdout` or `stderr` handles are not available
/// (i.e. the caller forgot to set `Stdio::piped()` before spawning).
pub async fn run_filtered_child(
    mut child: Child,
    config: OutputFilterConfig,
) -> Result<std::process::ExitStatus> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("stdout not piped — set Stdio::piped() before spawn"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("stderr not piped — set Stdio::piped() before spawn"))?;

    let config_stdout = config.clone();
    let config_stderr = config;

    // Drain stdout on a separate Tokio task
    let stdout_task = tokio::spawn(async move {
        let mut filter = OutputFilter::new(config_stdout);
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            for out in filter.filter_line(&line) {
                println!("{out}");
            }
        }
        for out in filter.finalize() {
            println!("{out}");
        }
    });

    // Drain stderr on a separate Tokio task
    let stderr_task = tokio::spawn(async move {
        let mut filter = OutputFilter::new(config_stderr);
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            for out in filter.filter_line(&line) {
                eprintln!("{out}");
            }
        }
        for out in filter.finalize() {
            eprintln!("{out}");
        }
    });

    // Wait for the child process to finish, then drain readers
    let status = child.wait().await?;

    // Join the output tasks (ignore individual task errors — output already written)
    if let Err(e) = stdout_task.await {
        debug!("stdout drain task error: {e}");
    }
    if let Err(e) = stderr_task.await {
        debug!("stderr drain task error: {e}");
    }

    Ok(status)
}
