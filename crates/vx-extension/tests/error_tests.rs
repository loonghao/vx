//! Tests for extension error types and diagnostics

use rstest::rstest;
use std::path::PathBuf;
use vx_extension::{ExtensionError, ExtensionResult};

// ============ Error Construction Tests ============

#[test]
fn test_config_not_found_error() {
    let err = ExtensionError::config_not_found("/path/to/extension");

    assert!(matches!(err, ExtensionError::ConfigNotFound { .. }));

    let diag = err.diagnostic();
    assert!(diag.contains("vx-extension.toml"));
    assert!(diag.contains("/path/to/extension"));
}

#[test]
fn test_extension_not_found_error() {
    let err = ExtensionError::extension_not_found(
        "my-extension",
        vec!["ext1".to_string(), "ext2".to_string(), "ext3".to_string()],
        vec![
            PathBuf::from("/home/user/.vx/extensions"),
            PathBuf::from("/project/.vx/extensions"),
        ],
    );

    assert!(matches!(err, ExtensionError::ExtensionNotFound { .. }));

    let diag = err.diagnostic();
    assert!(diag.contains("my-extension"));
    assert!(diag.contains("ext1"));
    assert!(diag.contains("ext2"));
    assert!(diag.contains("ext3"));
    assert!(diag.contains("vx ext install"));
}

#[test]
fn test_subcommand_not_found_error() {
    let err = ExtensionError::subcommand_not_found(
        "docker-compose",
        "invalid-cmd",
        vec!["up".to_string(), "down".to_string(), "logs".to_string()],
    );

    assert!(matches!(err, ExtensionError::SubcommandNotFound { .. }));

    let diag = err.diagnostic();
    assert!(diag.contains("invalid-cmd"));
    assert!(diag.contains("docker-compose"));
    assert!(diag.contains("up"));
    assert!(diag.contains("down"));
    assert!(diag.contains("logs"));
}

#[test]
fn test_no_entrypoint_error() {
    let err = ExtensionError::no_entrypoint("my-ext", vec!["cmd1".to_string(), "cmd2".to_string()]);

    assert!(matches!(err, ExtensionError::NoEntrypoint { .. }));

    let diag = err.diagnostic();
    assert!(diag.contains("my-ext"));
    assert!(diag.contains("cmd1"));
    assert!(diag.contains("cmd2"));
}

#[test]
fn test_no_entrypoint_error_empty_commands() {
    let err = ExtensionError::no_entrypoint("my-ext", vec![]);

    let diag = err.diagnostic();
    assert!(diag.contains("my-ext"));
    assert!(diag.contains("[entrypoint]"));
    assert!(diag.contains("main = "));
}

#[test]
fn test_script_not_found_error() {
    let err = ExtensionError::script_not_found(
        "my-ext",
        "scripts/run.py",
        "/home/user/.vx/extensions/my-ext",
    );

    assert!(matches!(err, ExtensionError::ScriptNotFound { .. }));

    let diag = err.diagnostic();
    assert!(diag.contains("run.py"));
    assert!(diag.contains("my-ext"));
}

#[test]
fn test_runtime_not_available_error() {
    let err =
        ExtensionError::runtime_not_available("my-ext", "python", Some(">= 3.10".to_string()));

    assert!(matches!(err, ExtensionError::RuntimeNotAvailable { .. }));

    let diag = err.diagnostic();
    assert!(diag.contains("python"));
    assert!(diag.contains(">= 3.10"));
    assert!(diag.contains("vx install"));
}

#[test]
fn test_runtime_not_available_no_constraint() {
    let err = ExtensionError::runtime_not_available("my-ext", "node", None);

    let diag = err.diagnostic();
    assert!(diag.contains("node"));
    assert!(diag.contains("vx install node"));
}

#[test]
fn test_link_failed_error() {
    let err =
        ExtensionError::link_failed("/path/to/source", "/path/to/target", "Permission denied");

    assert!(matches!(err, ExtensionError::LinkFailed { .. }));

    let diag = err.diagnostic();
    assert!(diag.contains("/path/to/source"));
    assert!(diag.contains("/path/to/target"));
    assert!(diag.contains("Permission denied"));
}

#[test]
fn test_not_a_dev_link_error() {
    let err = ExtensionError::NotADevLink {
        name: "my-ext".to_string(),
        path: PathBuf::from("/home/user/.vx/extensions/my-ext"),
    };

    let diag = err.diagnostic();
    assert!(diag.contains("my-ext"));
    assert!(diag.contains("not a development link"));
    assert!(diag.contains("vx ext dev"));
}

