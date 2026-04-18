//! User interface utilities
//!
//! This module provides:
//! - Consistent output formatting (UI)
//! - Re-exports from vx-console for unified progress management
//! - Tool suggestion display for friendly error messages

use crate::suggestions::{self, ToolSuggestion};
use colored::*;
use std::sync::atomic::{AtomicBool, Ordering};
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

// Re-export progress types from vx-console for unified progress management.
// All progress bars use the same style and global ProgressManager instance.
/// Alias kept for backward compatibility.
pub use vx_console::MultiStepProgress as MultiProgress;
pub use vx_console::{
    DownloadProgress, InstallProgress, ManagedDownload, ManagedSpinner, ManagedTask,
    MultiStepProgress, ProgressSpinner,
};
