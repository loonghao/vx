//! JSON file span exporter for OpenTelemetry
//!
//! Collects completed spans in memory and writes them to a JSON file
//! under `~/.vx/metrics/` when `shutdown()` is called.

use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::trace::{SpanData, SpanExporter};
use std::fmt;
use std::sync::{Arc, Mutex};

/// In-memory span collector that buffers spans for later JSON serialization.
#[derive(Clone)]
pub struct JsonFileExporter {
    spans: Arc<Mutex<Vec<SpanRecord>>>,
}

impl fmt::Debug for JsonFileExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.spans.lock().map(|s| s.len()).unwrap_or(0);
        f.debug_struct("JsonFileExporter")
            .field("span_count", &count)
            .finish()
    }
}

/// A simplified span record for JSON serialization.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpanRecord {
    pub name: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: String,
    pub start_time_unix_ns: u64,
    pub end_time_unix_ns: u64,
    pub duration_ms: f64,
    pub status: String,
    pub attributes: std::collections::HashMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<SpanEvent>,
}

/// A simplified span event for JSON serialization.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp_unix_ns: u64,
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub attributes: std::collections::HashMap<String, serde_json::Value>,
}

impl JsonFileExporter {
    pub fn new() -> Self {
        Self {
            spans: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Take all collected spans (drains the buffer).
    pub fn take_spans(&self) -> Vec<SpanRecord> {
        let mut spans = self.spans.lock().unwrap_or_else(|e| e.into_inner());
        std::mem::take(&mut *spans)
    }

    fn convert_span(span: &SpanData) -> SpanRecord {
        use opentelemetry::trace::Status;

        let start_ns = span
            .start_time
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let end_ns = span
            .end_time
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let duration_ms = (end_ns.saturating_sub(start_ns)) as f64 / 1_000_000.0;

        let status = match &span.status {
            Status::Unset => "unset".to_string(),
            Status::Ok => "ok".to_string(),
            Status::Error { description } => format!("error: {}", description),
        };

        let mut attributes = std::collections::HashMap::new();
        for kv in span.attributes.iter() {
            attributes.insert(
                kv.key.to_string(),
                serde_json::Value::String(kv.value.to_string()),
            );
        }

        let events = span
            .events
            .iter()
            .map(|e| {
                let ts = e
                    .timestamp
                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64;
                let mut attrs = std::collections::HashMap::new();
                for kv in &e.attributes {
                    attrs.insert(
                        kv.key.to_string(),
                        serde_json::Value::String(kv.value.to_string()),
                    );
                }
                SpanEvent {
                    name: e.name.to_string(),
                    timestamp_unix_ns: ts,
                    attributes: attrs,
                }
            })
            .collect();

        SpanRecord {
            name: span.name.to_string(),
            trace_id: span.span_context.trace_id().to_string(),
            span_id: span.span_context.span_id().to_string(),
            parent_span_id: span.parent_span_id.to_string(),
            start_time_unix_ns: start_ns,
            end_time_unix_ns: end_ns,
            duration_ms,
            status,
            attributes,
            events,
        }
    }
}

impl Default for JsonFileExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl SpanExporter for JsonFileExporter {
    fn export(
        &self,
        batch: Vec<SpanData>,
    ) -> impl std::future::Future<Output = OTelSdkResult> + Send {
        let records: Vec<SpanRecord> = batch.iter().map(Self::convert_span).collect();
        let spans = self.spans.clone();
        async move {
            let mut guard = spans.lock().unwrap_or_else(|e| e.into_inner());
            guard.extend(records);
            Ok(())
        }
    }

    fn shutdown(&mut self) -> OTelSdkResult {
        Ok(())
    }
}
