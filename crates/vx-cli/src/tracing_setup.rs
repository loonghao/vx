//! Tracing setup for vx CLI
//!
//! This module configures structured logging using tracing.
//! Progress bars are handled separately by the UI module to avoid conflicts.

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Setup tracing with default settings
pub fn setup_tracing() {
    init_tracing(false, false);
}

/// Setup tracing with debug mode
pub fn setup_tracing_with_debug(debug: bool) {
    init_tracing(debug, debug);
}

/// Initialize tracing without indicatif integration
///
/// We use pure indicatif for progress bars to avoid conflicts and duplicated output.
/// Tracing is used only for structured logging.
pub fn init_tracing(verbose: bool, debug: bool) {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        // Priority: debug > verbose > default
        // Also respect RUST_LOG environment variable
        let env_filter = if std::env::var("RUST_LOG").is_ok() {
            // Use RUST_LOG if set
            tracing_subscriber::EnvFilter::from_default_env()
        } else if debug {
            tracing_subscriber::EnvFilter::new("debug")
        } else if verbose {
            tracing_subscriber::EnvFilter::new("vx=debug,info")
        } else {
            // In normal mode, only show warnings and errors
            // Progress information is handled by indicatif
            tracing_subscriber::EnvFilter::new("warn,error")
        };

        if debug {
            // Debug mode: clean hierarchical format for stderr
            // Use pretty format with minimal noise
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_writer(std::io::stderr) // Write to stderr to separate from tool output
                        .with_target(false) // Hide target for cleaner output
                        .with_level(true)
                        .with_thread_ids(false)
                        .with_thread_names(false)
                        .with_file(false)
                        .with_line_number(false)
                        .without_time()
                        .with_ansi(true),
                )
                .try_init()
                .ok();
        } else {
            // Normal/verbose mode: simple format
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_target(verbose)
                        .with_level(verbose)
                        .without_time(),
                )
                .try_init()
                .ok();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_tracing() {
        // Just ensure it doesn't panic
        init_tracing(false, false);
        init_tracing(true, false);
        init_tracing(false, true);
    }
}
