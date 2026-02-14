//! # vx-console
//!
//! Unified console output system for vx.
//!
//! This crate provides a consistent API for console output, progress bars,
//! and terminal interactions across all vx components.
//!
//! ## Features
//!
//! - **Shell**: Cargo-style output abstraction with verbosity control
//! - **Progress**: Unified progress bars and spinners
//! - **Terminal**: Cross-platform terminal detection and adaptation
//! - **Interact**: Interactive input (confirm, password, select)
//! - **Theme**: Customizable output themes
//! - **Test**: Testing utilities for capturing output
//! - **Task**: Task execution with timing statistics
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use vx_console::{Shell, Verbosity, ColorChoice};
//!
//! // Create a shell
//! let mut shell = Shell::new();
//!
//! // Basic output
//! shell.status("Compiling", "vx v0.1.0").unwrap();
//! shell.info("Installing node...").unwrap();
//! shell.success("Installed node@20.10.0").unwrap();
//! shell.warn("Version 18 is deprecated").unwrap();
//! shell.error("Failed to download").unwrap();
//!
//! // Progress spinner
//! let spinner = shell.spinner("Downloading...");
//! // ... do work ...
//! spinner.finish_success("Downloaded");
//! ```
//!
//! ## Global Console
//!
//! ```rust,no_run
//! use vx_console::Console;
//!
//! // Get global console instance
//! let console = Console::global();
//! let guard = console.read().unwrap();
//!
//! guard.info("Hello, world!");
//! guard.success("Done!");
//! ```
//!
//! ## Task Execution with Timing
//!
//! ```rust,no_run
//! use vx_console::Console;
//!
//! let console = Console::new();
//!
//! // Execute a task with timing
//! let result = console.task("Building project", || {
//!     // ... do work ...
//!     Ok::<_, std::io::Error>(())
//! });
//! // Output: ✓ Building project (2.3s)
//! ```

mod format;
mod output;
mod shell;
mod style;
mod task;
mod term;
mod test_support;

#[cfg(feature = "progress")]
mod progress;

#[cfg(feature = "progress")]
mod interact;

// Re-exports
pub use format::{CiOutput, JsonOutput, OutputMode};
pub use output::{ColorChoice, ShellOut};
pub use shell::{Shell, ShellBuilder, Verbosity};
pub use style::{Color, Style, Theme, ThemeBuilder};
pub use task::{TaskResult, TimedTask, format_bytes, format_duration, format_speed};
pub use term::{CiEnvironment, Term, TermCapabilities, TerminalType};
pub use test_support::{TestOutput, TestWriter};

#[cfg(feature = "progress")]
pub use progress::{
    DownloadProgress, InstallProgress, ManagedDownload, ManagedSpinner, ManagedTask,
    MultiStepProgress, ProgressManager, ProgressSpinner,
};

#[cfg(feature = "progress")]
pub use interact::{confirm, input, multi_select, password, select};

use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};

/// Global console instance for convenient access.
static GLOBAL_CONSOLE: Lazy<Arc<RwLock<Console>>> =
    Lazy::new(|| Arc::new(RwLock::new(Console::new())));

/// Console provides a high-level API for console output.
///
/// It wraps a `Shell` and provides convenient methods for common output patterns.
#[derive(Debug)]
pub struct Console {
    shell: Shell,
    #[cfg(feature = "progress")]
    progress_manager: Arc<ProgressManager>,
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

impl Console {
    /// Create a new console with default settings.
    pub fn new() -> Self {
        Self {
            shell: Shell::new(),
            #[cfg(feature = "progress")]
            progress_manager: Arc::new(ProgressManager::new()),
        }
    }

    /// Get the global console instance.
    pub fn global() -> Arc<RwLock<Console>> {
        Arc::clone(&GLOBAL_CONSOLE)
    }

    /// Create a console builder for custom configuration.
    pub fn builder() -> ConsoleBuilder {
        ConsoleBuilder::new()
    }

