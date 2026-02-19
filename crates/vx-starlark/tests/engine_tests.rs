//! Tests for the Starlark execution engine

use std::path::Path;
use vx_starlark::StarlarkEngine;
use vx_starlark::context::ProviderContext;

/// Helper: create a minimal ProviderContext for testing
fn test_ctx() -> ProviderContext {
    let vx_home = std::env::temp_dir().join("vx-test");
    ProviderContext::new("test-provider", vx_home)
}

// ============================================================
// Basic function execution
// ============================================================

#[test]
fn test_call_function_returns_string() {
    let engine = StarlarkEngine::new();
    let ctx = test_ctx();

    let script = r#"
def greet(ctx):
    return "hello world"
"#;

    let result = engine.call_function(
        Path::new("test.star"),
        script,
        "greet",
        &ctx,
        &[],
    );

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap(), serde_json::json!("hello world"));
}

#[test]
fn test_call_function_returns_int() {
    let engine = StarlarkEngine::new();
    let ctx = test_ctx();

    let script = r#"
def answer(ctx):
    return 42
"#;

    let result = engine.call_function(
        Path::new("test.star"),
        script,
        "answer",
        &ctx,
        &[],
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), serde_json::json!(42));
}

#[test]
fn test_call_function_returns_bool() {
    let engine = StarlarkEngine::new();
    let ctx = test_ctx();

    let script = r#"
def is_ready(ctx):
    return True
"#;

    let result = engine.call_function(
        Path::new("test.star"),
        script,
        "is_ready",
        &ctx,
        &[],
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), serde_json::json!(true));
}

#[test]
fn test_call_function_returns_none() {
    let engine = StarlarkEngine::new();
    let ctx = test_ctx();

    let script = r#"
def nothing(ctx):
    return None
"#;

    let result = engine.call_function(
        Path::new("test.star"),
        script,
        "nothing",
        &ctx,
        &[],
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), serde_json::json!(null));
}

#[test]
fn test_call_function_returns_list() {
    let engine = StarlarkEngine::new();
    let ctx = test_ctx();

    let script = r#"
def get_versions(ctx):
    return [
        {"version": "1.0.0", "lts": True},
        {"version": "2.0.0", "lts": False},
    ]
"#;

    let result = engine.call_function(
        Path::new("test.star"),
        script,
        "get_versions",
        &ctx,
        &[],
    );

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    let json = result.unwrap();
    let arr = json.as_array().expect("Expected array");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["version"], "1.0.0");
    assert_eq!(arr[0]["lts"], true);
    assert_eq!(arr[1]["version"], "2.0.0");
    assert_eq!(arr[1]["lts"], false);
}

// ============================================================
// Context injection
// ============================================================

#[test]
fn test_ctx_platform_os_accessible() {
    let engine = StarlarkEngine::new();
    let ctx = test_ctx();

    let script = r#"
def get_os(ctx):
    return ctx["platform"]["os"]
"#;

    let result = engine.call_function(
        Path::new("test.star"),
        script,
        "get_os",
        &ctx,
        &[],
    );

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    let os = result.unwrap();
    // Should be one of: "windows", "linux", "macos"
    let os_str = os.as_str().expect("Expected string");
    assert!(
        ["windows", "linux", "macos"].contains(&os_str),
        "Unexpected OS: {}",
        os_str
    );
}

#[test]
fn test_ctx_platform_arch_accessible() {
    let engine = StarlarkEngine::new();
    let ctx = test_ctx();

    let script = r#"
def get_arch(ctx):
    return ctx["platform"]["arch"]
"#;

    let result = engine.call_function(
        Path::new("test.star"),
        script,
        "get_arch",
        &ctx,
        &[],
    );

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    let arch = result.unwrap();
    let arch_str = arch.as_str().expect("Expected string");
    assert!(
        ["x64", "arm64", "x86"].contains(&arch_str),
        "Unexpected arch: {}",
        arch_str
    );
}

// ============================================================
// Extra arguments
// ============================================================

