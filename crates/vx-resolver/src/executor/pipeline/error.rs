//! Structured error types for the execution pipeline (RFC 0029)
//!
//! Each pipeline stage has its own error type, and `PipelineError` aggregates them.
//! This replaces the blanket `anyhow::Error` usage with actionable, context-rich errors.

use std::path::PathBuf;
use thiserror::Error;

/// Error from the Resolve stage
#[derive(Error, Debug)]
pub enum ResolveError {
    #[error("runtime not found: {name}")]
    RuntimeNotFound { name: String },

    #[error("version not found: {runtime}@{version}")]
    VersionNotFound { runtime: String, version: String },

    #[error("no locked version for '{runtime}', run 'vx lock' to create a lockfile")]
    NoLockedVersion { runtime: String },

    #[error("dependency cycle detected: {}", cycle.join(" -> "))]
    DependencyCycle { cycle: Vec<String> },

    #[error("platform not supported: {runtime} requires {required}, current: {current}")]
    PlatformNotSupported {
        runtime: String,
        required: String,
        current: String,
    },

    #[error("failed to resolve version for {runtime}: {reason}")]
    ResolutionFailed { runtime: String, reason: String },

    #[error("--with dependency '{runtime}' is not a known runtime. Available: {available}")]
    UnknownWithDependency { runtime: String, available: String },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Error from the Ensure (installation) stage
#[derive(Error, Debug)]
pub enum EnsureError {
    #[error("failed to install {runtime}@{version}: {reason}")]
    InstallFailed {
        runtime: String,
        version: String,
        reason: String,
    },

    #[error("dependency {dep} required by {runtime} failed to install: {reason}")]
    DependencyInstallFailed {
        runtime: String,
        dep: String,
        reason: String,
    },

    #[error("download failed for {runtime}@{version} from {url}: {reason}")]
    DownloadFailed {
        runtime: String,
        version: String,
        url: String,
        reason: String,
    },

    #[error("auto-install is disabled, {runtime}@{version} is not installed.\n\nTo install it, run:\n\n  vx install {runtime}@{version}\n\nOr enable auto-install.")]
    AutoInstallDisabled { runtime: String, version: String },

    #[error("installation timeout for {runtime}@{version} after {seconds}s")]
    Timeout {
        runtime: String,
        version: String,
        seconds: u64,
    },

    #[error("platform not supported for {runtime}: {reason}")]
    PlatformNotSupported { runtime: String, reason: String },

    #[error("installation completed but executable not found at {path}")]
    PostInstallVerificationFailed { runtime: String, path: PathBuf },

    #[error("no versions found for {runtime}")]
    NoVersionsFound { runtime: String },

    #[error("installation command failed with exit code: {exit_code:?}")]
    CommandFailed { exit_code: Option<i32> },

    #[error("{runtime} is not installed. {hint}")]
    NotInstalled { runtime: String, hint: String },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Error from the Prepare stage
#[derive(Error, Debug)]
pub enum PrepareError {
    #[error("Unknown runtime '{runtime}'. Cannot auto-install.")]
    UnknownRuntime { runtime: String },

    #[error("no executable found for {runtime} after installation")]
    NoExecutable { runtime: String },

    #[error("executable not found at path: {path}")]
    ExecutableNotFound { path: PathBuf },

    #[error("failed to prepare environment for {runtime}: {reason}")]
    EnvironmentFailed { runtime: String, reason: String },

    #[error("proxy runtime {proxy} for {runtime} is not available: {reason}")]
    ProxyNotAvailable {
        runtime: String,
        proxy: String,
        reason: String,
    },

    #[error("'{runtime}' requires '{dependency}' which is not installed.\n\nTo install it, run:\n\n  vx install {dependency}\n\nOr enable auto-install to install it automatically.\n\nOriginal error: {reason}")]
    DependencyRequired {
        runtime: String,
        dependency: String,
        reason: String,
    },

    #[error("failed to prepare '{runtime}' after installing '{dependency}': {reason}")]
    ProxyRetryFailed {
        runtime: String,
        dependency: String,
        reason: String,
    },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Error from the Execute stage
#[derive(Error, Debug)]
pub enum ExecuteError {
    #[error("failed to spawn {executable}: {reason}")]
    SpawnFailed { executable: PathBuf, reason: String },

    #[error("execution timed out after {seconds}s")]
    Timeout { seconds: u64 },

    #[error("process was killed by signal")]
    Killed,

