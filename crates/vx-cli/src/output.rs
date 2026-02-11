//! Unified structured output system (RFC 0031)
//!
//! This module provides the `CommandOutput` trait and `OutputRenderer`
//! for unified structured output across all CLI commands.
//!
//! # Design
//!
//! - Commands implement `CommandOutput` to define their data structure
//! - `OutputRenderer` handles format selection (text/json/toon)
//! - Data goes to stdout, logs/progress go to stderr
//! - JSON mode automatically suppresses progress bars and emoji decorations

use crate::cli::OutputFormat;
use anyhow::Result;
use serde::Serialize;

/// Trait for command output that supports multiple render formats.
///
/// Commands implement this trait to enable `--json` and `--format` support.
/// The command only needs to define "what data to return" — the rendering
/// format is controlled by global CLI arguments.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Serialize)]
/// pub struct ListOutput {
///     pub runtimes: Vec<RuntimeEntry>,
/// }
///
/// impl CommandOutput for ListOutput {
///     fn render_text(&self, writer: &mut dyn std::io::Write) -> Result<()> {
///         writeln!(writer, "Installed Runtimes:")?;
///         for rt in &self.runtimes {
///             writeln!(writer, "  {} {}", rt.name, rt.version)?;
///         }
///         Ok(())
///     }
/// }
/// ```
pub trait CommandOutput: Serialize {
    /// Render human-readable text output to the given writer.
    ///
    /// This is called when `--format text` (default) is used.
    /// Output should include colors, emoji, and formatting for human consumption.
    fn render_text(&self, writer: &mut dyn std::io::Write) -> Result<()>;
}

/// Renders command output in the requested format.
///
/// Selects between text, JSON, and (future) TOON output based on
/// the global `--format` / `--json` flags.
pub struct OutputRenderer {
    format: OutputFormat,
}

impl OutputRenderer {
    /// Create a new renderer with the given format.
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Create a renderer for JSON output.
    pub fn json() -> Self {
        Self::new(OutputFormat::Json)
    }

    /// Create a renderer for text output.
    pub fn text() -> Self {
        Self::new(OutputFormat::Text)
    }

    /// Get the current output format.
    pub fn format(&self) -> OutputFormat {
        self.format
    }

    /// Check if JSON output is active.
    pub fn is_json(&self) -> bool {
        self.format == OutputFormat::Json
    }

    /// Render the output in the selected format.
    ///
    /// - Text: calls `output.render_text()` writing to stdout
    /// - JSON: serializes to pretty JSON and prints to stdout
    /// - TOON: not yet supported, returns an error
    pub fn render<T: CommandOutput>(&self, output: &T) -> Result<()> {
        match self.format {
            OutputFormat::Text => {
                let mut stdout = std::io::stdout().lock();
                output.render_text(&mut stdout)?;
                Ok(())
            }
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(output)?;
                println!("{json}");
                Ok(())
            }
            OutputFormat::Toon => {
                anyhow::bail!(
                    "TOON format is not yet supported. Use --json instead.\n\
                     See: https://github.com/toon-format/toon"
                );
            }
        }
    }

    /// Render output to a string (useful for testing).
    pub fn render_to_string<T: CommandOutput>(&self, output: &T) -> Result<String> {
        match self.format {
            OutputFormat::Text => {
                let mut buf = Vec::new();
                output.render_text(&mut buf)?;
                Ok(String::from_utf8(buf)?)
            }
            OutputFormat::Json => Ok(serde_json::to_string_pretty(output)?),
            OutputFormat::Toon => {
                anyhow::bail!("TOON format is not yet supported.");
            }
        }
    }
}