#[test]
fn test_call_function_with_version_arg() {
    let engine = StarlarkEngine::new();
    let ctx = test_ctx();

    let script = r#"
def download_url(ctx, version):
    return "https://example.com/releases/" + version + "/tool.tar.gz"
"#;

    let result = engine.call_function(
        Path::new("test.star"),
        script,
        "download_url",
        &ctx,
        &[serde_json::json!("1.2.3")],
    );

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    assert_eq!(
        result.unwrap(),
        serde_json::json!("https://example.com/releases/1.2.3/tool.tar.gz")
    );
}

// ============================================================
// Error handling
// ============================================================

#[test]
fn test_call_function_not_found() {
    let engine = StarlarkEngine::new();
    let ctx = test_ctx();

    let script = r#"
def existing_func(ctx):
    return "exists"
"#;

    let result = engine.call_function(
        Path::new("test.star"),
        script,
        "nonexistent_func",
        &ctx,
        &[],
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("nonexistent_func"),
        "Error should mention function name: {}",
        err
    );
}

#[test]
fn test_call_function_parse_error() {
    let engine = StarlarkEngine::new();
    let ctx = test_ctx();

    // Invalid Starlark syntax
    let script = r#"
def broken(ctx)
    return "missing colon"
"#;

    let result = engine.call_function(
        Path::new("test.star"),
        script,
        "broken",
        &ctx,
        &[],
    );

    assert!(result.is_err());
}

#[test]
fn test_call_function_runtime_error() {
    let engine = StarlarkEngine::new();
    let ctx = test_ctx();

    // Division by zero
    let script = r#"
def bad_func(ctx):
    x = 1 // 0
    return x
"#;

    let result = engine.call_function(
        Path::new("test.star"),
        script,
        "bad_func",
        &ctx,
        &[],
    );

    assert!(result.is_err());
}

// ============================================================
// Starlark language features
// ============================================================

#[test]
fn test_starlark_list_comprehension() {
    let engine = StarlarkEngine::new();
    let ctx = test_ctx();

    let script = r#"
def get_even_numbers(ctx):
    return [x for x in range(10) if x % 2 == 0]
"#;

    let result = engine.call_function(
        Path::new("test.star"),
        script,
        "get_even_numbers",
        &ctx,
        &[],
    );

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    let arr = result.unwrap();
    let nums: Vec<i64> = arr.as_array().unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect();
    assert_eq!(nums, vec![0, 2, 4, 6, 8]);
}

#[test]
fn test_starlark_string_format() {
    let engine = StarlarkEngine::new();
    let ctx = test_ctx();

    let script = r#"
def format_url(ctx, version):
    return "https://example.com/v{}/tool.zip".format(version)
"#;

    let result = engine.call_function(
        Path::new("test.star"),
        script,
        "format_url",
        &ctx,
        &[serde_json::json!("3.0.0")],
    );

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    assert_eq!(
        result.unwrap(),
        serde_json::json!("https://example.com/v3.0.0/tool.zip")
    );
}

#[test]
fn test_starlark_dict_operations() {
    let engine = StarlarkEngine::new();
    let ctx = test_ctx();

    let script = r#"
def get_env(ctx, version):
    return {
        "TOOL_HOME": "/opt/tool/" + version,
        "TOOL_VERSION": version,
    }
"#;

    let result = engine.call_function(
        Path::new("test.star"),
        script,
        "get_env",
        &ctx,
        &[serde_json::json!("2.0.0")],
    );

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    let obj = result.unwrap();
    assert_eq!(obj["TOOL_HOME"], "/opt/tool/2.0.0");
    assert_eq!(obj["TOOL_VERSION"], "2.0.0");
}

#[test]
fn test_starlark_helper_functions() {
    let engine = StarlarkEngine::new();
    let ctx = test_ctx();

    // Test that helper functions defined in the script can be called from main function
    let script = r#"
def _strip_v(version):
    if version.startswith("v"):
        return version[1:]
    return version

def normalize_version(ctx, version):
    return _strip_v(version)
"#;

    let result = engine.call_function(
        Path::new("test.star"),
        script,
        "normalize_version",
        &ctx,
        &[serde_json::json!("v1.2.3")],
    );

    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap(), serde_json::json!("1.2.3"));
}
