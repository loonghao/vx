//! Style and theme definitions.
//!
//! This module provides styling primitives using anstyle for zero-overhead
//! style definitions.

use std::fmt;

/// ANSI colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    /// 256-color palette.
    Ansi256(u8),
    /// RGB color.
    Rgb(u8, u8, u8),
}

impl Color {
    /// Convert to ANSI escape code for foreground.
    pub fn to_fg_code(&self) -> String {
        match self {
            Color::Black => "30".to_string(),
            Color::Red => "31".to_string(),
            Color::Green => "32".to_string(),
            Color::Yellow => "33".to_string(),
            Color::Blue => "34".to_string(),
            Color::Magenta => "35".to_string(),
            Color::Cyan => "36".to_string(),
            Color::White => "37".to_string(),
            Color::BrightBlack => "90".to_string(),
            Color::BrightRed => "91".to_string(),
            Color::BrightGreen => "92".to_string(),
            Color::BrightYellow => "93".to_string(),
            Color::BrightBlue => "94".to_string(),
            Color::BrightMagenta => "95".to_string(),
            Color::BrightCyan => "96".to_string(),
            Color::BrightWhite => "97".to_string(),
            Color::Ansi256(n) => format!("38;5;{}", n),
            Color::Rgb(r, g, b) => format!("38;2;{};{};{}", r, g, b),
        }
    }

    /// Convert to ANSI escape code for background.
    pub fn to_bg_code(&self) -> String {
        match self {
            Color::Black => "40".to_string(),
            Color::Red => "41".to_string(),
            Color::Green => "42".to_string(),
            Color::Yellow => "43".to_string(),
            Color::Blue => "44".to_string(),
            Color::Magenta => "45".to_string(),
            Color::Cyan => "46".to_string(),
            Color::White => "47".to_string(),
            Color::BrightBlack => "100".to_string(),
            Color::BrightRed => "101".to_string(),
            Color::BrightGreen => "102".to_string(),
            Color::BrightYellow => "103".to_string(),
            Color::BrightBlue => "104".to_string(),
            Color::BrightMagenta => "105".to_string(),
            Color::BrightCyan => "106".to_string(),
            Color::BrightWhite => "107".to_string(),
            Color::Ansi256(n) => format!("48;5;{}", n),
            Color::Rgb(r, g, b) => format!("48;2;{};{};{}", r, g, b),
        }
    }
}

/// Text style.
#[derive(Debug, Clone, Default)]
pub struct Style {
    fg: Option<Color>,
    bg: Option<Color>,
    bold: bool,
    dimmed: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
}

impl Style {
    /// Create a new empty style.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the foreground color.
    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    /// Set the background color.
    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    /// Make the text bold.
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Make the text dimmed.
    pub fn dimmed(mut self) -> Self {
        self.dimmed = true;
        self
    }

    /// Make the text italic.
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Make the text underlined.
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// Make the text strikethrough.
    pub fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    /// Apply this style to a string.
    pub fn apply(&self, text: &str) -> String {
        let mut codes = Vec::new();

        if self.bold {
            codes.push("1".to_string());
        }
        if self.dimmed {
            codes.push("2".to_string());
        }
        if self.italic {
            codes.push("3".to_string());
        }
        if self.underline {
            codes.push("4".to_string());
        }
        if self.strikethrough {
            codes.push("9".to_string());
        }
        if let Some(ref fg) = self.fg {
            codes.push(fg.to_fg_code());
        }
        if let Some(ref bg) = self.bg {
            codes.push(bg.to_bg_code());
        }

        if codes.is_empty() {
            text.to_string()
        } else {
            format!("\x1b[{}m{}\x1b[0m", codes.join(";"), text)
        }
    }
}

