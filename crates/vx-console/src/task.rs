//! Task execution with timing statistics.
//!
//! This module provides utilities for executing tasks with automatic
//! timing and progress reporting.

use std::future::Future;
use std::time::{Duration, Instant};

/// Result of a timed task execution.
#[derive(Debug, Clone)]
pub struct TaskResult<T> {
    /// The result value.
    pub value: T,
    /// Time taken to execute the task.
    pub duration: Duration,
}

impl<T> TaskResult<T> {
    /// Create a new task result.
    pub fn new(value: T, duration: Duration) -> Self {
        Self { value, duration }
    }

    /// Get the duration as a human-readable string.
    pub fn duration_string(&self) -> String {
        format_duration(self.duration)
    }

    /// Map the value to a new type.
    pub fn map<U, F>(self, f: F) -> TaskResult<U>
    where
        F: FnOnce(T) -> U,
    {
        TaskResult {
            value: f(self.value),
            duration: self.duration,
        }
    }
}

/// A timed task executor.
#[derive(Debug, Clone)]
pub struct TimedTask {
    name: String,
    start: Option<Instant>,
}

impl TimedTask {
    /// Create a new timed task.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            start: None,
        }
    }

    /// Start the task timer.
    pub fn start(&mut self) {
        self.start = Some(Instant::now());
    }

    /// Get the elapsed time since start.
    pub fn elapsed(&self) -> Duration {
        self.start.map(|s| s.elapsed()).unwrap_or(Duration::ZERO)
    }

    /// Get the task name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Execute a synchronous task with timing.
    pub fn execute<F, T, E>(_name: &str, f: F) -> Result<TaskResult<T>, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let start = Instant::now();
        let result = f()?;
        let duration = start.elapsed();
        Ok(TaskResult::new(result, duration))
    }

    /// Execute an async task with timing.
    pub async fn execute_async<F, Fut, T, E>(name: &str, f: F) -> Result<TaskResult<T>, E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        let _ = name; // Used for potential logging
        let start = Instant::now();
        let result = f().await?;
        let duration = start.elapsed();
        Ok(TaskResult::new(result, duration))
    }
}

/// Format a duration as a human-readable string.
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs_f64();

    if secs < 0.001 {
        format!("{:.0}µs", duration.as_micros())
    } else if secs < 1.0 {
        format!("{:.0}ms", duration.as_millis())
    } else if secs < 60.0 {
        format!("{:.1}s", secs)
    } else if secs < 3600.0 {
        let mins = (secs / 60.0).floor();
        let remaining_secs = secs % 60.0;
        format!("{}m {:.0}s", mins as u64, remaining_secs)
    } else {
        let hours = (secs / 3600.0).floor();
        let remaining_mins = ((secs % 3600.0) / 60.0).floor();
        format!("{}h {}m", hours as u64, remaining_mins as u64)
    }
}

/// Format bytes as a human-readable string.
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    }
}

/// Format a speed (bytes per second) as a human-readable string.
pub fn format_speed(bytes_per_sec: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;

    if bytes_per_sec < KB {
        format!("{:.0} B/s", bytes_per_sec)
    } else if bytes_per_sec < MB {
        format!("{:.1} KB/s", bytes_per_sec / KB)
    } else {
        format!("{:.1} MB/s", bytes_per_sec / MB)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_micros() {
        let d = Duration::from_micros(500);
        assert_eq!(format_duration(d), "500µs");
    }

    #[test]
    fn test_format_duration_millis() {
        let d = Duration::from_millis(500);
        assert_eq!(format_duration(d), "500ms");
    }

    #[test]
    fn test_format_duration_secs() {
        let d = Duration::from_secs_f64(2.5);
        assert_eq!(format_duration(d), "2.5s");
    }

    #[test]
    fn test_format_duration_mins() {
        let d = Duration::from_secs(125);
        assert_eq!(format_duration(d), "2m 5s");
    }

    #[test]
    fn test_format_duration_hours() {
        let d = Duration::from_secs(3725);
        assert_eq!(format_duration(d), "1h 2m");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1_572_864), "1.5 MB");
        assert_eq!(format_bytes(1_610_612_736), "1.50 GB");
    }

    #[test]
    fn test_format_speed() {
        assert_eq!(format_speed(500.0), "500 B/s");
        assert_eq!(format_speed(1536.0), "1.5 KB/s");
        assert_eq!(format_speed(1_572_864.0), "1.5 MB/s");
    }

    #[test]
    fn test_task_result() {
        let result = TaskResult::new(42, Duration::from_secs(2));
        assert_eq!(result.value, 42);
        assert_eq!(result.duration_string(), "2.0s");
    }

    #[test]
    fn test_task_result_map() {
        let result = TaskResult::new(42, Duration::from_secs(1));
        let mapped = result.map(|v| v * 2);
        assert_eq!(mapped.value, 84);
    }

    #[test]
    fn test_timed_task_execute() {
        let result: Result<TaskResult<i32>, ()> = TimedTask::execute("test", || Ok(42));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, 42);
    }
}
