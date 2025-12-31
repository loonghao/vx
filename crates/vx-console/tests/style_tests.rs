//! Style tests.

use rstest::rstest;
use vx_console::{Color, Style, Theme};

#[rstest]
fn test_style_empty() {
    let style = Style::new();
    let result = style.apply("test");
    assert_eq!(result, "test");
}

#[rstest]
fn test_style_bold() {
    let style = Style::new().bold();
    let result = style.apply("test");
    assert!(result.contains("\x1b["));
    assert!(result.contains("1"));
    assert!(result.contains("test"));
    assert!(result.contains("\x1b[0m"));
}

#[rstest]
fn test_style_fg_color() {
    let style = Style::new().fg(Color::Green);
    let result = style.apply("test");
    assert!(result.contains("\x1b["));
    assert!(result.contains("32")); // Green foreground
    assert!(result.contains("test"));
}

#[rstest]
fn test_style_combined() {
    let style = Style::new().fg(Color::Red).bold().underline();
    let result = style.apply("error");
    assert!(result.contains("\x1b["));
    assert!(result.contains("error"));
    assert!(result.contains("\x1b[0m"));
}

#[rstest]
fn test_style_dimmed() {
    let style = Style::new().dimmed();
    let result = style.apply("dim");
    assert!(result.contains("2")); // Dimmed code
}

#[rstest]
fn test_style_italic() {
    let style = Style::new().italic();
    let result = style.apply("italic");
    assert!(result.contains("3")); // Italic code
}

#[rstest]
fn test_color_fg_codes() {
    assert_eq!(Color::Black.to_fg_code(), "30");
    assert_eq!(Color::Red.to_fg_code(), "31");
    assert_eq!(Color::Green.to_fg_code(), "32");
    assert_eq!(Color::Yellow.to_fg_code(), "33");
    assert_eq!(Color::Blue.to_fg_code(), "34");
    assert_eq!(Color::Magenta.to_fg_code(), "35");
    assert_eq!(Color::Cyan.to_fg_code(), "36");
    assert_eq!(Color::White.to_fg_code(), "37");
}

#[rstest]
fn test_color_bright_fg_codes() {
    assert_eq!(Color::BrightBlack.to_fg_code(), "90");
    assert_eq!(Color::BrightRed.to_fg_code(), "91");
    assert_eq!(Color::BrightGreen.to_fg_code(), "92");
}

#[rstest]
fn test_color_ansi256() {
    let color = Color::Ansi256(123);
    assert_eq!(color.to_fg_code(), "38;5;123");
}

#[rstest]
fn test_color_rgb() {
    let color = Color::Rgb(255, 128, 64);
    assert_eq!(color.to_fg_code(), "38;2;255;128;64");
}

#[rstest]
fn test_theme_default() {
    let theme = Theme::default();
    assert_eq!(theme.success_prefix(), "✓");
    assert_eq!(theme.error_prefix(), "✗");
    assert_eq!(theme.warn_prefix(), "⚠");
    assert_eq!(theme.info_prefix(), "ℹ");
    assert_eq!(theme.hint_prefix(), "💡");
    assert_eq!(theme.debug_prefix(), "→");
}

#[rstest]
fn test_theme_minimal() {
    let theme = Theme::minimal();
    assert_eq!(theme.success_prefix(), "[OK]");
    assert_eq!(theme.error_prefix(), "[ERROR]");
    assert_eq!(theme.warn_prefix(), "[WARN]");
    assert_eq!(theme.info_prefix(), "[INFO]");
}

#[rstest]
fn test_theme_github() {
    let theme = Theme::github();
    assert_eq!(theme.success_prefix(), "::notice::");
    assert_eq!(theme.error_prefix(), "::error::");
    assert_eq!(theme.warn_prefix(), "::warning::");
    assert_eq!(theme.debug_prefix(), "::debug::");
}

#[rstest]
fn test_theme_builder() {
    let theme = Theme::builder()
        .success(Style::new().fg(Color::BrightGreen))
        .spinner_chars(&[".", "..", "...", "...."])
        .progress_chars("=>-")
        .build();

    assert_eq!(theme.progress_chars, "=>-");
    assert_eq!(theme.spinner_chars.len(), 4);
}
