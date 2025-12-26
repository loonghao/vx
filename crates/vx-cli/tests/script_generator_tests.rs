//! Unit tests for script generator - Tests for generate_wrapper_script function
//!
//! These tests verify that the dynamic script generation produces correct
//! platform-specific scripts for setting environment variables and executing commands.

use rstest::*;
use std::collections::HashMap;

mod common;
use common::cleanup_test_env;

// Import the function under test
use vx_cli::commands::generate_wrapper_script;

/// Test basic script generation with no environment variables
#[rstest]
#[test]
fn test_generate_script_no_env_vars() {
    let env_vars: HashMap<String, String> = HashMap::new();
    let cmd = "echo hello";

    let script = generate_wrapper_script(cmd, &env_vars);

    #[cfg(windows)]
    {
        assert!(script.contains("$ErrorActionPreference = 'Stop'"));
        assert!(script.contains("cmd /c \"echo hello\""));
        assert!(script.contains("exit $LASTEXITCODE"));
    }

    #[cfg(not(windows))]
    {
        assert!(script.contains("#!/usr/bin/env bash"));
        assert!(script.contains("set -euo pipefail"));
        assert!(script.contains("echo hello"));
    }

    cleanup_test_env();
}

/// Test script generation with PATH environment variable
#[rstest]
#[test]
fn test_generate_script_with_path() {
    let mut env_vars: HashMap<String, String> = HashMap::new();

    #[cfg(windows)]
    env_vars.insert(
        "PATH".to_string(),
        r"C:\Users\test\.vx\store\uv\0.7.12;C:\Windows\System32".to_string(),
    );

    #[cfg(not(windows))]
    env_vars.insert(
        "PATH".to_string(),
        "/home/test/.vx/store/uv/0.7.12:/usr/bin".to_string(),
    );

    let cmd = "uv run nox -s tests";

    let script = generate_wrapper_script(cmd, &env_vars);

    #[cfg(windows)]
    {
        assert!(script.contains("$env:PATH = "));
        assert!(script.contains(r"C:\Users\test\.vx\store\uv\0.7.12"));
    }

    #[cfg(not(windows))]
    {
        assert!(script.contains("export PATH="));
        assert!(script.contains("/home/test/.vx/store/uv/0.7.12"));
    }

    cleanup_test_env();
}

/// Test script generation with multiple environment variables
#[rstest]
#[test]
fn test_generate_script_multiple_env_vars() {
    let mut env_vars: HashMap<String, String> = HashMap::new();
    env_vars.insert("VX_DEV".to_string(), "1".to_string());
    env_vars.insert("NODE_ENV".to_string(), "test".to_string());
    env_vars.insert("CI".to_string(), "true".to_string());

    let cmd = "npm test";

    let script = generate_wrapper_script(cmd, &env_vars);

    #[cfg(windows)]
    {
        assert!(script.contains("$env:VX_DEV = '1'"));
        assert!(script.contains("$env:NODE_ENV = 'test'"));
        assert!(script.contains("$env:CI = 'true'"));
    }

    #[cfg(not(windows))]
    {
        assert!(script.contains("export VX_DEV='1'"));
        assert!(script.contains("export NODE_ENV='test'"));
        assert!(script.contains("export CI='true'"));
    }

    cleanup_test_env();
}

/// Test script generation with special characters in values (single quotes)
#[rstest]
#[test]
fn test_generate_script_escape_single_quotes() {
    let mut env_vars: HashMap<String, String> = HashMap::new();
    env_vars.insert("MESSAGE".to_string(), "It's a test".to_string());

    let cmd = "echo $MESSAGE";

    let script = generate_wrapper_script(cmd, &env_vars);

    #[cfg(windows)]
    {
        // PowerShell escapes single quotes by doubling them
        assert!(script.contains("$env:MESSAGE = 'It''s a test'"));
    }

    #[cfg(not(windows))]
    {
        // Bash escapes single quotes with '\''
        assert!(script.contains("export MESSAGE='It'\\''s a test'"));
    }

    cleanup_test_env();
}

/// Test script generation with complex PATH containing multiple entries
#[rstest]
#[test]
fn test_generate_script_complex_path() {
    let mut env_vars: HashMap<String, String> = HashMap::new();

    #[cfg(windows)]
    {
        env_vars.insert(
            "PATH".to_string(),
            r"C:\Users\test\.vx\store\node\22.0.0;C:\Users\test\.vx\store\uv\0.7.12;C:\Windows\System32;C:\Program Files\Git\bin".to_string(),
        );
    }

    #[cfg(not(windows))]
    {
        env_vars.insert(
            "PATH".to_string(),
            "/home/test/.vx/store/node/22.0.0:/home/test/.vx/store/uv/0.7.12:/usr/local/bin:/usr/bin".to_string(),
        );
    }

    let cmd = "node --version";

    let script = generate_wrapper_script(cmd, &env_vars);

    // Verify the PATH is included
    assert!(script.contains("PATH"));

    #[cfg(windows)]
    {
        assert!(script.contains("node\\22.0.0"));
        assert!(script.contains("uv\\0.7.12"));
    }

    #[cfg(not(windows))]
    {
        assert!(script.contains("node/22.0.0"));
        assert!(script.contains("uv/0.7.12"));
    }

    cleanup_test_env();
}

