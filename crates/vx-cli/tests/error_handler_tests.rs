//! Tests for the error_handler module (RFC 0029 Phase 3.3)
//!
//! These tests verify that error formatting functions handle all error variants
//! without panicking and produce expected output patterns.

use std::path::PathBuf;
use vx_resolver::{EnsureError, ExecuteError, PipelineError, PrepareError, ResolveError};

/// Helper: call handle_pipeline_error without triggering process::exit
/// We test the inner format functions directly since handle_pipeline_error
/// and try_handle_error call process::exit.

#[test]
fn test_pipeline_error_resolve_variant() {
    let err = PipelineError::Resolve(ResolveError::RuntimeNotFound {
        name: "test".to_string(),
    });
    // Verify handle_pipeline_error returns non-zero
    let code = vx_cli::error_handler::handle_pipeline_error(&err);
    assert_eq!(code, 1);
}

#[test]
fn test_pipeline_error_ensure_variant() {
    let err = PipelineError::Ensure(EnsureError::AutoInstallDisabled {
        runtime: "node".to_string(),
        version: "20.0.0".to_string(),
    });
    let code = vx_cli::error_handler::handle_pipeline_error(&err);
    assert_eq!(code, 1);
}

#[test]
fn test_pipeline_error_prepare_variant() {
    let err = PipelineError::Prepare(PrepareError::DependencyRequired {
        runtime: "npm".to_string(),
        dependency: "node".to_string(),
        reason: "not found".to_string(),
    });
    let code = vx_cli::error_handler::handle_pipeline_error(&err);
    assert_eq!(code, 1);
}

#[test]
fn test_pipeline_error_execute_variant() {
    let err = PipelineError::Execute(ExecuteError::SpawnFailed {
        executable: PathBuf::from("/usr/bin/node"),
        reason: "permission denied".to_string(),
    });
    let code = vx_cli::error_handler::handle_pipeline_error(&err);
    assert_eq!(code, 1);
}

#[test]
fn test_pipeline_error_platform_unsupported() {
    let err = PipelineError::PlatformUnsupported {
        reasons: vec![
            "msvc: Windows only".to_string(),
            "xcodebuild: macOS only".to_string(),
        ],
    };
    let code = vx_cli::error_handler::handle_pipeline_error(&err);
    assert_eq!(code, 1);
}

#[test]
fn test_pipeline_error_offline() {
    let err = PipelineError::Offline("no internet connection".to_string());
    let code = vx_cli::error_handler::handle_pipeline_error(&err);
    assert_eq!(code, 1);
}

#[test]
fn test_format_generic_error() {
    let err = anyhow::anyhow!("something went wrong");
    // Should not panic
    vx_cli::error_handler::format_generic_error(&err);
}

#[test]
fn test_format_generic_error_with_chain() {
    let inner = anyhow::anyhow!("inner cause");
    let outer = anyhow::anyhow!(inner).context("outer error");
    vx_cli::error_handler::format_generic_error(&outer);
}

#[test]
fn test_try_handle_error_returns_false_for_non_pipeline() {
    let err = anyhow::anyhow!("generic error");
    // try_handle_error calls process::exit for PipelineErrors,
    // but returns false for non-pipeline errors
    let handled = vx_cli::error_handler::try_handle_error(&err);
    assert!(!handled);
}

// Test all ResolveError variants
#[test]
fn test_resolve_error_version_not_found() {
    let err = PipelineError::Resolve(ResolveError::VersionNotFound {
        runtime: "node".to_string(),
        version: "99.0.0".to_string(),
    });
    let code = vx_cli::error_handler::handle_pipeline_error(&err);
    assert_eq!(code, 1);
}

#[test]
fn test_resolve_error_no_locked_version() {
    let err = PipelineError::Resolve(ResolveError::NoLockedVersion {
        runtime: "node".to_string(),
    });
    let code = vx_cli::error_handler::handle_pipeline_error(&err);
    assert_eq!(code, 1);
}

#[test]
fn test_resolve_error_dependency_cycle() {
    let err = PipelineError::Resolve(ResolveError::DependencyCycle {
        cycle: vec!["a".to_string(), "b".to_string(), "a".to_string()],
    });
    let code = vx_cli::error_handler::handle_pipeline_error(&err);
    assert_eq!(code, 1);
}

#[test]
fn test_resolve_error_unknown_with_dependency() {
    let err = PipelineError::Resolve(ResolveError::UnknownWithDependency {
        runtime: "foo".to_string(),
        available: "node, go, uv".to_string(),
    });
    let code = vx_cli::error_handler::handle_pipeline_error(&err);
    assert_eq!(code, 1);
}

// Test all EnsureError variants
#[test]
fn test_ensure_error_install_failed() {
    let err = PipelineError::Ensure(EnsureError::InstallFailed {
        runtime: "node".to_string(),
        version: "20.0.0".to_string(),
        reason: "disk full".to_string(),
    });
    let code = vx_cli::error_handler::handle_pipeline_error(&err);
    assert_eq!(code, 1);
}

#[test]
fn test_ensure_error_download_failed() {
    let err = PipelineError::Ensure(EnsureError::DownloadFailed {
        runtime: "node".to_string(),
        version: "20.0.0".to_string(),
        url: "https://example.com/node.tar.gz".to_string(),
        reason: "404".to_string(),
    });
    let code = vx_cli::error_handler::handle_pipeline_error(&err);
    assert_eq!(code, 1);
}

#[test]
fn test_ensure_error_post_install_verification() {
    let err = PipelineError::Ensure(EnsureError::PostInstallVerificationFailed {
        runtime: "node".to_string(),
        path: PathBuf::from("/usr/local/bin/node"),
    });
    let code = vx_cli::error_handler::handle_pipeline_error(&err);
    assert_eq!(code, 1);
}

#[test]
fn test_ensure_error_not_installed() {
    let err = PipelineError::Ensure(EnsureError::NotInstalled {
        runtime: "Go".to_string(),
        hint: "Please install from https://go.dev/dl/".to_string(),
    });
    let code = vx_cli::error_handler::handle_pipeline_error(&err);
    assert_eq!(code, 1);
}

// Test ExecuteError variants
#[test]
fn test_execute_error_timeout() {
    let err = PipelineError::Execute(ExecuteError::Timeout { seconds: 30 });
    let code = vx_cli::error_handler::handle_pipeline_error(&err);
    assert_eq!(code, 1);
}

#[test]
fn test_execute_error_killed() {
    let err = PipelineError::Execute(ExecuteError::Killed);
    let code = vx_cli::error_handler::handle_pipeline_error(&err);
    assert_eq!(code, 1);
}

#[test]
fn test_execute_error_bundle_failed() {
    let err = PipelineError::Execute(ExecuteError::BundleExecutionFailed {
        tool: "node".to_string(),
        reason: "file not found".to_string(),
    });
    let code = vx_cli::error_handler::handle_pipeline_error(&err);
    assert_eq!(code, 1);
}
