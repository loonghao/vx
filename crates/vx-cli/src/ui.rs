//! User interface utilities
//!
//! This module provides:
//! - Consistent output formatting (UI)
//! - Modern progress indicators (ProgressSpinner, DownloadProgress, MultiProgress)
//! - Global progress manager (ProgressManager)
//! - Tool suggestion display for friendly error messages

use crate::suggestions::{self, ToolSuggestion};
use colored::*;
use indicatif::{MultiProgress as IndicatifMultiProgress, ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

static VERBOSE: AtomicBool = AtomicBool::new(false);

// Global progress manager instance
static PROGRESS_MANAGER: OnceLock<Arc<ProgressManager>> = OnceLock::new();

/// Get the global progress manager
pub fn progress_manager() -> Arc<ProgressManager> {
    PROGRESS_MANAGER
        .get_or_init(|| Arc::new(ProgressManager::new()))
        .clone()
}

/// Global progress manager for coordinating multiple progress indicators
pub struct ProgressManager {
    multi: IndicatifMultiProgress,
    active_bars: Mutex<Vec<ProgressBar>>,
}

impl ProgressManager {
    /// Create a new progress manager
    pub fn new() -> Self {
        Self {
            multi: IndicatifMultiProgress::new(),
            active_bars: Mutex::new(Vec::new()),
        }
    }

    /// Create a new spinner under this manager
    pub fn add_spinner(&self, message: &str) -> ManagedSpinner {
        let bar = self.multi.add(ProgressBar::new_spinner());
        bar.set_style(
            ProgressStyle::with_template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(80));

        if let Ok(mut bars) = self.active_bars.lock() {
            bars.push(bar.clone());
        }

        ManagedSpinner { bar }
    }

    /// Create a download progress bar under this manager
    pub fn add_download(&self, total_size: u64, message: &str) -> ManagedDownload {
        let bar = self.multi.add(ProgressBar::new(total_size));
        bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} {msg}\n  {wide_bar:.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
            )
            .unwrap()
            .progress_chars("━━╺"),
        );
        bar.set_message(message.to_string());

        if let Ok(mut bars) = self.active_bars.lock() {
            bars.push(bar.clone());
        }

        ManagedDownload { bar }
    }

    /// Create a task progress bar under this manager
    pub fn add_task(&self, total: u64, message: &str) -> ManagedTask {
        let bar = self.multi.add(ProgressBar::new(total));
        bar.set_style(
            ProgressStyle::with_template("{spinner:.blue} {msg} [{bar:30.cyan/blue}] {pos}/{len}")
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

    /// Clear all active progress bars
    pub fn clear_all(&self) {
        if let Ok(mut bars) = self.active_bars.lock() {
            for bar in bars.drain(..) {
                bar.finish_and_clear();
            }
        }
    }

    /// Suspend progress bars for clean output
    pub fn suspend<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.multi.suspend(f)
    }
}

impl Default for ProgressManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A spinner managed by ProgressManager
pub struct ManagedSpinner {
    bar: ProgressBar,
}

impl ManagedSpinner {
    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    pub fn finish_with_message(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }

    pub fn finish_and_clear(&self) {
        self.bar.finish_and_clear();
    }

    pub fn finish_success(&self, message: &str) {
        self.bar
            .finish_with_message(format!("{} {}", "✓".green(), message));
    }

    pub fn finish_error(&self, message: &str) {
        self.bar
            .finish_with_message(format!("{} {}", "✗".red(), message.red()));
    }
}

impl Drop for ManagedSpinner {
    fn drop(&mut self) {
        if !self.bar.is_finished() {
            self.bar.finish_and_clear();
        }
    }
}

/// A download progress bar managed by ProgressManager
pub struct ManagedDownload {
    bar: ProgressBar,
}

impl ManagedDownload {
    pub fn set_length(&self, len: u64) {
        self.bar.set_length(len);
    }

    pub fn set_position(&self, pos: u64) {
        self.bar.set_position(pos);
    }

    pub fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }

    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    pub fn finish_with_message(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }

    pub fn finish_and_clear(&self) {
        self.bar.finish_and_clear();
    }
}

impl Drop for ManagedDownload {
    fn drop(&mut self) {
        if !self.bar.is_finished() {
            self.bar.finish_and_clear();
        }
    }
}

/// A task progress bar managed by ProgressManager
pub struct ManagedTask {
    bar: ProgressBar,
}

impl ManagedTask {
    pub fn set_position(&self, pos: u64) {
        self.bar.set_position(pos);
    }

    pub fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }

    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    pub fn finish_with_message(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }

    pub fn finish_and_clear(&self) {
        self.bar.finish_and_clear();
    }
}

