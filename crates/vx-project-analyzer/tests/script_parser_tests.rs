//! Tests for the script parser

use rstest::rstest;
use vx_project_analyzer::ScriptParser;

#[rstest]
#[case("pytest", vec!["pytest"])]
#[case("python -m pytest", vec!["python", "pytest"])]
#[case("npm run test", vec!["npm"])]
#[case("npx eslint .", vec!["npx", "eslint"])]
#[case("uv run pytest", vec!["uv", "pytest"])]
#[case("uvx ruff check .", vec!["uvx", "ruff"])]
#[case("cargo test", vec!["cargo"])]
#[case("cargo build --release", vec!["cargo"])]
fn test_script_parser_basic(#[case] script: &str, #[case] expected_tools: Vec<&str>) {
    let parser = ScriptParser::new();
    let tools = parser.parse(script);

    let tool_names: Vec<_> = tools.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(tool_names, expected_tools);
}

#[rstest]
#[case("echo 'hello'", vec![])]
#[case("cd src && ls", vec![])]
fn test_script_parser_shell_commands(#[case] script: &str, #[case] expected_tools: Vec<&str>) {
    let parser = ScriptParser::new();
    let tools = parser.parse(script);

    let tool_names: Vec<_> = tools.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(tool_names, expected_tools);
}

#[rstest]
#[case("pytest && ruff check .", vec!["pytest", "ruff"])]
#[case("npm install && npm run build", vec!["npm"])]
fn test_script_parser_chained_commands(#[case] script: &str, #[case] expected_tools: Vec<&str>) {
    let parser = ScriptParser::new();
    let tools = parser.parse(script);

    let tool_names: Vec<_> = tools.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(tool_names, expected_tools);
}

#[rstest]
fn test_script_parser_tool_invocation() {
    let parser = ScriptParser::new();
    let tools = parser.parse("npx eslint .");

    assert_eq!(tools.len(), 2);

    // npx
    assert_eq!(tools[0].name, "npx");
    assert_eq!(tools[0].invocation.to_string(), "npx");

    // eslint (via npx)
    assert_eq!(tools[1].name, "eslint");
    assert_eq!(tools[1].invocation.to_string(), "npx:eslint");
}

#[rstest]
fn test_script_parser_uv_invocation() {
    let parser = ScriptParser::new();
    let tools = parser.parse("uvx ruff check .");

    assert_eq!(tools.len(), 2);

    // uvx
    assert_eq!(tools[0].name, "uvx");

    // ruff (via uvx)
    assert_eq!(tools[1].name, "ruff");
    assert_eq!(tools[1].invocation.to_string(), "uvx:ruff");
}