    /// Get a reference to the underlying shell.
    pub fn shell(&self) -> &Shell {
        &self.shell
    }

    /// Get a mutable reference to the underlying shell.
    pub fn shell_mut(&mut self) -> &mut Shell {
        &mut self.shell
    }

    /// Set the verbosity level.
    pub fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.shell.set_verbosity(verbosity);
    }

    /// Set the color choice.
    pub fn set_color_choice(&mut self, color_choice: ColorChoice) {
        self.shell.set_color_choice(color_choice);
    }

    /// Print an info message.
    pub fn info(&self, message: &str) {
        let _ = self.shell.info(message);
    }

    /// Print a success message.
    pub fn success(&self, message: &str) {
        let _ = self.shell.success(message);
    }

    /// Print a warning message.
    pub fn warn(&self, message: &str) {
        let _ = self.shell.warn(message);
    }

    /// Print an error message.
    pub fn error(&self, message: &str) {
        let _ = self.shell.error(message);
    }

    /// Print a hint message.
    pub fn hint(&self, message: &str) {
        let _ = self.shell.hint(message);
    }

    /// Print a debug message (only in verbose mode).
    pub fn debug(&self, message: &str) {
        let _ = self.shell.debug(message);
    }

    /// Print a status message with a colored prefix.
    pub fn status(&self, status: &str, message: &str) {
        let _ = self.shell.status(status, message);
    }

    /// Create a spinner with the given message.
    #[cfg(feature = "progress")]
    pub fn spinner(&self, message: &str) -> ManagedSpinner {
        self.progress_manager.add_spinner(message)
    }

    /// Create a download progress bar.
    #[cfg(feature = "progress")]
    pub fn download_progress(&self, message: &str, total_size: u64) -> ManagedDownload {
        self.progress_manager.add_download(total_size, message)
    }

    /// Create a task progress bar.
    #[cfg(feature = "progress")]
    pub fn task_progress(&self, message: &str, total: u64) -> ManagedTask {
        self.progress_manager.add_task(total, message)
    }

    /// Get the progress manager.
    #[cfg(feature = "progress")]
    pub fn progress_manager(&self) -> &Arc<ProgressManager> {
        &self.progress_manager
    }

    /// Suspend progress bars and execute a function.
    #[cfg(feature = "progress")]
    pub fn suspend<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.progress_manager.suspend(f)
    }

    /// Execute a task with timing and progress display.
    ///
    /// Shows a spinner while the task is running, then displays the result
    /// with the elapsed time.
    ///
    /// # Example
    /// ```rust,no_run
    /// use vx_console::Console;
    ///
    /// let console = Console::new();
    /// let result = console.task("Building project", || {
    ///     // ... do work ...
    ///     Ok::<_, std::io::Error>(())
    /// });
    /// // Output: ✓ Building project (2.3s)
    /// ```
    #[cfg(feature = "progress")]
    pub fn task<F, T, E>(&self, message: &str, f: F) -> std::result::Result<TaskResult<T>, E>
    where
        F: FnOnce() -> std::result::Result<T, E>,
        E: std::fmt::Display,
    {
        use std::time::Instant;

        let spinner = self.spinner(message);
        let start = Instant::now();

        match f() {
            Ok(value) => {
                let duration = start.elapsed();
                spinner.finish_success(&format!(
                    "{} ({})",
                    message,
                    task::format_duration(duration)
                ));
                Ok(TaskResult::new(value, duration))
            }
            Err(e) => {
                spinner.finish_error(&format!("{}: {}", message, e));
                Err(e)
            }
        }
    }

    /// Execute an async task with timing and progress display.
    #[cfg(feature = "progress")]
    pub async fn task_async<F, Fut, T, E>(
        &self,
        message: &str,
        f: F,
    ) -> std::result::Result<TaskResult<T>, E>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = std::result::Result<T, E>>,
        E: std::fmt::Display,
    {
        use std::time::Instant;

        let spinner = self.spinner(message);
        let start = Instant::now();

        match f().await {
            Ok(value) => {
                let duration = start.elapsed();
                spinner.finish_success(&format!(
                    "{} ({})",
                    message,
                    task::format_duration(duration)
                ));
                Ok(TaskResult::new(value, duration))
            }
            Err(e) => {
                spinner.finish_error(&format!("{}: {}", message, e));
                Err(e)
            }
        }
    }

    /// Execute a multi-step task with progress display.
    #[cfg(feature = "progress")]
    pub fn steps<F, T, E>(&self, title: &str, f: F) -> std::result::Result<T, E>
    where
        F: FnOnce(&mut StepRunner) -> std::result::Result<T, E>,
    {
        let mut runner = StepRunner::new(title, &self.progress_manager);
        f(&mut runner)
    }
}

