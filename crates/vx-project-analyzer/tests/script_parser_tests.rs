//! Tests for the script parser
//!
//! The script parser detects tool invocations via specific patterns:
//! - uv run <tool>
//! - uvx <tool>
//! - npx <tool>
//! - python -m <module>
//! - pnpm exec <tool>
//! - yarn <tool>
//! - bunx <tool>
//!
//! Note: Direct command invocations (like `pytest`, `cargo`) are NOT detected
//! as they require more context about what tools are expected.

use rstest::rstest;
use vx_project_analyzer::{ScriptParser, ToolInvocation};

// ============================================
// UV/UVX Pattern Tests
// ============================================

#[rstest]
#[case("uv run pytest", "pytest", ToolInvocation::UvRun)]
#[case("uv run ruff check .", "ruff", ToolInvocation::UvRun)]
#[case("uv run mypy src/", "mypy", ToolInvocation::UvRun)]
fn test_uv_run_pattern(
    #[case] script: &str,
    #[case] expected_tool: &str,
    #[case] expected_invocation: ToolInvocation,
) {
    let parser = ScriptParser::new();
    let tools = parser.parse(script);

    assert_eq!(tools.len(), 1, "Should detect one tool in: {}", script);
    assert_eq!(tools[0].name, expected_tool);
    assert_eq!(tools[0].invocation, expected_invocation);
}

#[rstest]
#[case("uvx ruff check .", "ruff", ToolInvocation::Uvx)]
#[case("uvx black .", "black", ToolInvocation::Uvx)]
#[case("uvx mypy src/", "mypy", ToolInvocation::Uvx)]
fn test_uvx_pattern(
    #[case] script: &str,
    #[case] expected_tool: &str,
    #[case] expected_invocation: ToolInvocation,
) {
    let parser = ScriptParser::new();
    let tools = parser.parse(script);

    assert_eq!(tools.len(), 1, "Should detect one tool in: {}", script);
    assert_eq!(tools[0].name, expected_tool);
    assert_eq!(tools[0].invocation, expected_invocation);
}

// ============================================
// NPX Pattern Tests
// ============================================

#[rstest]
#[case("npx eslint .", "eslint", ToolInvocation::Npx)]
#[case("npx --yes prettier --write .", "prettier", ToolInvocation::Npx)]
#[case("npx tsc --noEmit", "tsc", ToolInvocation::Npx)]
#[case("npx @biomejs/biome check .", "@biomejs/biome", ToolInvocation::Npx)]
fn test_npx_pattern(
    #[case] script: &str,
    #[case] expected_tool: &str,
    #[case] expected_invocation: ToolInvocation,
) {
    let parser = ScriptParser::new();
    let tools = parser.parse(script);

    assert_eq!(tools.len(), 1, "Should detect one tool in: {}", script);
    assert_eq!(tools[0].name, expected_tool);
    assert_eq!(tools[0].invocation, expected_invocation);
}

// ============================================
// Python Module Pattern Tests
// ============================================

#[rstest]
#[case("python -m pytest", "pytest", ToolInvocation::PythonModule)]
#[case("python -m mypy src/", "mypy", ToolInvocation::PythonModule)]
#[case("python3 -m black .", "black", ToolInvocation::PythonModule)]
fn test_python_module_pattern(
    #[case] script: &str,
    #[case] expected_tool: &str,
    #[case] expected_invocation: ToolInvocation,
) {
    let parser = ScriptParser::new();
    let tools = parser.parse(script);

    assert_eq!(tools.len(), 1, "Should detect one tool in: {}", script);
    assert_eq!(tools[0].name, expected_tool);
    assert_eq!(tools[0].invocation, expected_invocation);
}

// ============================================
// Bunx Pattern Tests
// ============================================