/// Theme for console output.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Success style (e.g., green).
    pub success: Style,
    /// Error style (e.g., red).
    pub error: Style,
    /// Warning style (e.g., yellow).
    pub warn: Style,
    /// Info style (e.g., blue).
    pub info: Style,
    /// Hint style (e.g., cyan).
    pub hint: Style,
    /// Debug style (e.g., magenta).
    pub debug: Style,
    /// Success prefix character.
    pub success_prefix: String,
    /// Error prefix character.
    pub error_prefix: String,
    /// Warning prefix character.
    pub warn_prefix: String,
    /// Info prefix character.
    pub info_prefix: String,
    /// Hint prefix character.
    pub hint_prefix: String,
    /// Debug prefix character.
    pub debug_prefix: String,
    /// Spinner characters.
    pub spinner_chars: Vec<String>,
    /// Progress bar characters.
    pub progress_chars: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            success: Style::new().fg(Color::Green).bold(),
            error: Style::new().fg(Color::Red).bold(),
            warn: Style::new().fg(Color::Yellow).bold(),
            info: Style::new().fg(Color::Blue).bold(),
            hint: Style::new().fg(Color::Cyan),
            debug: Style::new().fg(Color::Magenta),
            success_prefix: "✓".to_string(),
            error_prefix: "✗".to_string(),
            warn_prefix: "⚠".to_string(),
            info_prefix: "ℹ".to_string(),
            hint_prefix: "💡".to_string(),
            debug_prefix: "→".to_string(),
            spinner_chars: vec![
                "⠋".to_string(),
                "⠙".to_string(),
                "⠹".to_string(),
                "⠸".to_string(),
                "⠼".to_string(),
                "⠴".to_string(),
                "⠦".to_string(),
                "⠧".to_string(),
                "⠇".to_string(),
                "⠏".to_string(),
            ],
            progress_chars: "━━╺".to_string(),
        }
    }
}

impl Theme {
    /// Create a new theme builder.
    pub fn builder() -> ThemeBuilder {
        ThemeBuilder::default()
    }

    /// Create a minimal theme (no colors, ASCII characters).
    pub fn minimal() -> Self {
        Self {
            success: Style::new(),
            error: Style::new(),
            warn: Style::new(),
            info: Style::new(),
            hint: Style::new(),
            debug: Style::new(),
            success_prefix: "[OK]".to_string(),
            error_prefix: "[ERROR]".to_string(),
            warn_prefix: "[WARN]".to_string(),
            info_prefix: "[INFO]".to_string(),
            hint_prefix: "[HINT]".to_string(),
            debug_prefix: "[DEBUG]".to_string(),
            spinner_chars: vec![
                "|".to_string(),
                "/".to_string(),
                "-".to_string(),
                "\\".to_string(),
            ],
            progress_chars: "=>-".to_string(),
        }
    }

    /// Create a colorful theme.
    pub fn colorful() -> Self {
        Self {
            success: Style::new().fg(Color::BrightGreen).bold(),
            error: Style::new().fg(Color::BrightRed).bold(),
            warn: Style::new().fg(Color::BrightYellow).bold(),
            info: Style::new().fg(Color::BrightBlue).bold(),
            hint: Style::new().fg(Color::BrightCyan),
            debug: Style::new().fg(Color::BrightMagenta),
            ..Self::default()
        }
    }

    /// Create a GitHub Actions compatible theme.
    pub fn github() -> Self {
        Self {
            success_prefix: "::notice::".to_string(),
            error_prefix: "::error::".to_string(),
            warn_prefix: "::warning::".to_string(),
            info_prefix: "".to_string(),
            hint_prefix: "".to_string(),
            debug_prefix: "::debug::".to_string(),
            ..Self::minimal()
        }
    }

    /// Create a Cargo-style theme.
    pub fn cargo() -> Self {
        Self {
            success: Style::new().fg(Color::Green).bold(),
            error: Style::new().fg(Color::Red).bold(),
            warn: Style::new().fg(Color::Yellow).bold(),
            info: Style::new().fg(Color::Cyan).bold(),
            hint: Style::new().fg(Color::Cyan),
            debug: Style::new().dimmed(),
            success_prefix: "   Finished".to_string(),
            error_prefix: "error".to_string(),
            warn_prefix: "warning".to_string(),
            info_prefix: "".to_string(),
            hint_prefix: "".to_string(),
            debug_prefix: "".to_string(),
            spinner_chars: vec![
                "⠋".to_string(),
                "⠙".to_string(),
                "⠹".to_string(),
                "⠸".to_string(),
                "⠼".to_string(),
                "⠴".to_string(),
                "⠦".to_string(),
                "⠧".to_string(),
                "⠇".to_string(),
                "⠏".to_string(),
            ],
            progress_chars: "━━╺".to_string(),
        }
    }

