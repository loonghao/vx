//! Progress bars and spinners.
//!
//! This module provides unified progress reporting using indicatif.

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Mutex;
use std::time::Duration;

/// Progress manager for handling multiple progress bars.
#[derive(Debug)]
pub struct ProgressManager {
    multi: MultiProgress,
    active_bars: Mutex<Vec<ProgressBar>>,
}

impl Default for ProgressManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressManager {
    /// Create a new progress manager.
    pub fn new() -> Self {
        Self {
            multi: MultiProgress::new(),
            active_bars: Mutex::new(Vec::new()),
        }
    }

    /// Add a spinner with the given message.
    pub fn add_spinner(&self, message: &str) -> ManagedSpinner {
        let bar = self.multi.add(ProgressBar::new_spinner());
        bar.set_style(
            ProgressStyle::with_template("  {spinner:.green} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✓"]),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(80));

        if let Ok(mut bars) = self.active_bars.lock() {
            bars.push(bar.clone());
        }

        ManagedSpinner { bar }
    }

    /// Add a download progress bar.
    pub fn add_download(&self, total_size: u64, message: &str) -> ManagedDownload {
        let bar = self.multi.add(ProgressBar::new(total_size));
        bar.set_style(
            ProgressStyle::with_template(
                "  {spinner:.green} {msg} {wide_bar:.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
            )
            .unwrap()
            .progress_chars("━━╺"),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(100));

        if let Ok(mut bars) = self.active_bars.lock() {
            bars.push(bar.clone());
        }

        ManagedDownload { bar }
    }

    /// Add a task progress bar.
    pub fn add_task(&self, total: u64, message: &str) -> ManagedTask {
        let bar = self.multi.add(ProgressBar::new(total));
        bar.set_style(
            ProgressStyle::with_template(
                "  {spinner:.green} {msg} [{bar:40.cyan/blue}] {pos}/{len}",
            )
            .unwrap()
            .progress_chars("━━╺"),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(100));

        if let Ok(mut bars) = self.active_bars.lock() {
            bars.push(bar.clone());
        }

        ManagedTask { bar }
    }

    /// Clear all active progress bars.
    pub fn clear_all(&self) {
        if let Ok(mut bars) = self.active_bars.lock() {
            for bar in bars.drain(..) {
                bar.finish_and_clear();
            }
        }
    }

    /// Suspend progress bars and execute a function.
    pub fn suspend<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.multi.suspend(f)
    }

    /// Get the underlying MultiProgress.
    pub fn multi(&self) -> &MultiProgress {
        &self.multi
    }
}

/// A managed spinner that auto-removes from the manager.
#[derive(Debug)]
pub struct ManagedSpinner {
    bar: ProgressBar,
}

impl ManagedSpinner {
    /// Set the spinner message.
    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    /// Finish with a message.
    pub fn finish_with_message(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }

    /// Finish and clear the spinner.
    pub fn finish_and_clear(&self) {
        self.bar.finish_and_clear();
    }

    /// Finish with a success message.
    pub fn finish_success(&self, message: &str) {
        self.bar.set_style(
            ProgressStyle::with_template("  {spinner:.green} {msg}")
                .unwrap()
                .tick_strings(&["✓"]),
        );
        self.bar
            .finish_with_message(format!("\x1b[32m✓\x1b[0m {}", message));
    }

    /// Finish with an error message.
    pub fn finish_error(&self, message: &str) {
        self.bar.set_style(
            ProgressStyle::with_template("  {spinner:.red} {msg}")
                .unwrap()
                .tick_strings(&["✗"]),
        );
        self.bar
            .finish_with_message(format!("\x1b[31m✗\x1b[0m {}", message));
    }
}

/// A managed download progress bar.
#[derive(Debug)]
pub struct ManagedDownload {
    bar: ProgressBar,
}

impl ManagedDownload {
    /// Set the total size.
    pub fn set_length(&self, len: u64) {
        self.bar.set_length(len);
    }

    /// Set the current position.
    pub fn set_position(&self, pos: u64) {
        self.bar.set_position(pos);
    }

    /// Increment the position.
    pub fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }

    /// Set the message.
    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    /// Finish with a message.
    pub fn finish_with_message(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }

    /// Finish and clear.
    pub fn finish_and_clear(&self) {
        self.bar.finish_and_clear();
    }
}

/// A managed task progress bar.
#[derive(Debug)]
pub struct ManagedTask {
    bar: ProgressBar,
}

impl ManagedTask {
    /// Set the current position.
    pub fn set_position(&self, pos: u64) {
        self.bar.set_position(pos);
    }

    /// Increment the position.
    pub fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }

    /// Set the message.
    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    /// Finish with a message.
    pub fn finish_with_message(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }

    /// Finish and clear.
    pub fn finish_and_clear(&self) {
        self.bar.finish_and_clear();
    }
}

/// Simple spinner (standalone, not managed).
#[derive(Debug)]
pub struct ProgressSpinner {
    bar: ProgressBar,
}

