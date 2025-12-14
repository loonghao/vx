//! Progress reporting utilities for installation operations

use crate::Result;
use std::sync::Arc;

/// Progress reporting interface for installation operations
#[async_trait::async_trait]
pub trait ProgressReporter: Send + Sync {
    /// Start a new progress operation
    async fn start(&self, message: &str, total: Option<u64>);

    /// Update progress with current position
    async fn update(&self, position: u64, message: Option<&str>);

    /// Increment progress by a delta
    async fn increment(&self, delta: u64);

    /// Finish the progress operation
    async fn finish(&self, message: &str);

    /// Finish with an error message
    async fn finish_with_error(&self, message: &str);

    /// Set the total size (useful when total is unknown initially)
    async fn set_total(&self, total: u64);
}

/// Progress style configuration
#[derive(Debug, Clone)]
pub struct ProgressStyle {
    /// Template for progress display
    pub template: String,
    /// Characters used for progress bar
    pub progress_chars: String,
    /// Whether to show elapsed time
    pub show_elapsed: bool,
    /// Whether to show ETA
    pub show_eta: bool,
    /// Whether to show transfer rate
    pub show_rate: bool,
}

impl Default for ProgressStyle {
    fn default() -> Self {
        Self {
            template: "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})".to_string(),
            progress_chars: "#>-".to_string(),
            show_elapsed: true,
            show_eta: true,
            show_rate: true,
        }
    }
}

impl ProgressStyle {
    /// Create a simple progress style
    pub fn simple() -> Self {
        Self {
            template: "{wide_bar} {pos}/{len}".to_string(),
            progress_chars: "=>-".to_string(),
            show_elapsed: false,
            show_eta: false,
            show_rate: false,
        }
    }

    /// Create a detailed progress style with all information
    pub fn detailed() -> Self {
        Self::default()
    }

    /// Create a minimal progress style
    pub fn minimal() -> Self {
        Self {
            template: "{spinner} {msg}".to_string(),
            progress_chars: "⠁⠂⠄⡀⢀⠠⠐⠈".to_string(),
            show_elapsed: false,
            show_eta: false,
            show_rate: false,
        }
    }
}

/// Console-based progress reporter using indicatif
#[cfg(feature = "progress")]
pub struct ConsoleProgressReporter {
    bar: std::sync::Mutex<Option<indicatif::ProgressBar>>,
    style: ProgressStyle,
}

#[cfg(feature = "progress")]
impl ConsoleProgressReporter {
    /// Create a new console progress reporter
    pub fn new(style: ProgressStyle) -> Self {
        Self {
            bar: std::sync::Mutex::new(None),
            style,
        }
    }
}

#[cfg(feature = "progress")]
impl Default for ConsoleProgressReporter {
    fn default() -> Self {
        Self::new(ProgressStyle::default())
    }
}

#[cfg(feature = "progress")]
#[async_trait::async_trait]
impl ProgressReporter for ConsoleProgressReporter {
    async fn start(&self, message: &str, total: Option<u64>) {
        use indicatif::{ProgressBar, ProgressStyle as IndicatifStyle};

        let bar = if let Some(total) = total {
            ProgressBar::new(total)
        } else {
            ProgressBar::new_spinner()
        };

        let style = IndicatifStyle::with_template(&self.style.template)
            .unwrap_or_else(|_| IndicatifStyle::default_bar())
            .progress_chars(&self.style.progress_chars);

        bar.set_style(style);
        bar.set_message(message.to_string());

        // Store the bar
        if let Ok(mut bar_guard) = self.bar.lock() {
            *bar_guard = Some(bar);
        }
    }

    async fn update(&self, position: u64, message: Option<&str>) {
        if let Ok(bar_guard) = self.bar.lock() {
            if let Some(ref bar) = *bar_guard {
                bar.set_position(position);
                if let Some(msg) = message {
                    bar.set_message(msg.to_string());
                }
            }
        }
    }

    async fn increment(&self, delta: u64) {
        if let Ok(bar_guard) = self.bar.lock() {
            if let Some(ref bar) = *bar_guard {
                bar.inc(delta);
            }
        }
    }

    async fn finish(&self, message: &str) {
        if let Ok(mut bar_guard) = self.bar.lock() {
            if let Some(bar) = bar_guard.take() {
                bar.finish_with_message(message.to_string());
            }
        }
    }

    async fn finish_with_error(&self, message: &str) {
        if let Ok(mut bar_guard) = self.bar.lock() {
            if let Some(bar) = bar_guard.take() {
                bar.finish_with_message(format!("❌ {}", message));
            }
        }
    }

    async fn set_total(&self, total: u64) {
        if let Ok(bar_guard) = self.bar.lock() {
            if let Some(ref bar) = *bar_guard {
                bar.set_length(total);
            }
        }
    }
}

/// No-op progress reporter for when progress reporting is disabled
pub struct NoOpProgressReporter;

#[async_trait::async_trait]
impl ProgressReporter for NoOpProgressReporter {
    async fn start(&self, _message: &str, _total: Option<u64>) {}
    async fn update(&self, _position: u64, _message: Option<&str>) {}
    async fn increment(&self, _delta: u64) {}
    async fn finish(&self, _message: &str) {}
    async fn finish_with_error(&self, _message: &str) {}
    async fn set_total(&self, _total: u64) {}
}

/// Create a progress reporter based on configuration
pub fn create_progress_reporter(style: ProgressStyle, enabled: bool) -> Arc<dyn ProgressReporter> {
    if enabled {
        #[cfg(feature = "progress")]
        {
            Arc::new(ConsoleProgressReporter::new(style))
        }
        #[cfg(not(feature = "progress"))]
        {
            let _ = style;
            Arc::new(NoOpProgressReporter)
        }
    } else {
        Arc::new(NoOpProgressReporter)
    }
}

/// Progress context for tracking multiple operations
pub struct ProgressContext {
    reporter: Arc<dyn ProgressReporter>,
    enabled: bool,
}

impl ProgressContext {
    /// Create a new progress context
    pub fn new(reporter: Arc<dyn ProgressReporter>, enabled: bool) -> Self {
        Self { reporter, enabled }
    }

    /// Create a disabled progress context
    pub fn disabled() -> Self {
        Self {
            reporter: Arc::new(NoOpProgressReporter),
            enabled: false,
        }
    }

    /// Get the progress reporter
    pub fn reporter(&self) -> &Arc<dyn ProgressReporter> {
        &self.reporter
    }

    /// Check if progress reporting is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Start a new progress operation
    pub async fn start(&self, message: &str, total: Option<u64>) -> Result<()> {
        if self.enabled {
            self.reporter.start(message, total).await;
        }
        Ok(())
    }

    /// Update progress
    pub async fn update(&self, position: u64, message: Option<&str>) -> Result<()> {
        if self.enabled {
            self.reporter.update(position, message).await;
        }
        Ok(())
    }

    /// Increment progress
    pub async fn increment(&self, delta: u64) -> Result<()> {
        if self.enabled {
            self.reporter.increment(delta).await;
        }
        Ok(())
    }

    /// Finish progress
    pub async fn finish(&self, message: &str) -> Result<()> {
        if self.enabled {
            self.reporter.finish(message).await;
        }
        Ok(())
    }
}