    /// Create a npm-style theme.
    pub fn npm() -> Self {
        Self {
            success: Style::new().fg(Color::Green),
            error: Style::new().fg(Color::Red).bold(),
            warn: Style::new().fg(Color::Yellow),
            info: Style::new().fg(Color::Cyan),
            hint: Style::new().fg(Color::Magenta),
            debug: Style::new().dimmed(),
            success_prefix: "✔".to_string(),
            error_prefix: "✖".to_string(),
            warn_prefix: "⚠".to_string(),
            info_prefix: "ℹ".to_string(),
            hint_prefix: "→".to_string(),
            debug_prefix: "·".to_string(),
            spinner_chars: vec![
                "⠋".to_string(),
                "⠙".to_string(),
                "⠹".to_string(),
                "⠸".to_string(),
                "⠼".to_string(),
                "⠴".to_string(),
                "⠦".to_string(),
                "⠧".to_string(),
                "⠇".to_string(),
                "⠏".to_string(),
            ],
            progress_chars: "█░░".to_string(),
        }
    }

    /// Create a simple emoji theme.
    pub fn emoji() -> Self {
        Self {
            success: Style::new(),
            error: Style::new(),
            warn: Style::new(),
            info: Style::new(),
            hint: Style::new(),
            debug: Style::new().dimmed(),
            success_prefix: "✅".to_string(),
            error_prefix: "❌".to_string(),
            warn_prefix: "⚠️".to_string(),
            info_prefix: "ℹ️".to_string(),
            hint_prefix: "💡".to_string(),
            debug_prefix: "🔍".to_string(),
            spinner_chars: vec![
                "🕐".to_string(),
                "🕑".to_string(),
                "🕒".to_string(),
                "🕓".to_string(),
                "🕔".to_string(),
                "🕕".to_string(),
                "🕖".to_string(),
                "🕗".to_string(),
                "🕘".to_string(),
                "🕙".to_string(),
                "🕚".to_string(),
                "🕛".to_string(),
            ],
            progress_chars: "🟩🟩⬜".to_string(),
        }
    }

    /// Create a monochrome theme (no colors, but with Unicode).
    pub fn monochrome() -> Self {
        Self {
            success: Style::new(),
            error: Style::new().bold(),
            warn: Style::new(),
            info: Style::new(),
            hint: Style::new().dimmed(),
            debug: Style::new().dimmed(),
            success_prefix: "✓".to_string(),
            error_prefix: "✗".to_string(),
            warn_prefix: "!".to_string(),
            info_prefix: "·".to_string(),
            hint_prefix: "→".to_string(),
            debug_prefix: "…".to_string(),
            spinner_chars: vec![
                "⠋".to_string(),
                "⠙".to_string(),
                "⠹".to_string(),
                "⠸".to_string(),
                "⠼".to_string(),
                "⠴".to_string(),
                "⠦".to_string(),
                "⠧".to_string(),
                "⠇".to_string(),
                "⠏".to_string(),
            ],
            progress_chars: "━━╺".to_string(),
        }
    }

    /// Detect the best theme based on terminal capabilities.
    pub fn detect() -> Self {
        use crate::term::{Term, TerminalType};

        let term = Term::detect();

        // Use minimal theme for non-Unicode terminals
        if !term.supports_unicode() {
            return Self::minimal();
        }

        // Use GitHub theme in GitHub Actions
        if let Some(crate::term::CiEnvironment::GitHubActions) = term.ci_environment() {
            return Self::github();
        }

        // Use monochrome theme if colors are not supported
        if !term.supports_color() {
            return Self::monochrome();
        }

        // Use colorful theme for modern terminals
        match term.terminal_type() {
            TerminalType::WindowsTerminal
            | TerminalType::ITerm2
            | TerminalType::VSCode
            | TerminalType::WezTerm
            | TerminalType::Kitty => Self::colorful(),
            _ => Self::default(),
        }
    }