#[test]
fn test_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err = ExtensionError::io(
        "Failed to read file",
        Some(PathBuf::from("/path/to/file")),
        io_err,
    );

    assert!(matches!(err, ExtensionError::Io { .. }));

    let diag = err.diagnostic();
    assert!(diag.contains("Failed to read file"));
    assert!(diag.contains("/path/to/file"));
}

#[test]
fn test_io_error_no_path() {
    let io_err = std::io::Error::new(std::io::ErrorKind::Other, "unknown error");
    let err = ExtensionError::io("Something went wrong", None, io_err);

    let diag = err.diagnostic();
    assert!(diag.contains("Something went wrong"));
}

// ============ Exit Code Tests ============

#[rstest]
#[case(ExtensionError::config_not_found("/path"), 64)]
#[case(ExtensionError::extension_not_found("test", vec![], vec![]), 66)]
#[case(ExtensionError::subcommand_not_found("ext", "cmd", vec![]), 64)]
#[case(ExtensionError::no_entrypoint("ext", vec![]), 78)]
#[case(ExtensionError::script_not_found("ext", "script.py", "/path"), 66)]
#[case(ExtensionError::runtime_not_available("ext", "python", None), 69)]
fn test_exit_codes(#[case] err: ExtensionError, #[case] expected_code: i32) {
    assert_eq!(err.exit_code(), expected_code);
}

#[test]
fn test_execution_failed_exit_code() {
    let err = ExtensionError::ExecutionFailed {
        extension: "test".to_string(),
        exit_code: Some(42),
        stderr: None,
    };
    assert_eq!(err.exit_code(), 42);

    let err_no_code = ExtensionError::ExecutionFailed {
        extension: "test".to_string(),
        exit_code: None,
        stderr: None,
    };
    assert_eq!(err_no_code.exit_code(), 1);
}

// ============ Recoverable Tests ============

#[test]
fn test_is_recoverable() {
    // Recoverable errors
    assert!(ExtensionError::extension_not_found("test", vec![], vec![]).is_recoverable());
    assert!(ExtensionError::subcommand_not_found("ext", "cmd", vec![]).is_recoverable());
    assert!(ExtensionError::runtime_not_available("ext", "python", None).is_recoverable());

    // Non-recoverable errors
    assert!(!ExtensionError::config_not_found("/path").is_recoverable());
    assert!(!ExtensionError::no_entrypoint("ext", vec![]).is_recoverable());
    assert!(!ExtensionError::script_not_found("ext", "script.py", "/path").is_recoverable());
}

// ============ Display Tests ============

#[test]
fn test_error_display() {
    let err = ExtensionError::extension_not_found("my-ext", vec![], vec![]);
    let display = format!("{}", err);
    assert!(display.contains("my-ext"));
    assert!(display.contains("not found"));
}

#[test]
fn test_config_invalid_display() {
    let err = ExtensionError::ConfigInvalid {
        path: PathBuf::from("/path/to/config.toml"),
        reason: "expected `=`".to_string(),
        line: Some(10),
        column: Some(5),
    };

    let display = format!("{}", err);
    assert!(display.contains("Invalid extension configuration"));

    let diag = err.diagnostic();
    assert!(diag.contains("line 10"));
    assert!(diag.contains("expected `=`"));
}

// ============ Result Type Tests ============

#[test]
fn test_extension_result_type() {
    fn returns_ok() -> ExtensionResult<String> {
        Ok("success".to_string())
    }

    fn returns_err() -> ExtensionResult<String> {
        Err(ExtensionError::extension_not_found("test", vec![], vec![]))
    }

    assert!(returns_ok().is_ok());
    assert!(returns_err().is_err());
}

// ============ Duplicate Extension Tests ============

#[test]
fn test_duplicate_extension_error() {
    let err = ExtensionError::DuplicateExtension {
        name: "my-ext".to_string(),
        paths: vec![
            PathBuf::from("/home/user/.vx/extensions-dev/my-ext"),
            PathBuf::from("/home/user/.vx/extensions/my-ext"),
        ],
    };

    let diag = err.diagnostic();
    assert!(diag.contains("my-ext"));
    assert!(diag.contains("extensions-dev"));
    assert!(diag.contains("Priority order"));
}

// ============ Permission Denied Tests ============

#[test]
fn test_permission_denied_error() {
    let err = ExtensionError::PermissionDenied {
        message: "Cannot write to extensions directory".to_string(),
        path: PathBuf::from("/root/.vx/extensions"),
    };

    let diag = err.diagnostic();
    assert!(diag.contains("Permission denied"));
    assert!(diag.contains("Cannot write"));
    assert!(diag.contains("/root/.vx/extensions"));
}
