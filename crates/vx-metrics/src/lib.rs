//! VX Metrics - Observability for vx
//!
//! Provides unified metrics, tracing and logging for vx commands using OpenTelemetry.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐     ┌──────────────────────┐     ┌─────────────────────┐
//! │  tracing     │────▶│ tracing-opentelemetry │────▶│  OTel SpanExporter  │
//! │  (existing)  │     │ (bridge layer)        │     │  (JSON file writer) │
//! │  info_span!  │     └──────────────────────┘     └─────────┬───────────┘
//! │  debug!()    │                                            │
//! └─────────────┘     ┌──────────────────────┐     ┌──────────▼──────────┐
//!                     │  tracing-subscriber   │     │  ~/.vx/metrics/     │
//!                     │  (fmt layer - stderr) │     │  <timestamp>.json   │
//!                     └──────────────────────┘     └─────────────────────┘
//! ```
//!
//! # Output
//!
//! Each `vx` command execution writes a JSON metrics file to `~/.vx/metrics/`:
//!
//! ```json
//! {
//!   "version": "1",
//!   "timestamp": "2026-02-07T10:30:00Z",
//!   "command": "vx node --version",
//!   "exit_code": 0,
//!   "total_duration_ms": 1234,
//!   "stages": {
//!     "resolve": { "duration_ms": 50 },
//!     "ensure": { "duration_ms": 800 },
//!     "prepare": { "duration_ms": 10 },
//!     "execute": { "duration_ms": 374 }
//!   },
//!   "spans": [...]
//! }
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! // Initialize at CLI startup
//! let _guard = vx_metrics::init(vx_metrics::MetricsConfig {
//!     debug: false,
//!     verbose: false,
//!     command: "vx node --version".to_string(),
//! });
//!
//! // Existing tracing code works as-is
//! tracing::info_span!("resolve").in_scope(|| { /* ... */ });
//!
//! // On exit, guard flushes spans to JSON file
//! ```

pub mod exporter;
pub mod init;
pub mod report;
pub mod visualize;

pub use init::{init, MetricsConfig, MetricsGuard};
pub use report::{CommandMetrics, StageMetrics};
pub use visualize::{
    generate_ai_summary, generate_html_report, load_metrics, render_comparison, render_insights,
    render_summary,
};