#[rstest]
#[case("bunx eslint .", "eslint", ToolInvocation::Bunx)]
#[case("bunx prettier --write .", "prettier", ToolInvocation::Bunx)]
fn test_bunx_pattern(
    #[case] script: &str,
    #[case] expected_tool: &str,
    #[case] expected_invocation: ToolInvocation,
) {
    let parser = ScriptParser::new();
    let tools = parser.parse(script);

    assert_eq!(tools.len(), 1, "Should detect one tool in: {}", script);
    assert_eq!(tools[0].name, expected_tool);
    assert_eq!(tools[0].invocation, expected_invocation);
}

// ============================================
// Direct Commands (NOT detected)
// ============================================

#[rstest]
#[case("pytest")]
#[case("cargo test")]
#[case("cargo build --release")]
#[case("npm run test")]
#[case("npm install")]
#[case("echo 'hello'")]
#[case("cd src && ls")]
fn test_direct_commands_not_detected(#[case] script: &str) {
    let parser = ScriptParser::new();
    let tools = parser.parse(script);

    // Direct commands are not detected by the parser
    // This is by design - they require more context
    assert!(
        tools.is_empty(),
        "Direct command '{}' should not be detected (found: {:?})",
        script,
        tools.iter().map(|t| &t.name).collect::<Vec<_>>()
    );
}

// ============================================
// Chained Commands Tests
// ============================================

#[rstest]
fn test_chained_uv_commands() {
    let parser = ScriptParser::new();
    let tools = parser.parse("uv run pytest && uv run ruff check .");

    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].name, "pytest");
    assert_eq!(tools[1].name, "ruff");
}

#[rstest]
fn test_chained_npx_commands() {
    let parser = ScriptParser::new();
    let tools = parser.parse("npx eslint . && npx prettier --check .");

    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].name, "eslint");
    assert_eq!(tools[1].name, "prettier");
}

#[rstest]
fn test_mixed_chained_commands() {
    let parser = ScriptParser::new();
    let tools = parser.parse("uvx ruff check . && npx eslint .");

    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].name, "ruff");
    assert_eq!(tools[0].invocation, ToolInvocation::Uvx);
    assert_eq!(tools[1].name, "eslint");
    assert_eq!(tools[1].invocation, ToolInvocation::Npx);
}

// ============================================
// Arguments Parsing Tests
// ============================================

#[rstest]
fn test_tool_with_arguments() {
    let parser = ScriptParser::new();
    let tools = parser.parse("uv run pytest tests/ -v --cov=src");

    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "pytest");
    assert_eq!(tools[0].args, vec!["tests/", "-v", "--cov=src"]);
}

#[rstest]
fn test_npx_with_arguments() {
    let parser = ScriptParser::new();
    let tools = parser.parse("npx eslint . --fix --ext .ts,.tsx");

    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "eslint");
    assert_eq!(tools[0].args, vec![".", "--fix", "--ext", ".ts,.tsx"]);
}

// ============================================
// Edge Cases
// ============================================

#[rstest]
fn test_empty_command() {
    let parser = ScriptParser::new();
    let tools = parser.parse("");

    assert!(tools.is_empty());
}

#[rstest]
fn test_whitespace_only() {
    let parser = ScriptParser::new();
    let tools = parser.parse("   \t\n  ");

    assert!(tools.is_empty());
}

#[rstest]
fn test_semicolon_separator() {
    let parser = ScriptParser::new();
    let tools = parser.parse("uv run pytest; uv run mypy src/");

    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].name, "pytest");
    assert_eq!(tools[1].name, "mypy");
}

#[rstest]
fn test_or_separator() {
    let parser = ScriptParser::new();
    let tools = parser.parse("uv run pytest || uv run pytest --last-failed");

    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].name, "pytest");
    assert_eq!(tools[1].name, "pytest");
}

// ============================================
// Tool Availability (default is false)
// ============================================

#[rstest]
fn test_tool_availability_default() {
    let parser = ScriptParser::new();
    let tools = parser.parse("uv run pytest");

    assert_eq!(tools.len(), 1);
    assert!(
        !tools[0].is_available,
        "Tool should not be available by default"
    );
}