    /// Get the success prefix.
    pub fn success_prefix(&self) -> &str {
        &self.success_prefix
    }

    /// Get the error prefix.
    pub fn error_prefix(&self) -> &str {
        &self.error_prefix
    }

    /// Get the warning prefix.
    pub fn warn_prefix(&self) -> &str {
        &self.warn_prefix
    }

    /// Get the info prefix.
    pub fn info_prefix(&self) -> &str {
        &self.info_prefix
    }

    /// Get the hint prefix.
    pub fn hint_prefix(&self) -> &str {
        &self.hint_prefix
    }

    /// Get the debug prefix.
    pub fn debug_prefix(&self) -> &str {
        &self.debug_prefix
    }

    /// Get the success style.
    pub fn success_style(&self) -> Style {
        self.success.clone()
    }

    /// Get the error style.
    pub fn error_style(&self) -> Style {
        self.error.clone()
    }

    /// Get the warning style.
    pub fn warn_style(&self) -> Style {
        self.warn.clone()
    }

    /// Get the info style.
    pub fn info_style(&self) -> Style {
        self.info.clone()
    }

    /// Get the hint style.
    pub fn hint_style(&self) -> Style {
        self.hint.clone()
    }

    /// Get the debug style.
    pub fn debug_style(&self) -> Style {
        self.debug.clone()
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Style {{ bold: {}, dimmed: {} }}",
            self.bold, self.dimmed
        )
    }
}

/// Builder for creating a custom theme.
#[derive(Debug, Default)]
pub struct ThemeBuilder {
    theme: Theme,
}

impl ThemeBuilder {
    /// Set the success style.
    pub fn success(mut self, style: Style) -> Self {
        self.theme.success = style;
        self
    }

    /// Set the error style.
    pub fn error(mut self, style: Style) -> Self {
        self.theme.error = style;
        self
    }

    /// Set the warning style.
    pub fn warn(mut self, style: Style) -> Self {
        self.theme.warn = style;
        self
    }

    /// Set the info style.
    pub fn info(mut self, style: Style) -> Self {
        self.theme.info = style;
        self
    }

    /// Set the hint style.
    pub fn hint(mut self, style: Style) -> Self {
        self.theme.hint = style;
        self
    }

    /// Set the debug style.
    pub fn debug(mut self, style: Style) -> Self {
        self.theme.debug = style;
        self
    }

    /// Set the spinner characters.
    pub fn spinner_chars(mut self, chars: &[&str]) -> Self {
        self.theme.spinner_chars = chars.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set the progress bar characters.
    pub fn progress_chars(mut self, chars: &str) -> Self {
        self.theme.progress_chars = chars.to_string();
        self
    }

    /// Build the theme.
    pub fn build(self) -> Theme {
        self.theme
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_apply() {
        let style = Style::new().fg(Color::Green).bold();
        let result = style.apply("test");
        assert!(result.contains("\x1b["));
        assert!(result.contains("test"));
        assert!(result.contains("\x1b[0m"));
    }

    #[test]
    fn test_style_empty() {
        let style = Style::new();
        let result = style.apply("test");
        assert_eq!(result, "test");
    }

    #[test]
    fn test_theme_default() {
        let theme = Theme::default();
        assert_eq!(theme.success_prefix(), "✓");
        assert_eq!(theme.error_prefix(), "✗");
    }

    #[test]
    fn test_theme_minimal() {
        let theme = Theme::minimal();
        assert_eq!(theme.success_prefix(), "[OK]");
        assert_eq!(theme.error_prefix(), "[ERROR]");
    }
}
