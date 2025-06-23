//! Progress tracking for downloads

use std::time::{Duration, Instant};

/// Progress callback function type
pub type ProgressCallback = Box<dyn Fn(ProgressInfo) + Send + Sync>;

/// Progress information for downloads
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    /// Total bytes to download (if known)
    pub total_bytes: Option<u64>,
    /// Bytes downloaded so far
    pub downloaded_bytes: u64,
    /// Download speed in bytes per second
    pub speed_bps: f64,
    /// Percentage completed (0.0 to 100.0)
    pub percentage: f64,
    /// Estimated time remaining
    pub eta: Option<Duration>,
    /// Time elapsed since download started
    pub elapsed: Duration,
    /// Current filename being downloaded
    pub filename: String,
}

impl ProgressInfo {
    /// Create a new progress info
    pub fn new(filename: String) -> Self {
        Self {
            total_bytes: None,
            downloaded_bytes: 0,
            speed_bps: 0.0,
            percentage: 0.0,
            eta: None,
            elapsed: Duration::from_secs(0),
            filename,
        }
    }

    /// Update progress with new downloaded bytes
    pub fn update(&mut self, downloaded_bytes: u64, elapsed: Duration) {
        self.downloaded_bytes = downloaded_bytes;
        self.elapsed = elapsed;

        // Calculate speed
        if elapsed.as_secs_f64() > 0.0 {
            self.speed_bps = downloaded_bytes as f64 / elapsed.as_secs_f64();
        }

        // Calculate percentage
        if let Some(total) = self.total_bytes {
            if total > 0 {
                self.percentage = (downloaded_bytes as f64 / total as f64) * 100.0;

                // Calculate ETA
                if self.speed_bps > 0.0 {
                    let remaining_bytes = total.saturating_sub(downloaded_bytes);
                    let eta_seconds = remaining_bytes as f64 / self.speed_bps;
                    self.eta = Some(Duration::from_secs_f64(eta_seconds));
                }
            }
        }
    }

    /// Set total bytes
    pub fn set_total_bytes(&mut self, total_bytes: u64) {
        self.total_bytes = Some(total_bytes);
    }

    /// Get speed in MB/s
    pub fn speed_mbps(&self) -> f64 {
        self.speed_bps / 1_000_000.0
    }

    /// Get human-readable speed
    pub fn speed_human(&self) -> String {
        let speed = self.speed_bps;
        if speed >= 1_000_000_000.0 {
            format!("{:.1} GB/s", speed / 1_000_000_000.0)
        } else if speed >= 1_000_000.0 {
            format!("{:.1} MB/s", speed / 1_000_000.0)
        } else if speed >= 1_000.0 {
            format!("{:.1} KB/s", speed / 1_000.0)
        } else {
            format!("{:.0} B/s", speed)
        }
    }

    /// Get human-readable size
    pub fn size_human(&self) -> String {
        let bytes = self.downloaded_bytes;
        if bytes >= 1_000_000_000 {
            format!("{:.1} GB", bytes as f64 / 1_000_000_000.0)
        } else if bytes >= 1_000_000 {
            format!("{:.1} MB", bytes as f64 / 1_000_000.0)
        } else if bytes >= 1_000 {
            format!("{:.1} KB", bytes as f64 / 1_000.0)
        } else {
            format!("{} B", bytes)
        }
    }

    /// Get human-readable ETA
    pub fn eta_human(&self) -> String {
        match self.eta {
            Some(eta) => {
                let total_seconds = eta.as_secs();
                if total_seconds >= 3600 {
                    format!("{}h {}m", total_seconds / 3600, (total_seconds % 3600) / 60)
                } else if total_seconds >= 60 {
                    format!("{}m {}s", total_seconds / 60, total_seconds % 60)
                } else {
                    format!("{}s", total_seconds)
                }
            }
            None => "Unknown".to_string(),
        }
    }
}

/// Progress tracker for managing download progress
pub struct ProgressTracker {
    start_time: Instant,
    last_update: Instant,
    info: ProgressInfo,
    callback: Option<ProgressCallback>,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(filename: String, callback: Option<ProgressCallback>) -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_update: now,
            info: ProgressInfo::new(filename),
            callback,
        }
    }

    /// Set total bytes
    pub fn set_total_bytes(&mut self, total_bytes: u64) {
        self.info.set_total_bytes(total_bytes);
    }

    /// Update progress
    pub fn update(&mut self, downloaded_bytes: u64) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.start_time);

        self.info.update(downloaded_bytes, elapsed);
        self.last_update = now;

        // Call callback if provided
        if let Some(ref callback) = self.callback {
            callback(self.info.clone());
        }
    }

    /// Get current progress info
    pub fn info(&self) -> &ProgressInfo {
        &self.info
    }
}
