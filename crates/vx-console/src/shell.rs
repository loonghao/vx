//! Shell - Cargo-style output abstraction.
//!
//! The Shell struct encapsulates stdout/stderr and provides methods for
//! printing messages with consistent styling.

use crate::format::OutputMode;
use crate::output::{ColorChoice, ShellOut};
use crate::style::{Color, Style, Theme};
use crate::term::Term;
use crate::Result;

#[cfg(feature = "progress")]
use crate::progress::{ManagedSpinner, ProgressManager};

use std::fmt::Display;
use std::io::Write;
use std::sync::Arc;

/// Verbosity level for shell output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Verbosity {
    /// Only show errors.
    Quiet,
    /// Normal output level.
    #[default]
    Normal,
    /// Show additional debug information.
    Verbose,
}

/// Shell provides methods for printing to the terminal.
///
/// This is inspired by Cargo's Shell implementation.
#[derive(Debug)]
pub struct Shell {
    output: ShellOut,
    verbosity: Verbosity,
    theme: Theme,
    needs_clear: bool,
    /// Output mode (RFC 0031): controls JSON/CI/Standard behavior
    output_mode: OutputMode,
    #[cfg(feature = "progress")]
    progress_manager: Option<Arc<ProgressManager>>,
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

impl Shell {
    /// Create a new shell with default settings.
    pub fn new() -> Self {
        Self {
            output: ShellOut::stream(),
            verbosity: Verbosity::Normal,
            theme: Theme::default(),
            needs_clear: false,
            output_mode: OutputMode::Standard,
            #[cfg(feature = "progress")]
            progress_manager: None,
        }
    }

    /// Create a shell builder for custom configuration.
    pub fn builder() -> ShellBuilder {
        ShellBuilder::new()
    }

    /// Create a shell that writes to a buffer (for testing).
    pub fn from_write(out: Box<dyn Write + Send>) -> Self {
        Self {
            output: ShellOut::Write(std::sync::Mutex::new(out)),
            verbosity: Verbosity::Normal,
            theme: Theme::minimal(),
            needs_clear: false,
            output_mode: OutputMode::Standard,
            #[cfg(feature = "progress")]
            progress_manager: None,
        }
    }

    /// Get the current verbosity level.
    pub fn verbosity(&self) -> Verbosity {
        self.verbosity
    }

    /// Set the verbosity level.
    pub fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.verbosity = verbosity;
    }

    /// Get the current color choice.
    pub fn color_choice(&self) -> ColorChoice {
        self.output.color_choice()
    }

    /// Set the color choice.
    pub fn set_color_choice(&mut self, color_choice: ColorChoice) {
        self.output.set_color_choice(color_choice);
    }

    /// Check if the shell supports color output.
    pub fn supports_color(&self) -> bool {
        self.output.supports_color()
    }

    /// Get the current output mode (RFC 0031).
    pub fn output_mode(&self) -> OutputMode {
        self.output_mode
    }

    /// Set the output mode (RFC 0031).
    pub fn set_output_mode(&mut self, mode: OutputMode) {
        self.output_mode = mode;
    }

    /// Check if JSON output mode is active.
    pub fn is_json_mode(&self) -> bool {
        self.output_mode == OutputMode::Json
    }

    /// Check if stderr is a TTY.
    pub fn is_tty(&self) -> bool {
        self.output.is_tty()
    }

    /// Get the terminal width.
    pub fn term_width(&self) -> Option<usize> {
        Term::detect().size().map(|(w, _)| w as usize)
    }

    /// Get the current theme.
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Set the theme.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// Print a status message with a colored prefix.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vx_console::Shell;
    /// let shell = Shell::new();
    /// shell.status("Compiling", "vx v0.1.0").unwrap();
    /// // Output: "   Compiling vx v0.1.0"
    /// ```
    pub fn status(&self, status: impl Display, message: impl Display) -> Result<()> {
        self.status_with_color(status, message, Color::Green)
    }

