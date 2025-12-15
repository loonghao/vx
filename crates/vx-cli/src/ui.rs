//! User interface utilities

use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

static VERBOSE: AtomicBool = AtomicBool::new(false);

/// UI utilities for consistent output formatting
pub struct UI;

impl UI {
    /// Set verbose mode
    pub fn set_verbose(verbose: bool) {
        VERBOSE.store(verbose, Ordering::Relaxed);
    }

    /// Check if verbose mode is enabled
    pub fn is_verbose() -> bool {
        VERBOSE.load(Ordering::Relaxed)
    }

    /// Print an info message
    pub fn info(message: &str) {
        println!("{} {}", "ℹ".blue(), message);
    }

    /// Print a success message
    pub fn success(message: &str) {
        println!("{} {}", "✓".green(), message);
    }

    /// Print a warning message
    pub fn warn(message: &str) {
        println!("{} {}", "⚠".yellow(), message.yellow());
    }

    /// Print an error message
    pub fn error(message: &str) {
        eprintln!("{} {}", "✗".red(), message.red());
    }

    /// Print a debug message (only in verbose mode)
    pub fn debug(message: &str) {
        if Self::is_verbose() {
            println!("{} {}", "→".purple(), message.dimmed());
        }
    }

    /// Print a hint message
    pub fn hint(message: &str) {
        println!("{} {}", "💡".cyan(), message.dimmed());
    }

    /// Print a list item
    pub fn item(message: &str) {
        println!("  {}", message);
    }

    /// Print a detail line (indented)
    pub fn detail(message: &str) {
        println!("    {}", message.dimmed());
    }

    /// Print a separator line
    pub fn separator() {
        println!("{}", "─".repeat(50).dimmed());
    }

    /// Print a header
    pub fn header(message: &str) {
        println!("\n{}", message.bold().underline());
    }

    /// Print a progress message
    pub fn progress(message: &str) {
        print!("{} {}...", "⏳".yellow(), message);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }

    /// Complete a progress message
    pub fn progress_done() {
        println!(" {}", "Done!".green());
    }

    /// Print a spinner (placeholder for now)
    pub fn spinner(message: &str) {
        println!("{} {}", "⏳".yellow(), message);
    }

    /// Print a step message
    pub fn step(message: &str) {
        println!("{} {}", "▶".blue(), message);
    }

    /// Alias for warn method (for backward compatibility)
    pub fn warning(message: &str) {
        Self::warn(message);
    }

    /// Create a new spinner (returns a simple message for now)
    pub fn new_spinner(message: &str) -> SimpleSpinner {
        Self::spinner(message);
        SimpleSpinner
    }
}

/// Simple spinner placeholder
pub struct SimpleSpinner;

impl SimpleSpinner {
    pub fn finish_and_clear(&self) {
        // For now, just print a completion message
        println!(" {}", "Done!".green());
    }
}

/// A beautiful progress spinner using indicatif
pub struct ProgressSpinner {
    bar: ProgressBar,
}

impl ProgressSpinner {
    /// Create a new progress spinner with a message
    pub fn new(message: &str) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::with_template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(80));
        Self { bar }
    }

    /// Create a spinner for download operations
    pub fn new_download(message: &str) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::with_template("{spinner:.green} {msg}")
                .unwrap()
                .tick_strings(&["▹▹▹▹▹", "▸▹▹▹▹", "▹▸▹▹▹", "▹▹▸▹▹", "▹▹▹▸▹", "▹▹▹▹▸"]),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(120));
        Self { bar }
    }

    /// Create a spinner for installation operations
    pub fn new_install(message: &str) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::with_template("{spinner:.blue} {msg}")
                .unwrap()
                .tick_strings(&["◐", "◓", "◑", "◒"]),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(100));
        Self { bar }
    }

    /// Update the spinner message
    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    /// Finish the spinner with a success message
    pub fn finish_with_message(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }

    /// Finish the spinner and clear it
    pub fn finish_and_clear(&self) {
        self.bar.finish_and_clear();
    }

    /// Finish the spinner with an error
    pub fn finish_with_error(&self, message: &str) {
        self.bar
            .finish_with_message(format!("{} {}", "✗".red(), message.red()));
    }
}

impl Drop for ProgressSpinner {
    fn drop(&mut self) {
        if !self.bar.is_finished() {
            self.bar.finish_and_clear();
        }
    }
}

/// A progress bar for download operations with size tracking
pub struct DownloadProgress {
    bar: ProgressBar,
}

impl DownloadProgress {
    /// Create a new download progress bar
    pub fn new(total_size: u64, message: &str) -> Self {
        let bar = ProgressBar::new(total_size);
        bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} {msg}\n{wide_bar:.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
            )
            .unwrap()
            .progress_chars("━━╺"),
        );
        bar.set_message(message.to_string());
        Self { bar }
    }

    /// Create a download progress bar with unknown size
    pub fn new_unknown(message: &str) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::with_template("{spinner:.green} {msg} {bytes} ({bytes_per_sec})")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(80));
        Self { bar }
    }

    /// Set the total size (useful when size becomes known)
    pub fn set_length(&self, len: u64) {
        self.bar.set_length(len);
    }

    /// Update progress
    pub fn set_position(&self, pos: u64) {
        self.bar.set_position(pos);
    }

    /// Increment progress
    pub fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }

    /// Update message
    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    /// Finish with success
    pub fn finish_with_message(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }

    /// Finish and clear
    pub fn finish_and_clear(&self) {
        self.bar.finish_and_clear();
    }
}

impl Drop for DownloadProgress {
    fn drop(&mut self) {
        if !self.bar.is_finished() {
            self.bar.finish_and_clear();
        }
    }
}

/// Multi-step progress indicator
pub struct MultiProgress {
    steps: Vec<String>,
    current: usize,
    bar: ProgressBar,
}

impl MultiProgress {
    /// Create a new multi-step progress indicator
    pub fn new(steps: Vec<String>) -> Self {
        let total = steps.len() as u64;
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::with_template("{spinner:.cyan} [{pos}/{len}] {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
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

    /// Move to the next step
    pub fn next_step(&mut self) {
        self.current += 1;
        self.bar.inc(1);
        if let Some(step) = self.steps.get(self.current) {
            self.bar.set_message(step.clone());
        }
    }

    /// Finish all steps
    pub fn finish(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }
}

impl Drop for MultiProgress {
    fn drop(&mut self) {
        if !self.bar.is_finished() {
            self.bar.finish_and_clear();
        }
    }
}
