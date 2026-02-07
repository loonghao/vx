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

    #[error("auto-install is disabled, {runtime}@{version} is not installed")]
    AutoInstallDisabled { runtime: String, version: String },

    #[error("installation timeout for {runtime}@{version} after {seconds}s")]
    Timeout {
        runtime: String,
        version: String,
        seconds: u64,
    },

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

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Error from the Execute stage
#[derive(Error, Debug)]
pub enum ExecuteError {
    #[error("failed to spawn {executable}: {reason}")]
    SpawnFailed {
        executable: PathBuf,
        reason: String,
    },

    #[error("execution timed out after {seconds}s")]
    Timeout { seconds: u64 },

    #[error("process was killed by signal")]
    Killed,

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
}
