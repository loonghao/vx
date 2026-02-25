//! User interface utilities
//!
//! This module provides:
//! - Consistent output formatting (UI)
//! - Modern progress indicators (ProgressSpinner, DownloadProgress, MultiProgress)
//! - Re-exports from vx-console for unified progress management
//! - Tool suggestion display for friendly error messages

use crate::suggestions::{self, ToolSuggestion};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use vx_console::global_progress_manager;

static VERBOSE: AtomicBool = AtomicBool::new(false);

/// Get the global progress manager (re-export from vx-console)
///
/// This ensures all progress bars and messages use the same MultiProgress instance.
pub fn progress_manager() -> std::sync::Arc<vx_console::ProgressManager> {
    global_progress_manager()
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
        global_progress_manager().println(&format!("{} {}", "ℹ".blue(), message));
    }

    /// Print a success message
    pub fn success(message: &str) {
        global_progress_manager().println(&format!("{} {}", "✓".green(), message));
    }

    /// Print a warning message
    pub fn warn(message: &str) {
        global_progress_manager().println(&format!("{} {}", "⚠".yellow(), message.yellow()));
    }

    /// Print an error message
    pub fn error(message: &str) {
        // errors go to stderr; use eprintln inside suspend to avoid glitches
        global_progress_manager().suspend(|| {
            eprintln!("{} {}", "✗".red(), message.red());
        });
    }

    /// Print a debug message (only in verbose mode)
    pub fn debug(message: &str) {
        if Self::is_verbose() {
            global_progress_manager().println(&format!("{} {}", "→".purple(), message.dimmed()));
        }
    }

    /// Print a hint message
    pub fn hint(message: &str) {
        global_progress_manager().println(&format!("{} {}", "💡".cyan(), message.dimmed()));
    }

    /// Print a list item
    pub fn item(message: &str) {
        global_progress_manager().println(&format!("  {}", message));
    }

    /// Print a detail line (indented)
    pub fn detail(message: &str) {
        global_progress_manager().println(&format!("    {}", message.dimmed()));
    }

    /// Print a separator line
    pub fn separator() {
        global_progress_manager().println(&"─".repeat(50).dimmed().to_string());
    }

    /// Print a header
    pub fn header(message: &str) {
        global_progress_manager().println(&format!("\n{}", message.bold().underline()));
    }

    /// Print a section header (for multi-step operations)
    pub fn section(message: &str) {
        global_progress_manager().println(&format!("\n{} {}", "▸".cyan().bold(), message.bold()));
    }

    /// Print a progress message
    pub fn progress(message: &str) {
        global_progress_manager().println(&format!("{} {}...", "⏳".yellow(), message));
    }

    /// Complete a progress message
    pub fn progress_done() {
        global_progress_manager().println(&format!(" {}", "Done!".green()));
    }

    /// Print a spinner (placeholder for now)
    pub fn spinner(message: &str) {
        global_progress_manager().println(&format!("{} {}", "⏳".yellow(), message));
    }

    /// Print a step message
    pub fn step(message: &str) {
        global_progress_manager().println(&format!("{} {}", "▶".blue(), message));
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
        // Use global progress manager to avoid interleaving with progress bars
        let pm = global_progress_manager();
        pm.suspend(|| {
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
                        eprintln!(
                            "{} Did you mean: {} ({})",
                            "💡".cyan(),
                            suggestion.suggested_tool.cyan().bold(),
                            suggestion.description.dimmed()
                        );
                    } else {
                        eprintln!(
                            "{} Did you mean: {}",
                            "💡".cyan(),
                            suggestion.suggested_tool.cyan().bold()
                        );
                    }
                }
            }

            eprintln!();
            eprintln!(
                "{} {}",
                "💡".cyan(),
                "Use 'vx list' to see all supported tools".dimmed()
            );

            let issue_url = suggestions::get_feature_request_url(tool_name);
            eprintln!(
                "{} Request support for '{}': {}",
                "💡".cyan(),
                tool_name,
                issue_url.dimmed()
            );
        });
    }

    /// Display a friendly "tool not found" error with suggestions (simpler version)
    pub fn tool_not_found_simple(tool_name: &str, suggestion: Option<&ToolSuggestion>) {
        global_progress_manager().suspend(|| {
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
        });
    }
}

/// Simple spinner placeholder
pub struct SimpleSpinner;

impl SimpleSpinner {
    pub fn finish_and_clear(&self) {
        // For now, just print a completion message
        global_progress_manager().println(&format!(" {}", "Done!".green()));
    }
}

/// A beautiful progress spinner using indicatif (managed by global ProgressManager)
pub struct ProgressSpinner {
    bar: ProgressBar,
}

impl ProgressSpinner {
    /// Create a new progress spinner with a message
    pub fn new(message: &str) -> Self {
        let pm = global_progress_manager();
        let bar = pm.multi().add(ProgressBar::new_spinner());
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
        let pm = global_progress_manager();
        let bar = pm.multi().add(ProgressBar::new_spinner());
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
        let pm = global_progress_manager();
        let bar = pm.multi().add(ProgressBar::new_spinner());
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

/// A progress bar for download operations with size tracking (managed by global ProgressManager)
pub struct DownloadProgress {
    bar: ProgressBar,
}

impl DownloadProgress {
    /// Create a new download progress bar with modern style
    pub fn new(total_size: u64, message: &str) -> Self {
        let pm = global_progress_manager();
        let bar = pm.multi().add(ProgressBar::new(total_size));
        bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} {msg} {wide_bar:.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
            )
            .unwrap()
            .progress_chars("━━╺"),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(100));
        Self { bar }
    }

    /// Create a download progress bar with unknown size
    pub fn new_unknown(message: &str) -> Self {
        let pm = global_progress_manager();
        let bar = pm.multi().add(ProgressBar::new_spinner());
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

/// Multi-step progress indicator with modern style (managed by global ProgressManager)
pub struct MultiProgress {
    steps: Vec<String>,
    current: usize,
    bar: ProgressBar,
}

impl MultiProgress {
    /// Create a new multi-step progress indicator
    pub fn new(steps: Vec<String>) -> Self {
        let total = steps.len() as u64;
        let pm = global_progress_manager();
        let bar = pm.multi().add(ProgressBar::new(total));
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

/// Installation progress tracker for multi-tool installations (managed by global ProgressManager)
pub struct InstallProgress {
    main_bar: ProgressBar,
    current_bar: Option<ProgressBar>,
}

impl InstallProgress {
    /// Create a new installation progress tracker
    pub fn new(total_tools: usize, title: &str) -> Self {
        let pm = global_progress_manager();

        // Main progress bar showing overall progress
        let main_bar = pm.multi().add(ProgressBar::new(total_tools as u64));
        main_bar.set_style(
            ProgressStyle::with_template("{msg} [{bar:40.green/dim}] {pos}/{len} tools")
                .unwrap()
                .progress_chars("━━╺"),
        );
        main_bar.set_message(title.to_string());
        main_bar.enable_steady_tick(Duration::from_millis(100));

        Self {
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
        let pm = global_progress_manager();
        let bar = pm.multi().add(ProgressBar::new_spinner());
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
        if let Some(bar) = self.current_bar.take()
            && !bar.is_finished()
        {
            bar.finish_and_clear();
        }
        if !self.main_bar.is_finished() {
            self.main_bar.finish_and_clear();
        }
    }
}

// Re-export ManagedSpinner and ManagedDownload from vx-console for convenience
pub use vx_console::{ManagedDownload, ManagedSpinner, ManagedTask};
