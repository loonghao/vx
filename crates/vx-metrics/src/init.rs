//! Unified initialization for tracing + OpenTelemetry metrics.
//!
//! Replaces the existing `tracing_setup.rs` with a single `init()` call
//! that sets up:
//! 1. `tracing-subscriber` fmt layer (stderr output, like before)
//! 2. `tracing-opentelemetry` layer (bridges tracing spans → OTel spans)
//! 3. `JsonFileExporter` (collects spans in memory)
//!
//! On drop, `MetricsGuard` flushes all spans to a JSON file under `~/.vx/metrics/`.

use anyhow::Result;
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
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
        let exit_code = self
            .exit_code
            .load(std::sync::atomic::Ordering::Relaxed);

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

    // Build the tracing subscriber with OpenTelemetry layer.
    //
    // All branches use stderr for the fmt layer to keep tool stdout clean.
    // The EnvFilter controls console verbosity; the OTel layer captures
    // all vx spans regardless (since the filter includes vx=trace).
    INIT.call_once(|| {
        let env_filter = if std::env::var("RUST_LOG").is_ok() {
            tracing_subscriber::EnvFilter::from_default_env()
        } else if config.debug {
            tracing_subscriber::EnvFilter::new("debug")
        } else if config.verbose {
            tracing_subscriber::EnvFilter::new("vx=debug,info")
        } else {
            // Normal mode: console only sees warn/error,
            // but vx=trace allows OTel to capture all spans
            tracing_subscriber::EnvFilter::new("vx=trace,warn,error")
        };

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
            .with(env_filter)
            .with(fmt_layer)
            .with(otel_layer)
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