impl Drop for ManagedTask {
    fn drop(&mut self) {
        if !self.bar.is_finished() {
            self.bar.finish_and_clear();
        }
    }
}

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

    /// Format a header string (returns colored string)
    pub fn format_header(message: &str) -> String {
        message.bold().underline().to_string()
    }

    /// Format a success string (returns colored string)
    pub fn format_success(message: &str) -> String {
        message.green().to_string()
    }

    /// Format a warning string (returns colored string)
    pub fn format_warn(message: &str) -> String {
        message.yellow().to_string()
    }

    /// Format an error string (returns colored string)
    pub fn format_error(message: &str) -> String {
        message.red().to_string()
    }

    /// Create a new spinner (returns a simple message for now)
    pub fn new_spinner(message: &str) -> SimpleSpinner {
        Self::spinner(message);
        SimpleSpinner
    }

    /// Display a friendly "tool not found" error with suggestions
    ///
    /// This function:
    /// 1. Shows the error message
    /// 2. Checks for known aliases (e.g., "rust" -> "cargo")
    /// 3. Suggests similar tool names using fuzzy matching
    /// 4. Provides a link to request new tool support
    pub fn tool_not_found(tool_name: &str, available_tools: &[String]) {
        // Use eprintln for all output to ensure consistent ordering
        eprintln!(
            "{} {}",
            "✗".red(),
            format!("Tool '{}' is not supported by vx", tool_name).red()
        );

        // Get suggestions
        let suggestions_list = suggestions::get_tool_suggestions(tool_name, available_tools);

        if !suggestions_list.is_empty() {
            eprintln!();
            for suggestion in &suggestions_list {
                if suggestion.is_alias {
                    // Alias match - more confident suggestion
                    eprintln!(
                        "{} Did you mean: {} ({})",
                        "💡".cyan(),
                        suggestion.suggested_tool.cyan().bold(),
                        suggestion.description.dimmed()
                    );
                } else {
                    // Fuzzy match
                    eprintln!(
                        "{} Did you mean: {}",
                        "💡".cyan(),
                        suggestion.suggested_tool.cyan().bold()
                    );
                }
            }
        }

        // Show available tools hint
        eprintln!();
        eprintln!(
            "{} {}",
            "💡".cyan(),
            "Use 'vx list' to see all supported tools".dimmed()
        );

        // Show feature request link
        let issue_url = suggestions::get_feature_request_url(tool_name);
        eprintln!(
            "{} Request support for '{}': {}",
            "💡".cyan(),
            tool_name,
            issue_url.dimmed()
        );
    }

    /// Display a friendly "tool not found" error with suggestions (simpler version)
    pub fn tool_not_found_simple(tool_name: &str, suggestion: Option<&ToolSuggestion>) {
        eprintln!(
            "{} {}",
            "✗".red(),
            format!("Tool '{}' is not supported", tool_name).red()
        );

        if let Some(s) = suggestion {
            eprintln!();
            if s.is_alias {
                eprintln!(
                    "{} Did you mean: {} ({})",
                    "💡".cyan(),
                    s.suggested_tool.cyan().bold(),
                    s.description.dimmed()
                );
            } else {
                eprintln!(
                    "{} Did you mean: {}",
                    "💡".cyan(),
                    s.suggested_tool.cyan().bold()
                );
            }
        }

        eprintln!(
            "{} {}",
            "💡".cyan(),
            "Use 'vx list' to see all supported tools".dimmed()
        );
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

    /// Create a spinner for download operations with modern style
    pub fn new_download(message: &str) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::with_template("{spinner:.green} {msg}")
                .unwrap()
                .tick_strings(&["◜", "◠", "◝", "◞", "◡", "◟"]),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(100));
        Self { bar }
    }

    /// Create a spinner for installation operations with modern style
    pub fn new_install(message: &str) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::with_template("{spinner:.blue} {msg}")
                .unwrap()
                .tick_strings(&["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"]),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(80));
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
    /// Create a new download progress bar with modern style
    pub fn new(total_size: u64, message: &str) -> Self {
        let bar = ProgressBar::new(total_size);
        bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} {msg}\n  {wide_bar:.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
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

/// Multi-step progress indicator with modern style
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

/// Installation progress tracker for multi-tool installations
pub struct InstallProgress {
    multi: IndicatifMultiProgress,
    main_bar: ProgressBar,
    current_bar: Option<ProgressBar>,
}

impl InstallProgress {
    /// Create a new installation progress tracker
    pub fn new(total_tools: usize, title: &str) -> Self {
        let multi = IndicatifMultiProgress::new();

        // Main progress bar showing overall progress
        let main_bar = multi.add(ProgressBar::new(total_tools as u64));
        main_bar.set_style(
            ProgressStyle::with_template("{msg}\n  {wide_bar:.green/dim} {pos}/{len} tools")
                .unwrap()
                .progress_chars("━━╺"),
        );
        main_bar.set_message(title.to_string());

        Self {
            multi,
            main_bar,
            current_bar: None,
        }
    }

    /// Start installing a tool
    pub fn start_tool(&mut self, tool_name: &str, version: &str) {
        // Clear previous tool bar if any
        if let Some(bar) = self.current_bar.take() {
            bar.finish_and_clear();
        }

        // Create new spinner for current tool
        let bar = self.multi.add(ProgressBar::new_spinner());
        bar.set_style(
            ProgressStyle::with_template("  {spinner:.blue} Installing {msg}")
                .unwrap()
                .tick_strings(&["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"]),
        );
        bar.set_message(format!("{}@{}", tool_name, version));
        bar.enable_steady_tick(Duration::from_millis(80));
        self.current_bar = Some(bar);
    }

    /// Mark current tool as completed
    pub fn complete_tool(&mut self, success: bool, tool_name: &str, version: &str) {
        if let Some(bar) = self.current_bar.take() {
            if success {
                bar.finish_with_message(format!("{} {}@{}", "✓".green(), tool_name, version));
            } else {
                bar.finish_with_message(format!("{} {}@{}", "✗".red(), tool_name, version));
            }
        }
        self.main_bar.inc(1);
    }

    /// Finish all installations
    pub fn finish(&self, message: &str) {
        self.main_bar.finish_with_message(message.to_string());
    }
}

impl Drop for InstallProgress {
    fn drop(&mut self) {
        if let Some(bar) = self.current_bar.take() {
            if !bar.is_finished() {
                bar.finish_and_clear();
            }
        }
        if !self.main_bar.is_finished() {
            self.main_bar.finish_and_clear();
        }
    }
}
