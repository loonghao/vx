//! User interface utilities

use colored::*;
use std::sync::atomic::{AtomicBool, Ordering};

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
        println!("{} {}", "✅".green(), message);
    }

    /// Print a warning message
    pub fn warn(message: &str) {
        println!("{} {}", "⚠".yellow(), message.yellow());
    }

    /// Print an error message
    pub fn error(message: &str) {
        eprintln!("{} {}", "❌".red(), message.red());
    }

    /// Print a debug message (only in verbose mode)
    pub fn debug(message: &str) {
        if Self::is_verbose() {
            println!("{} {}", "🐛".purple(), message.dimmed());
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