    #[error("failed to execute bundled '{tool}': {reason}")]
    BundleExecutionFailed { tool: String, reason: String },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Top-level pipeline error that wraps stage-specific errors
#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("resolve: {0}")]
    Resolve(#[source] ResolveError),

    #[error("install: {0}")]
    Ensure(#[source] EnsureError),

    #[error("prepare: {0}")]
    Prepare(#[source] PrepareError),

    #[error("execute: {0}")]
    Execute(#[source] ExecuteError),

    #[error("platform not supported:\n{}", reasons.join("\n  - "))]
    PlatformUnsupported { reasons: Vec<String> },

    #[error("incompatible dependencies: {details}")]
    IncompatibleDependencies { details: String },

    #[error("platform check failed for {runtime}: {reason}")]
    PlatformCheckFailed { runtime: String, reason: String },

    #[error("offline: {0}")]
    Offline(String),
}

impl From<ResolveError> for PipelineError {
    fn from(e: ResolveError) -> Self {
        PipelineError::Resolve(e)
    }
}

impl From<EnsureError> for PipelineError {
    fn from(e: EnsureError) -> Self {
        PipelineError::Ensure(e)
    }
}

impl From<PrepareError> for PipelineError {
    fn from(e: PrepareError) -> Self {
        PipelineError::Prepare(e)
    }
}

impl From<ExecuteError> for PipelineError {
    fn from(e: ExecuteError) -> Self {
        PipelineError::Execute(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_error_display() {
        let err = ResolveError::RuntimeNotFound {
            name: "unknown-tool".to_string(),
        };
        assert_eq!(err.to_string(), "runtime not found: unknown-tool");
    }

    #[test]
    fn test_resolve_error_version_not_found() {
        let err = ResolveError::VersionNotFound {
            runtime: "node".to_string(),
            version: "99.0.0".to_string(),
        };
        assert_eq!(err.to_string(), "version not found: node@99.0.0");
    }

    #[test]
    fn test_ensure_error_display() {
        let err = EnsureError::InstallFailed {
            runtime: "node".to_string(),
            version: "20.0.0".to_string(),
            reason: "network timeout".to_string(),
        };
        assert!(err.to_string().contains("node@20.0.0"));
        assert!(err.to_string().contains("network timeout"));
    }

    #[test]
    fn test_ensure_error_auto_install_disabled() {
        let err = EnsureError::AutoInstallDisabled {
            runtime: "go".to_string(),
            version: "1.21.0".to_string(),
        };
        assert!(err.to_string().contains("auto-install is disabled"));
    }

    #[test]
    fn test_prepare_error_display() {
        let err = PrepareError::NoExecutable {
            runtime: "node".to_string(),
        };
        assert!(err.to_string().contains("no executable"));
    }

    #[test]
    fn test_execute_error_display() {
        let err = ExecuteError::SpawnFailed {
            executable: PathBuf::from("/usr/local/bin/node"),
            reason: "permission denied".to_string(),
        };
        assert!(err.to_string().contains("permission denied"));
    }

    #[test]
    fn test_pipeline_error_from_resolve() {
        let resolve_err = ResolveError::RuntimeNotFound {
            name: "test".to_string(),
        };
        let pipeline_err: PipelineError = resolve_err.into();
        assert!(matches!(pipeline_err, PipelineError::Resolve(_)));
        assert!(pipeline_err.to_string().contains("resolve"));
    }

    #[test]
    fn test_pipeline_error_from_ensure() {
        let ensure_err = EnsureError::AutoInstallDisabled {
            runtime: "node".to_string(),
            version: "20.0.0".to_string(),
        };
        let pipeline_err: PipelineError = ensure_err.into();
        assert!(matches!(pipeline_err, PipelineError::Ensure(_)));
    }

    #[test]
    fn test_pipeline_error_platform_unsupported() {
        let err = PipelineError::PlatformUnsupported {
            reasons: vec![
                "msvc: Windows only".to_string(),
                "xcodebuild: macOS only".to_string(),
            ],
        };
        let msg = err.to_string();
        assert!(msg.contains("msvc"));
        assert!(msg.contains("xcodebuild"));
    }

    #[test]
    fn test_resolve_error_unknown_with_dependency() {
        let err = ResolveError::UnknownWithDependency {
            runtime: "foo".to_string(),
            available: "node, go, uv".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("--with dependency"));
        assert!(msg.contains("foo"));
        assert!(msg.contains("node, go, uv"));
    }

    #[test]
    fn test_ensure_error_platform_not_supported() {
        let err = EnsureError::PlatformNotSupported {
            runtime: "msvc".to_string(),
            reason: "Windows only".to_string(),
        };
        assert!(err.to_string().contains("msvc"));
        assert!(err.to_string().contains("Windows only"));
    }

    #[test]
    fn test_ensure_error_post_install_verification() {
        let err = EnsureError::PostInstallVerificationFailed {
            runtime: "node".to_string(),
            path: PathBuf::from("/usr/local/bin/node"),
        };
        assert!(err.to_string().contains("executable not found"));
        assert!(err.to_string().contains("/usr/local/bin/node"));
    }

    #[test]
    fn test_ensure_error_no_versions_found() {
        let err = EnsureError::NoVersionsFound {
            runtime: "unknown-tool".to_string(),
        };
        assert!(err.to_string().contains("no versions found"));
        assert!(err.to_string().contains("unknown-tool"));
    }

    #[test]
    fn test_ensure_error_command_failed() {
        let err = EnsureError::CommandFailed { exit_code: Some(1) };
        assert!(err.to_string().contains("failed"));
    }

    #[test]
    fn test_ensure_error_not_installed() {
        let err = EnsureError::NotInstalled {
            runtime: "Go".to_string(),
            hint: "Please install from https://go.dev/dl/".to_string(),
        };
        assert!(err.to_string().contains("Go"));
        assert!(err.to_string().contains("https://go.dev/dl/"));
    }

    #[test]
    fn test_prepare_error_dependency_required() {
        let err = PrepareError::DependencyRequired {
            runtime: "npm".to_string(),
            dependency: "node".to_string(),
            reason: "not found".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("npm"));
        assert!(msg.contains("node"));
        assert!(msg.contains("vx install node"));
    }

    #[test]
    fn test_prepare_error_proxy_retry_failed() {
        let err = PrepareError::ProxyRetryFailed {
            runtime: "msbuild".to_string(),
            dependency: "dotnet".to_string(),
            reason: "still not found".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("msbuild"));
        assert!(msg.contains("dotnet"));
    }

    #[test]
    fn test_execute_error_bundle_execution_failed() {
        let err = ExecuteError::BundleExecutionFailed {
            tool: "node".to_string(),
            reason: "file not found".to_string(),
        };
        assert!(err.to_string().contains("bundled"));
        assert!(err.to_string().contains("node"));
    }
}