    /// Print a status message with a custom color.
    pub fn status_with_color(
        &self,
        status: impl Display,
        message: impl Display,
        color: Color,
    ) -> Result<()> {
        if self.verbosity == Verbosity::Quiet {
            return Ok(());
        }

        let status_str = format!("{:>12}", status);
        let styled_status = self.style_text(&status_str, Style::new().fg(color).bold());

        self.output
            .write_stderr(&format!("{} {}\n", styled_status, message))?;
        Ok(())
    }

    /// Print an info message.
    pub fn info(&self, message: impl Display) -> Result<()> {
        if self.verbosity == Verbosity::Quiet {
            return Ok(());
        }

        let prefix = self.style_text(self.theme.info_prefix(), self.theme.info_style());
        self.output
            .write_stderr(&format!("{} {}\n", prefix, message))?;
        Ok(())
    }

    /// Print a success message.
    pub fn success(&self, message: impl Display) -> Result<()> {
        if self.verbosity == Verbosity::Quiet {
            return Ok(());
        }

        let prefix = self.style_text(self.theme.success_prefix(), self.theme.success_style());
        self.output
            .write_stderr(&format!("{} {}\n", prefix, message))?;
        Ok(())
    }

    /// Print a warning message.
    pub fn warn(&self, message: impl Display) -> Result<()> {
        let prefix = self.style_text(self.theme.warn_prefix(), self.theme.warn_style());
        self.output
            .write_stderr(&format!("{} {}\n", prefix, message))?;
        Ok(())
    }

    /// Print an error message.
    pub fn error(&self, message: impl Display) -> Result<()> {
        let prefix = self.style_text(self.theme.error_prefix(), self.theme.error_style());
        self.output
            .write_stderr(&format!("{} {}\n", prefix, message))?;
        Ok(())
    }

    /// Print a hint message.
    pub fn hint(&self, message: impl Display) -> Result<()> {
        if self.verbosity == Verbosity::Quiet {
            return Ok(());
        }

        let prefix = self.style_text(self.theme.hint_prefix(), self.theme.hint_style());
        self.output
            .write_stderr(&format!("{} {}\n", prefix, message))?;
        Ok(())
    }

    /// Print a debug message (only in verbose mode).
    pub fn debug(&self, message: impl Display) -> Result<()> {
        if self.verbosity != Verbosity::Verbose {
            return Ok(());
        }

        let prefix = self.style_text(self.theme.debug_prefix(), self.theme.debug_style());
        self.output
            .write_stderr(&format!("{} {}\n", prefix, message))?;
        Ok(())
    }

    /// Print a step message.
    pub fn step(&self, message: impl Display) -> Result<()> {
        if self.verbosity == Verbosity::Quiet {
            return Ok(());
        }

        let prefix = self.style_text("▶", Style::new().fg(Color::Blue));
        self.output
            .write_stderr(&format!("{} {}\n", prefix, message))?;
        Ok(())
    }

    /// Print an indented item.
    pub fn item(&self, message: impl Display) -> Result<()> {
        if self.verbosity == Verbosity::Quiet {
            return Ok(());
        }

        self.output.write_stderr(&format!("  {}\n", message))?;
        Ok(())
    }

    /// Print a detail line (more indented, dimmed).
    pub fn detail(&self, message: impl Display) -> Result<()> {
        if self.verbosity == Verbosity::Quiet {
            return Ok(());
        }

        let styled = self.style_text(&message.to_string(), Style::new().dimmed());
        self.output.write_stderr(&format!("    {}\n", styled))?;
        Ok(())
    }

    /// Print a header.
    pub fn header(&self, message: impl Display) -> Result<()> {
        if self.verbosity == Verbosity::Quiet {
            return Ok(());
        }

        let styled = self.style_text(&message.to_string(), Style::new().bold().underline());
        self.output.write_stderr(&format!("{}\n", styled))?;
        Ok(())
    }

    /// Print a separator line.
    pub fn separator(&self) -> Result<()> {
        if self.verbosity == Verbosity::Quiet {
            return Ok(());
        }

        let width = self.term_width().unwrap_or(80);
        let line = "─".repeat(width.min(80));
        let styled = self.style_text(&line, Style::new().dimmed());
        self.output.write_stderr(&format!("{}\n", styled))?;
        Ok(())
    }

