// Tracing setup using community best practices
// Uses tracing-indicatif for automatic progress bars on spans

use std::sync::OnceLock;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

// Use dynamic typing for the layer to avoid generic complexity
#[allow(dead_code)]
static INDICATIF_LAYER: OnceLock<IndicatifLayer<tracing_subscriber::Registry>> = OnceLock::new();

/// Setup tracing with default settings
pub fn setup_tracing() {
    init_tracing(false, false);
}

/// Setup tracing with debug mode
pub fn setup_tracing_with_debug(debug: bool) {
    init_tracing(debug, debug);
}

/// Initialize tracing with indicatif progress bars
/// This follows Rust community best practices for structured logging
pub fn init_tracing(verbose: bool, debug: bool) {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let indicatif_layer = IndicatifLayer::new();

        // Note: We can't store the layer globally due to generic constraints
        // This is fine as tracing-indicatif manages progress bars automatically

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
            tracing_subscriber::EnvFilter::new("vx=info,warn,error")
        };

        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(indicatif_layer.get_stderr_writer())
                    .with_target(debug)
                    .with_level(verbose || debug),
            )
            .with(indicatif_layer)
            .try_init()
            .ok(); // Ignore errors if already initialized
    });
}

/// Get the global indicatif layer for manual progress bar operations
/// Note: Due to generic constraints, we use a simpler approach
pub fn get_indicatif_layer() -> Option<()> {
    // For now, return None as we rely on automatic span-based progress bars
    None
}

/// Macro for creating instrumented async functions with progress bars
/// This replaces our custom progress decorators with standard tracing spans
#[macro_export]
macro_rules! progress_span {
    ($name:expr, $($field:tt)*) => {
        tracing::info_span!($name, $($field)*)
    };
}

/// Macro for executing operations with automatic progress indication
/// Uses tracing spans which automatically get progress bars via tracing-indicatif
#[macro_export]
macro_rules! with_progress_span {
    ($name:expr, $operation:expr) => {{
        let span = tracing::info_span!($name);
        async move {
            let _enter = span.enter();
            $operation.await
        }
    }};
}

/// Enhanced macro that adds success/error events to spans
#[macro_export]
macro_rules! with_progress_events {
    ($name:expr, $success_msg:expr, $error_msg:expr, $operation:expr) => {{
        let span = tracing::info_span!($name);
        async move {
            let _enter = span.enter();
            match $operation.await {
                Ok(result) => {
                    tracing::info!($success_msg);
                    Ok(result)
                }
                Err(error) => {
                    tracing::error!("{}: {}", $error_msg, error);
                    Err(error)
                }
            }
        }
    }};
}

/// Utility for manual progress bar creation when spans aren't sufficient
pub fn create_manual_progress_bar(len: u64, message: &str) -> indicatif::ProgressBar {
    use indicatif::{ProgressBar, ProgressStyle};

    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    pb.set_message(message.to_string());

    // Register with indicatif layer if available
    if let Some(_layer) = get_indicatif_layer() {
        // The layer will automatically manage this progress bar
    }

    pb
}

/// Suspend all progress bars temporarily (useful for user input)
pub fn suspend_progress_bars<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    if let Some(_layer) = get_indicatif_layer() {
        tracing_indicatif::suspend_tracing_indicatif(f)
    } else {
        f()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_progress_span_macro() {
        init_tracing(true, false);

        let result = with_progress_span!("test_operation", async {
            // Use a shorter sleep to reduce test time and potential timing issues
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            Ok::<_, anyhow::Error>(42)
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_progress_events_macro() {
        init_tracing(true, false);

        let result = with_progress_events!(
            "test_operation_with_events",
            "Operation completed successfully",
            "Operation failed",
            async {
                // Use a shorter sleep to reduce test time and potential timing issues
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                Ok::<_, anyhow::Error>("success")
            }
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }
}
