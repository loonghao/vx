//! Unified initialization for tracing + OpenTelemetry metrics.
//!
//! Replaces the existing `tracing_setup.rs` with a single `init()` call
//! that sets up:
//! 1. `tracing-subscriber` fmt layer (stderr output, with per-layer filter)
//! 2. `tracing-opentelemetry` layer (bridges tracing spans → OTel spans, always captures vx=trace)
//! 3. `JsonFileExporter` (collects spans in memory)
//!
//! Per-layer filtering ensures that debug/trace messages are only captured by
//! the OTel layer for metrics, while the fmt layer only shows warn/error in
//! normal mode (or debug/trace in --verbose/--debug mode).
//!
//! On drop, `MetricsGuard` flushes all spans to a JSON file under `~/.vx/metrics/`.

use anyhow::Result;
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::exporter::JsonFileExporter;
use crate::report::CommandMetrics;

/// Configuration for metrics initialization.
#[derive(Debug, Clone, Default)]
pub struct MetricsConfig {
    /// Enable debug-level tracing output
    pub debug: bool,
    /// Enable verbose tracing output
    pub verbose: bool,
    /// The command string being executed (for report metadata)
    pub command: String,
    /// Custom metrics output directory (defaults to ~/.vx/metrics)
    pub metrics_dir: Option<PathBuf>,
}

/// Guard that flushes metrics to disk on drop.
///
/// Hold this in `main()` — when the program exits, the guard writes
/// the collected spans to `~/.vx/metrics/<timestamp>.json`.
pub struct MetricsGuard {
    exporter: JsonFileExporter,
    provider: SdkTracerProvider,
    start_time: Instant,
    command: String,
    metrics_dir: PathBuf,
    exit_code: Arc<std::sync::atomic::AtomicI32>,
}

impl MetricsGuard {
    /// Set the exit code for the metrics report.
    pub fn set_exit_code(&self, code: i32) {
        self.exit_code
            .store(code, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get the exit code Arc for sharing with other code.
    pub fn exit_code_handle(&self) -> Arc<std::sync::atomic::AtomicI32> {
        self.exit_code.clone()
    }

    /// Force flush and write the metrics report now.
    pub fn flush(&self) {
        if let Err(e) = self.write_report() {
            eprintln!("[vx-metrics] Failed to write metrics report: {}", e);
        }
    }

    fn write_report(&self) -> Result<()> {
        // Flush the tracer provider to ensure all spans are exported
        let _ = self.provider.force_flush();

        let spans = self.exporter.take_spans();

        // Skip writing if no spans were collected (e.g., --help, list)
        if spans.is_empty() {
            return Ok(());
        }

        let elapsed = self.start_time.elapsed();
        let exit_code = self.exit_code.load(std::sync::atomic::Ordering::Relaxed);

        let mut metrics = CommandMetrics::new(self.command.clone());
        metrics.exit_code = Some(exit_code);
        metrics.total_duration_ms = elapsed.as_secs_f64() * 1000.0;
        metrics.spans = spans;
        metrics.extract_stages_from_spans();

        // Ensure metrics directory exists
        std::fs::create_dir_all(&self.metrics_dir)?;

        // Write to timestamped JSON file
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S_%3f");
        let filename = format!("{}.json", timestamp);
        let filepath = self.metrics_dir.join(&filename);

        let json = serde_json::to_string_pretty(&metrics)?;
        std::fs::write(&filepath, &json)?;

        // Clean up old metrics files (keep last 50)
        cleanup_old_metrics(&self.metrics_dir, 50);

        Ok(())
    }
}

impl Drop for MetricsGuard {
    fn drop(&mut self) {
        self.flush();
    }
}

/// Initialize the unified tracing + OpenTelemetry metrics system.
///
/// Returns a `MetricsGuard` that must be held for the lifetime of the program.
/// When dropped, it writes the metrics report to `~/.vx/metrics/`.
///
/// # Example
///
/// ```rust,ignore
/// let _guard = vx_metrics::init(vx_metrics::MetricsConfig {
///     debug: false,
///     verbose: false,
///     command: "vx node --version".to_string(),
///     ..Default::default()
/// });
/// ```
pub fn init(config: MetricsConfig) -> MetricsGuard {
    use std::sync::Once;
    static INIT: Once = Once::new();

    let exporter = JsonFileExporter::new();
    let exporter_clone = exporter.clone();

    // Build OpenTelemetry tracer provider with our in-memory JSON exporter
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter_clone)
        .build();

    let tracer = provider.tracer("vx");

    let metrics_dir = config
        .metrics_dir
        .clone()
        .unwrap_or_else(default_metrics_dir);
    let start_time = Instant::now();
    let command = config.command.clone();

    // Build the tracing subscriber with per-layer filtering.
    //
    // The OTel layer needs vx=trace to capture all spans for metrics,
    // but the fmt layer should only show user-relevant messages (warn/error
    // in normal mode). Using per-layer filters ensures debug/trace messages
    // don't leak to stderr in normal operation.
    INIT.call_once(|| {
        // Fmt layer filter: controls what the user sees on stderr.
        let fmt_filter_str = fmt_filter_directive(&config);
        let fmt_filter = tracing_subscriber::EnvFilter::new(&fmt_filter_str);

        // OTel layer filter: always capture all vx spans for metrics collection.
        let otel_filter_str = otel_filter_directive();
        let otel_filter = tracing_subscriber::EnvFilter::new(&otel_filter_str);

        // OpenTelemetry layer (captures spans and exports to our exporter)
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        // Fmt layer: always write to stderr to separate from tool output.
        // Adjust verbosity via format options.
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr)
            .with_target(config.debug || config.verbose)
            .with_level(config.debug || config.verbose)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_file(false)
            .with_line_number(false)
            .without_time()
            .with_ansi(true);

        tracing_subscriber::registry()
            .with(fmt_layer.with_filter(fmt_filter))
            .with(otel_layer.with_filter(otel_filter))
            .try_init()
            .ok();
    });