    /// Print a newline.
    pub fn newline(&self) -> Result<()> {
        self.output.write_stderr("\n")?;
        Ok(())
    }

    /// Execute a closure only in verbose mode.
    pub fn verbose<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&Shell) -> Result<()>,
    {
        if self.verbosity == Verbosity::Verbose {
            f(self)?;
        }
        Ok(())
    }

    /// Create a spinner with the given message.
    #[cfg(feature = "progress")]
    pub fn spinner(&self, message: &str) -> ManagedSpinner {
        if let Some(ref pm) = self.progress_manager {
            pm.add_spinner(message)
        } else {
            ProgressManager::new().add_spinner(message)
        }
    }

    /// Set the progress manager.
    #[cfg(feature = "progress")]
    pub fn set_progress_manager(&mut self, pm: Arc<ProgressManager>) {
        self.progress_manager = Some(pm);
    }

    /// Style text according to the current color choice.
    fn style_text(&self, text: &str, style: Style) -> String {
        if self.supports_color() {
            style.apply(text)
        } else {
            text.to_string()
        }
    }

    /// Mark that the shell needs to clear the current line.
    pub fn set_needs_clear(&mut self, needs_clear: bool) {
        self.needs_clear = needs_clear;
    }

    /// Check if the shell needs to clear the current line.
    pub fn needs_clear(&self) -> bool {
        self.needs_clear
    }

    /// Clear the current line if needed.
    pub fn maybe_clear(&mut self) -> Result<()> {
        if self.needs_clear {
            self.clear_line()?;
            self.needs_clear = false;
        }
        Ok(())
    }

    /// Clear the current line.
    pub fn clear_line(&self) -> Result<()> {
        self.output.write_stderr("\r\x1b[K")?;
        Ok(())
    }

    /// Write raw text to stderr.
    pub fn write_stderr(&self, text: &str) -> Result<()> {
        self.output.write_stderr(text)?;
        Ok(())
    }

    /// Write raw text to stdout.
    pub fn write_stdout(&self, text: &str) -> Result<()> {
        self.output.write_stdout(text)?;
        Ok(())
    }
}

/// Builder for creating a customized Shell.
#[derive(Debug, Default)]
pub struct ShellBuilder {
    verbosity: Option<Verbosity>,
    color_choice: Option<ColorChoice>,
    theme: Option<Theme>,
    output: Option<ShellOut>,
    output_mode: Option<OutputMode>,
}

impl ShellBuilder {
    /// Create a new shell builder.
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

    /// Set the theme.
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Set a custom output (for testing).
    pub fn output(mut self, output: ShellOut) -> Self {
        self.output = Some(output);
        self
    }

    /// Set the output mode (RFC 0031).
    pub fn output_mode(mut self, mode: OutputMode) -> Self {
        self.output_mode = Some(mode);
        self
    }

    /// Build the shell.
    pub fn build(self) -> Shell {
        let mut output = self.output.unwrap_or_else(ShellOut::stream);

        if let Some(color_choice) = self.color_choice {
            output.set_color_choice(color_choice);
        }

        let output_mode = self.output_mode.unwrap_or_default();

        // In JSON mode, force no-color
        if output_mode == OutputMode::Json {
            output.set_color_choice(ColorChoice::Never);
        }

        Shell {
            output,
            verbosity: self.verbosity.unwrap_or_default(),
            theme: self.theme.unwrap_or_default(),
            needs_clear: false,
            output_mode,
            #[cfg(feature = "progress")]
            progress_manager: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbosity_default() {
        assert_eq!(Verbosity::default(), Verbosity::Normal);
    }

    #[test]
    fn test_shell_builder() {
        let shell = Shell::builder()
            .verbosity(Verbosity::Verbose)
            .color_choice(ColorChoice::Never)
            .build();

        assert_eq!(shell.verbosity(), Verbosity::Verbose);
        assert_eq!(shell.color_choice(), ColorChoice::Never);
    }
}
