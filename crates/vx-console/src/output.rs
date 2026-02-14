//! Output abstraction for Shell.
//!
//! This module provides the `ShellOut` enum that abstracts over different
//! output destinations (streams vs. buffers).

use crate::Result;
use crate::term::Term;

use std::io::{self, Write};
use std::sync::Mutex;

/// Color choice for terminal output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorChoice {
    /// Always use colors.
    Always,
    /// Never use colors.
    Never,
    /// Auto-detect based on terminal capabilities.
    #[default]
    Auto,
}

impl ColorChoice {
    /// Check if colors should be used based on this choice and terminal capabilities.
    pub fn should_use_color(&self, is_tty: bool) -> bool {
        match self {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => is_tty && !is_ci_no_color(),
        }
    }
}

/// Check if we're in a CI environment that doesn't support colors.
fn is_ci_no_color() -> bool {
    // NO_COLOR environment variable (https://no-color.org/)
    if std::env::var("NO_COLOR").is_ok() {
        return true;
    }

    // TERM=dumb
    if std::env::var("TERM").map(|t| t == "dumb").unwrap_or(false) {
        return true;
    }

    false
}

/// Output destination for Shell.
/// Output destination for Shell.
pub enum ShellOut {
    /// Output to stdout/stderr streams.
    Stream {
        stdout: Mutex<io::Stdout>,
        stderr: Mutex<io::Stderr>,
        stderr_tty: bool,
        color_choice: ColorChoice,
    },
    /// Output to a custom writer (for testing).
    Write(Mutex<Box<dyn Write + Send>>),
}

impl std::fmt::Debug for ShellOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellOut::Stream {
                stderr_tty,
                color_choice,
                ..
            } => f
                .debug_struct("Stream")
                .field("stderr_tty", stderr_tty)
                .field("color_choice", color_choice)
                .finish(),
            ShellOut::Write(_) => f.debug_struct("Write").finish(),
        }
    }
}

impl ShellOut {
    /// Create a new stream output.
    pub fn stream() -> Self {
        let term = Term::detect();
        ShellOut::Stream {
            stdout: Mutex::new(io::stdout()),
            stderr: Mutex::new(io::stderr()),
            stderr_tty: term.is_tty(),
            color_choice: ColorChoice::Auto,
        }
    }

    /// Get the current color choice.
    pub fn color_choice(&self) -> ColorChoice {
        match self {
            ShellOut::Stream { color_choice, .. } => *color_choice,
            ShellOut::Write(_) => ColorChoice::Never,
        }
    }

    /// Set the color choice.
    pub fn set_color_choice(&mut self, choice: ColorChoice) {
        if let ShellOut::Stream { color_choice, .. } = self {
            *color_choice = choice;
        }
    }

    /// Check if colors are supported.
    pub fn supports_color(&self) -> bool {
        match self {
            ShellOut::Stream {
                stderr_tty,
                color_choice,
                ..
            } => color_choice.should_use_color(*stderr_tty),
            ShellOut::Write(_) => false,
        }
    }

    /// Check if stderr is a TTY.
    pub fn is_tty(&self) -> bool {
        match self {
            ShellOut::Stream { stderr_tty, .. } => *stderr_tty,
            ShellOut::Write(_) => false,
        }
    }

    /// Write to stderr.
    pub fn write_stderr(&self, text: &str) -> Result<()> {
        match self {
            ShellOut::Stream { stderr, .. } => {
                let mut stderr = stderr
                    .lock()
                    .map_err(|_| io::Error::other("Failed to lock stderr"))?;
                write!(stderr, "{}", text)?;
                stderr.flush()?;
            }
            ShellOut::Write(w) => {
                if let Ok(mut writer) = w.lock() {
                    write!(writer, "{}", text)?;
                    writer.flush()?;
                }
            }
        }
        Ok(())
    }

    /// Write to stdout.
    pub fn write_stdout(&self, text: &str) -> Result<()> {
        match self {
            ShellOut::Stream { stdout, .. } => {
                let mut stdout = stdout
                    .lock()
                    .map_err(|_| io::Error::other("Failed to lock stdout"))?;
                write!(stdout, "{}", text)?;
                stdout.flush()?;
            }
            ShellOut::Write(w) => {
                if let Ok(mut writer) = w.lock() {
                    write!(writer, "{}", text)?;
                    writer.flush()?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_choice_always() {
        assert!(ColorChoice::Always.should_use_color(false));
        assert!(ColorChoice::Always.should_use_color(true));
    }

    #[test]
    fn test_color_choice_never() {
        assert!(!ColorChoice::Never.should_use_color(false));
        assert!(!ColorChoice::Never.should_use_color(true));
    }

    #[test]
    fn test_color_choice_auto() {
        // Auto should return false for non-TTY
        assert!(!ColorChoice::Auto.should_use_color(false));
    }
}