    MetricsGuard {
        exporter,
        provider,
        start_time,
        command,
        metrics_dir,
        exit_code: Arc::new(std::sync::atomic::AtomicI32::new(0)),
    }
}

/// Get the default metrics directory (~/.vx/metrics).
fn default_metrics_dir() -> PathBuf {
    vx_paths::VxPaths::default().base_dir.join("metrics")
}

/// Build the fmt layer filter string based on config.
///
/// The fmt filter controls what the user sees on stderr:
/// - Normal mode: only warn and error
/// - Verbose mode: vx debug messages + info from other crates
/// - Debug mode: all debug messages
/// - RUST_LOG env: user-specified filter
///
/// This is separate from the OTel filter to enable per-layer filtering.
pub fn fmt_filter_directive(config: &MetricsConfig) -> String {
    if std::env::var("RUST_LOG").is_ok() {
        std::env::var("RUST_LOG").unwrap_or_default()
    } else if config.debug {
        "debug".to_string()
    } else if config.verbose {
        "vx=debug,info".to_string()
    } else {
        "warn,error".to_string()
    }
}

/// Build the OTel layer filter string.
///
/// The OTel filter always captures all vx spans (trace level) for metrics
/// collection, regardless of the user's verbosity preference.
/// Only overridden if RUST_LOG is explicitly set.
pub fn otel_filter_directive() -> String {
    if std::env::var("RUST_LOG").is_ok() {
        std::env::var("RUST_LOG").unwrap_or_default()
    } else {
        "vx=trace,warn,error".to_string()
    }
}

/// Clean up old metrics files, keeping only the most recent `keep` files.
fn cleanup_old_metrics(dir: &std::path::Path, keep: usize) {
    let mut files: Vec<_> = match std::fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect(),
        Err(_) => return,
    };

    if files.len() <= keep {
        return;
    }

    // Sort by name (timestamps sort lexicographically)
    files.sort_by_key(|e| e.file_name());

    // Remove oldest files
    let to_remove = files.len() - keep;
    for entry in files.into_iter().take(to_remove) {
        let _ = std::fs::remove_file(entry.path());
    }
}