/// Helper for running multi-step tasks.
#[cfg(feature = "progress")]
pub struct StepRunner<'a> {
    #[allow(dead_code)]
    title: String,
    progress_manager: &'a ProgressManager,
    current_spinner: Option<ManagedSpinner>,
}

#[cfg(feature = "progress")]
impl<'a> StepRunner<'a> {
    fn new(title: &str, progress_manager: &'a ProgressManager) -> Self {
        Self {
            title: title.to_string(),
            progress_manager,
            current_spinner: None,
        }
    }

    /// Start a new step.
    pub fn step(&mut self, message: &str) -> Result<()> {
        // Finish previous step if any
        if let Some(spinner) = self.current_spinner.take() {
            spinner.finish_success(spinner.message());
        }

        // Start new step
        self.current_spinner = Some(self.progress_manager.add_spinner(message));
        Ok(())
    }

    /// Mark the current step as complete.
    pub fn complete(&mut self) {
        if let Some(spinner) = self.current_spinner.take() {
            spinner.finish_success(spinner.message());
        }
    }

    /// Mark the current step as failed.
    pub fn fail(&mut self, error: &str) {
        if let Some(spinner) = self.current_spinner.take() {
            spinner.finish_error(error);
        }
    }
}

/// Builder for creating a customized Console.
#[derive(Debug, Default)]
pub struct ConsoleBuilder {
    verbosity: Option<Verbosity>,
    color_choice: Option<ColorChoice>,
    output_mode: Option<OutputMode>,
    theme: Option<Theme>,
}

impl ConsoleBuilder {
    /// Create a new console builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the verbosity level.
    pub fn verbosity(mut self, verbosity: Verbosity) -> Self {
        self.verbosity = Some(verbosity);
        self
    }

    /// Set the color choice.
    pub fn color_choice(mut self, color_choice: ColorChoice) -> Self {
        self.color_choice = Some(color_choice);
        self
    }

    /// Set the output mode.
    pub fn output_mode(mut self, mode: OutputMode) -> Self {
        self.output_mode = Some(mode);
        self
    }

    /// Set the theme.
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Build the console.
    pub fn build(self) -> Console {
        let mut shell_builder = Shell::builder();

        if let Some(verbosity) = self.verbosity {
            shell_builder = shell_builder.verbosity(verbosity);
        }

        if let Some(color_choice) = self.color_choice {
            shell_builder = shell_builder.color_choice(color_choice);
        }

        if let Some(theme) = self.theme {
            shell_builder = shell_builder.theme(theme);
        }

        // RFC 0031: Wire output_mode through to Shell
        if let Some(output_mode) = self.output_mode {
            shell_builder = shell_builder.output_mode(output_mode);
        }

        Console {
            shell: shell_builder.build(),
            #[cfg(feature = "progress")]
            progress_manager: Arc::new(ProgressManager::new()),
        }
    }
}

/// Errors that can occur in vx-console.
#[derive(Debug, thiserror::Error)]
pub enum ConsoleError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Terminal error.
    #[error("Terminal error: {0}")]
    Terminal(String),

    /// User cancelled operation.
    #[error("Operation cancelled by user")]
    Cancelled,
}

/// Result type for vx-console operations.
pub type Result<T> = std::result::Result<T, ConsoleError>;