/// Test script generation with command containing quotes
#[rstest]
#[test]
fn test_generate_script_command_with_quotes() {
    let env_vars: HashMap<String, String> = HashMap::new();
    let cmd = r#"echo "Hello World""#;

    let script = generate_wrapper_script(cmd, &env_vars);

    #[cfg(windows)]
    {
        // PowerShell uses cmd /c with escaped quotes
        assert!(script.contains("cmd /c"));
    }

    #[cfg(not(windows))]
    {
        assert!(script.contains(r#"echo "Hello World""#));
    }

    cleanup_test_env();
}

/// Test script generation with empty command
#[rstest]
#[test]
fn test_generate_script_empty_command() {
    let env_vars: HashMap<String, String> = HashMap::new();
    let cmd = "";

    let script = generate_wrapper_script(cmd, &env_vars);

    // Script should still be valid even with empty command
    #[cfg(windows)]
    {
        assert!(script.contains("$ErrorActionPreference = 'Stop'"));
    }

    #[cfg(not(windows))]
    {
        assert!(script.contains("#!/usr/bin/env bash"));
    }

    cleanup_test_env();
}

/// Test script generation with value containing backslashes (Windows paths)
#[rstest]
#[test]
fn test_generate_script_backslash_in_value() {
    let mut env_vars: HashMap<String, String> = HashMap::new();
    env_vars.insert(
        "PROJECT_DIR".to_string(),
        r"C:\Users\test\projects\my-app".to_string(),
    );

    let cmd = "dir";

    let script = generate_wrapper_script(cmd, &env_vars);

    #[cfg(windows)]
    {
        // Backslashes should be preserved in PowerShell single-quoted strings
        assert!(script.contains(r"C:\Users\test\projects\my-app"));
    }

    cleanup_test_env();
}

/// Test that Windows scripts use CRLF line endings
#[rstest]
#[test]
#[cfg(windows)]
fn test_windows_script_line_endings() {
    let env_vars: HashMap<String, String> = HashMap::new();
    let cmd = "echo test";

    let script = generate_wrapper_script(cmd, &env_vars);

    // Windows scripts should use CRLF
    assert!(script.contains("\r\n"));

    cleanup_test_env();
}

/// Test that Unix scripts use LF line endings
#[rstest]
#[test]
#[cfg(not(windows))]
fn test_unix_script_line_endings() {
    let env_vars: HashMap<String, String> = HashMap::new();
    let cmd = "echo test";

    let script = generate_wrapper_script(cmd, &env_vars);

    // Unix scripts should use LF, not CRLF
    assert!(!script.contains("\r\n"));
    assert!(script.contains('\n'));

    cleanup_test_env();
}

/// Test script generation with percent signs (Windows batch special char)
#[rstest]
#[test]
fn test_generate_script_percent_in_value() {
    let mut env_vars: HashMap<String, String> = HashMap::new();
    env_vars.insert("COVERAGE".to_string(), "100%".to_string());

    let cmd = "echo done";

    let script = generate_wrapper_script(cmd, &env_vars);

    // Percent signs should be handled correctly
    assert!(script.contains("100%"));

    cleanup_test_env();
}

/// Test script generation preserves environment variable order consistency
#[rstest]
#[test]
fn test_generate_script_env_vars_present() {
    let mut env_vars: HashMap<String, String> = HashMap::new();
    env_vars.insert("A".to_string(), "1".to_string());
    env_vars.insert("B".to_string(), "2".to_string());
    env_vars.insert("C".to_string(), "3".to_string());

    let cmd = "test";

    let script = generate_wrapper_script(cmd, &env_vars);

    // All variables should be present (order may vary due to HashMap)
    #[cfg(windows)]
    {
        assert!(script.contains("$env:A = '1'"));
        assert!(script.contains("$env:B = '2'"));
        assert!(script.contains("$env:C = '3'"));
    }

    #[cfg(not(windows))]
    {
        assert!(script.contains("export A='1'"));
        assert!(script.contains("export B='2'"));
        assert!(script.contains("export C='3'"));
    }

    cleanup_test_env();
}
