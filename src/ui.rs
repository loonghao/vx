use colored::Colorize;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use indicatif::{ProgressBar, ProgressFinish, ProgressState, ProgressStyle};
use std::fmt::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Global verbose flag
static VERBOSE: AtomicBool = AtomicBool::new(false);

/// UI utilities for better CLI experience
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
    /// Create a new progress bar for downloads
    pub fn new_progress_bar(len: u64) -> ProgressBar {
        ProgressBar::new(len)
            .with_style(
                ProgressStyle::with_template(
                    "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})"
                )
                .unwrap()
                .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
                    write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
                })
                .progress_chars("#>-")
            )
            .with_finish(ProgressFinish::WithMessage("[SUCCESS] Download complete".into()))
    }

    /// Create a spinner for indeterminate operations
    pub fn new_spinner(message: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }

    /// Print success message
    pub fn success(message: &str) {
        println!("{} {}", "[SUCCESS]".green().bold(), message);
    }

    /// Print error message
    pub fn error(message: &str) {
        eprintln!("{} {}", "[ERROR]".red().bold(), message);
    }

    /// Print warning message
    pub fn warning(message: &str) {
        println!("{} {}", "[WARNING]".yellow().bold(), message);
    }

    /// Print info message (only in verbose mode)
    pub fn info(message: &str) {
        if Self::is_verbose() {
            println!("{} {}", "[INFO]".blue().bold(), message);
        }
    }

    /// Print step message (only in verbose mode)
    pub fn step(message: &str) {
        if Self::is_verbose() {
            println!("{} {}", "[STEP]".cyan().bold(), message);
        }
    }

    /// Print hint message
    pub fn hint(message: &str) {
        println!("{} {}", "[HINT]".bright_yellow().bold(), message.dimmed());
    }

    /// Ask for confirmation
    pub fn confirm(message: &str, default: bool) -> anyhow::Result<bool> {
        let result = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(message)
            .default(default)
            .interact()?;
        Ok(result)
    }

    /// Select from options
    pub fn select(message: &str, options: &[&str]) -> anyhow::Result<usize> {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(message)
            .items(options)
            .default(0)
            .interact()?;
        Ok(selection)
    }

    /// Clear current line
    pub fn clear_line() {
        let term = Term::stdout();
        let _ = term.clear_line();
    }

    /// Print header with separator
    pub fn header(title: &str) {
        println!();
        println!("{}", title.bright_cyan().bold());
        println!("{}", "─".repeat(title.len()).bright_black());
    }

    /// Print section separator
    pub fn separator() {
        println!("{}", "─".repeat(50).bright_black());
    }

    /// Format file size
    pub fn format_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    /// Format duration
    pub fn format_duration(duration: Duration) -> String {
        let secs = duration.as_secs();
        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else {
            format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
        }
    }

    /// Show tool status with colors
    pub fn show_tool_status(tool: &str, version: Option<&str>, is_active: bool) {
        let status_icon = if is_active { "[ACTIVE]" } else { "[INACTIVE]" };
        let tool_name = if is_active {
            tool.bright_green().bold()
        } else {
            tool.normal()
        };

        match version {
            Some(v) => println!("  {} {} {}", status_icon.green(), tool_name, v.dimmed()),
            None => println!("  {} {}", status_icon.dimmed(), tool_name),
        }
    }

    /// Show package list with formatting
    pub fn show_package_list(packages: &[(String, String, bool)]) {
        if packages.is_empty() {
            Self::info("No packages found");
            return;
        }

        Self::header("Installed Packages");
        for (name, version, is_active) in packages {
            Self::show_tool_status(name, Some(version), *is_active);
        }
    }

    /// Show stats in a formatted way
    pub fn show_stats(
        total_packages: usize,
        total_versions: usize,
        total_size: u64,
        last_updated: &str,
    ) {
        Self::header("Package Statistics");
        println!(
            "  Total packages: {}",
            total_packages.to_string().bright_cyan()
        );
        println!(
            "  Total versions: {}",
            total_versions.to_string().bright_cyan()
        );
        println!(
            "  Total size: {}",
            Self::format_size(total_size).bright_cyan()
        );
        println!("  Last updated: {}", last_updated.bright_cyan());
    }

    /// Show available updates
    pub fn show_updates(updates: &[(String, String, String)]) {
        if updates.is_empty() {
            Self::success("All packages are up to date");
            return;
        }

        Self::header("Available Updates");
        for (tool, current, latest) in updates {
            println!(
                "  * {} {} -> {}",
                tool.bright_white(),
                current.red(),
                latest.green()
            );
        }
    }

    /// Show command execution
    pub fn show_command_execution(tool: &str, args: &[String]) {
        let command = format!("{} {}", tool, args.join(" "));
        Self::step(&format!("Running: {}", command.bright_white()));
    }
}
