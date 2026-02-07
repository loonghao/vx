//! Execution Pipeline (RFC 0029)
//!
//! This module implements the Pipeline architecture for executor refactoring.
//! The pipeline decomposes the monolithic `Executor::execute_with_with_deps` into
//! four independent stages connected via an `ExecutionPlan` intermediate representation:
//!
//! ```text
//! ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐
//! │ Resolve  │ → │  Ensure  │ → │ Prepare  │ → │ Execute  │
//! │  Stage   │   │  Stage   │   │  Stage   │   │  Stage   │
//! └──────────┘   └──────────┘   └──────────┘   └──────────┘
//!      │              │              │              │
//!      ▼              ▼              ▼              ▼
//! ExecutionPlan  ExecutionPlan  PreparedExec    ExitCode
//! ```
//!
//! ## Design Goals
//! - **Separation of concerns**: Each stage handles one responsibility
//! - **Testability**: Each stage can be tested independently with mocked inputs
//! - **Observability**: Structured errors with full context at each stage
//! - **Compatibility**: Wraps existing code, does not break current behavior

pub mod error;
pub mod orchestrator;
pub mod plan;
pub mod stage;
pub mod stages;

// Re-export core types
pub use error::{EnsureError, ExecuteError, PipelineError, PrepareError, ResolveError};
pub use orchestrator::ExecutionPipeline;
pub use plan::{
    ExecutionConfig, ExecutionPlan, InstallStatus, PlannedRuntime, ProxyConfig,
    VersionResolution, VersionSource,
};
pub use stage::Stage;
pub use stages::{
    EnsureStage, ExecuteStage, PrepareStage, PreparedExecution, ResolveRequest, ResolveStage,
    WithDepRequest,
};
