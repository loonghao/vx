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
fn test_render_toon_not_supported() {
    let renderer = OutputRenderer::new(OutputFormat::Toon);
    let result = renderer.render_to_string(&test_output());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("TOON format is not yet supported"));
}

#[test]
fn test_renderer_is_json() {
    assert!(OutputRenderer::json().is_json());
    assert!(!OutputRenderer::text().is_json());
}

#[test]
fn test_renderer_format() {
    let renderer = OutputRenderer::new(OutputFormat::Text);
    assert_eq!(renderer.format(), OutputFormat::Text);

    let renderer = OutputRenderer::new(OutputFormat::Json);
    assert_eq!(renderer.format(), OutputFormat::Json);
}
