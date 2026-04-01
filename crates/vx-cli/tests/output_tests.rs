//! Tests for the unified structured output system (RFC 0031)

use anyhow::Result;
use serde::Serialize;
use vx_cli::cli::OutputFormat;
use vx_cli::output::{CommandOutput, OutputRenderer};

#[derive(Serialize)]
struct TestOutput {
    name: String,
    count: u32,
}

impl CommandOutput for TestOutput {
    fn render_text(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        writeln!(writer, "Name: {}", self.name)?;
        writeln!(writer, "Count: {}", self.count)?;
        Ok(())
    }
}

fn test_output() -> TestOutput {
    TestOutput {
        name: "test".to_string(),
        count: 42,
    }
}

#[test]
fn test_render_json() {
    let renderer = OutputRenderer::json();
    let result = renderer.render_to_string(&test_output()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(json["name"], "test");
    assert_eq!(json["count"], 42);
}

#[test]
fn test_render_text() {
    let renderer = OutputRenderer::text();
    let result = renderer.render_to_string(&test_output()).unwrap();
    assert!(result.contains("Name: test"));
    assert!(result.contains("Count: 42"));
}

#[test]
fn test_render_toon_supported() {
    // TOON format is implemented via toon_format::encode_default
    let renderer = OutputRenderer::new(OutputFormat::Toon);
    let result = renderer.render_to_string(&test_output());
    // TOON format should succeed and produce non-empty output
    assert!(
        result.is_ok(),
        "TOON format should be supported: {:?}",
        result.err()
    );
    let toon = result.unwrap();
    assert!(!toon.is_empty(), "TOON output should not be empty");
}

#[test]
fn test_renderer_is_json() {
    assert!(OutputRenderer::json().is_json());
    assert!(!OutputRenderer::text().is_json());
}

#[test]
fn test_renderer_format() {
    // Use new_exact() to bypass TTY detection — OutputRenderer::new(Text) may
    // auto-upgrade to Json in non-TTY environments (e.g. CI). This test only
    // verifies that the format field is stored correctly when set explicitly.
    let renderer = OutputRenderer::new_exact(OutputFormat::Text);
    assert_eq!(renderer.format(), OutputFormat::Text);

    let renderer = OutputRenderer::new_exact(OutputFormat::Json);
    assert_eq!(renderer.format(), OutputFormat::Json);
}

#[test]
fn test_renderer_new_auto_upgrades_in_non_tty() {
    // OutputRenderer::new(Text) should auto-upgrade to Json outside a TTY
    // (unless VX_OUTPUT=text is set). We can't control whether CI is a TTY,
    // so just verify that the result is either Text or Json — never Toon.
    let renderer = OutputRenderer::new(OutputFormat::Text);
    assert!(
        renderer.format() == OutputFormat::Text || renderer.format() == OutputFormat::Json,
        "new(Text) should produce Text or Json, never Toon"
    );
}