impl ProgressSpinner {
    /// Create a new spinner.
    pub fn new(message: &str) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::with_template("  {spinner:.green} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✓"]),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(80));
        Self { bar }
    }

    /// Create a download spinner.
    pub fn new_download(message: &str) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::with_template("  {spinner:.green} Downloading {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✓"]),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(80));
        Self { bar }
    }

    /// Create an install spinner.
    pub fn new_install(message: &str) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::with_template("  {spinner:.green} Installing {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✓"]),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(80));
        Self { bar }
    }

    /// Set the message.
    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    /// Finish with a message.
    pub fn finish_with_message(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }

    /// Finish and clear.
    pub fn finish_and_clear(&self) {
        self.bar.finish_and_clear();
    }

    /// Finish with an error.
    pub fn finish_with_error(&self, message: &str) {
        self.bar.set_style(
            ProgressStyle::with_template("  {spinner:.red} {msg}")
                .unwrap()
                .tick_strings(&["✗"]),
        );
        self.bar
            .finish_with_message(format!("\x1b[31m✗\x1b[0m {}", message));
    }
}

/// Download progress bar (standalone).
#[derive(Debug)]
pub struct DownloadProgress {
    bar: ProgressBar,
}

impl DownloadProgress {
    /// Create a new download progress bar.
    pub fn new(total_size: u64, message: &str) -> Self {
        let bar = ProgressBar::new(total_size);
        bar.set_style(
            ProgressStyle::with_template(
                "  {spinner:.green} {msg} {wide_bar:.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
            )
            .unwrap()
            .progress_chars("━━╺"),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(100));
        Self { bar }
    }

    /// Create a download progress bar with unknown size.
    pub fn new_unknown(message: &str) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::with_template("  {spinner:.green} {msg} {bytes} ({bytes_per_sec})")
                .unwrap(),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(100));
        Self { bar }
    }

    /// Set the total size.
    pub fn set_length(&self, len: u64) {
        self.bar.set_length(len);
    }

    /// Set the current position.
    pub fn set_position(&self, pos: u64) {
        self.bar.set_position(pos);
    }

    /// Increment the position.
    pub fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }

    /// Set the message.
    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    /// Finish with a message.
    pub fn finish_with_message(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }

    /// Finish and clear.
    pub fn finish_and_clear(&self) {
        self.bar.finish_and_clear();
    }
}

/// Multi-step progress.
#[derive(Debug)]
pub struct MultiStepProgress {
    steps: Vec<String>,
    current: usize,
    bar: ProgressBar,
}

impl MultiStepProgress {
    /// Create a new multi-step progress.
    pub fn new(steps: Vec<String>) -> Self {
        let total = steps.len() as u64;
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::with_template("  {spinner:.green} [{pos}/{len}] {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✓"]),
        );

        if let Some(first) = steps.first() {
            bar.set_message(first.clone());
        }
        bar.enable_steady_tick(Duration::from_millis(80));

        Self {
            steps,
            current: 0,
            bar,
        }
    }

    /// Move to the next step.
    pub fn next_step(&mut self) {
        self.current += 1;
        self.bar.inc(1);
        if let Some(step) = self.steps.get(self.current) {
            self.bar.set_message(step.clone());
        }
    }

    /// Finish the progress.
    pub fn finish(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }
}

/// Install progress for multiple tools.
#[derive(Debug)]
pub struct InstallProgress {
    multi: MultiProgress,
    main_bar: ProgressBar,
    current_bar: Option<ProgressBar>,
    #[allow(dead_code)]
    total: u64,
    completed: u64,
}

impl InstallProgress {
    /// Create a new install progress.
    pub fn new(total_tools: u64, title: &str) -> Self {
        let multi = MultiProgress::new();
        let main_bar = multi.add(ProgressBar::new(total_tools));
        main_bar.set_style(
            ProgressStyle::with_template("{msg} [{bar:40.cyan/blue}] {pos}/{len}")
                .unwrap()
                .progress_chars("━━╺"),
        );
        main_bar.set_message(title.to_string());

        Self {
            multi,
            main_bar,
            current_bar: None,
            total: total_tools,
            completed: 0,
        }
    }

    /// Start installing a tool.
    pub fn start_tool(&mut self, name: &str) {
        if let Some(bar) = self.current_bar.take() {
            bar.finish_and_clear();
        }

        let bar = self.multi.add(ProgressBar::new_spinner());
        bar.set_style(
            ProgressStyle::with_template("  {spinner:.green} Installing {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✓"]),
        );
        bar.set_message(name.to_string());
        bar.enable_steady_tick(Duration::from_millis(80));
        self.current_bar = Some(bar);
    }

    /// Complete the current tool.
    pub fn complete_tool(&mut self, success: bool) {
        if let Some(bar) = self.current_bar.take() {
            if success {
                bar.finish_with_message("✓".to_string());
            } else {
                bar.finish_with_message("✗".to_string());
            }
        }
        self.completed += 1;
        self.main_bar.set_position(self.completed);
    }

    /// Finish the install progress.
    pub fn finish(&self, message: &str) {
        self.main_bar.finish_with_message(message.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_manager_new() {
        let pm = ProgressManager::new();
        // Just verify it doesn't panic
        let _ = pm.multi();
    }

    #[test]
    fn test_progress_spinner_new() {
        let spinner = ProgressSpinner::new("test");
        spinner.finish_and_clear();
    }

    #[test]
    fn test_multi_step_progress() {
        let mut progress = MultiStepProgress::new(vec![
            "Step 1".to_string(),
            "Step 2".to_string(),
            "Step 3".to_string(),
        ]);
        progress.next_step();
        progress.next_step();
        progress.finish("Done");
    }
}
