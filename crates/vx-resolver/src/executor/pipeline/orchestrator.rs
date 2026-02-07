//! Execution Pipeline Orchestrator (RFC 0029)
//!
//! The `ExecutionPipeline` composes all four stages into a single execution flow:
//!
//! ```text
//! ResolveRequest → ResolveStage → ExecutionPlan
//!                                      ↓
//!                  EnsureStage  → ExecutionPlan (installed)
//!                                      ↓
//!                  PrepareStage → PreparedExecution
//!                                      ↓
//!                  ExecuteStage → i32 (exit code)
//! ```
//!
//! The pipeline is the primary entry point for executing runtimes.
//! In Phase 1, it exists alongside the monolithic `Executor::execute_with_with_deps`.
//! In later phases, the executor will delegate to this pipeline.

use async_trait::async_trait;
use tracing::debug;

use super::error::PipelineError;
use super::stage::Stage;
use super::stages::ensure::EnsureStage;
use super::stages::execute::ExecuteStage;
use super::stages::prepare::PrepareStage;
use super::stages::resolve::{ResolveRequest, ResolveStage};

/// The full execution pipeline: `ResolveRequest` → `i32` (exit code)
///
/// Composes all four stages and handles error wrapping.
/// Each stage's error type is automatically wrapped into `PipelineError`.
pub struct ExecutionPipeline<'a> {
    resolve: ResolveStage<'a>,
    ensure: EnsureStage<'a>,
    prepare: PrepareStage<'a>,
    execute: ExecuteStage,
}

impl<'a> ExecutionPipeline<'a> {
    /// Create a new execution pipeline from individual stages
    pub fn new(
        resolve: ResolveStage<'a>,
        ensure: EnsureStage<'a>,
        prepare: PrepareStage<'a>,
        execute: ExecuteStage,
    ) -> Self {
        Self {
            resolve,
            ensure,
            prepare,
            execute,
        }
    }

    /// Run the full pipeline: resolve → ensure → prepare → execute
    pub async fn run(&self, request: ResolveRequest) -> Result<i32, PipelineError> {
        debug!("[Pipeline] Starting: {}", request.runtime_name);

        // Stage 1: Resolve
        let plan = self.resolve.execute(request).await?;
        debug!(
            "[Pipeline] Resolved: primary={}, needs_install={}",
            plan.primary.name,
            plan.needs_install()
        );

        // Stage 2: Ensure installed
        let plan = self.ensure.execute(plan).await?;
        debug!(
            "[Pipeline] Ensured: primary executable={:?}",
            plan.primary.executable
        );

        // Stage 3: Prepare environment
        let prepared = self.prepare.execute(plan).await?;
        debug!(
            "[Pipeline] Prepared: executable={}, args={:?}",
            prepared.executable.display(),
            prepared.args
        );

        // Stage 4: Execute
        let exit_code = self.execute.execute(prepared).await?;
        debug!("[Pipeline] Complete: exit_code={}", exit_code);

        Ok(exit_code)
    }
}

/// The pipeline itself implements Stage for composability
#[async_trait]
impl<'a> Stage<ResolveRequest, i32> for ExecutionPipeline<'a> {
    type Error = PipelineError;

    async fn execute(&self, input: ResolveRequest) -> Result<i32, PipelineError> {
        self.run(input).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Resolver, ResolverConfig, RuntimeMap};

    fn test_resolver() -> Resolver {
        Resolver::new(ResolverConfig::default(), RuntimeMap::empty()).unwrap()
    }

    #[test]
    fn test_pipeline_construction() {
        let resolver = test_resolver();
        let config = ResolverConfig::default();

        let resolve = ResolveStage::new(&resolver, &config);
        let ensure = EnsureStage::new(&resolver, &config, None, None);
        let prepare = PrepareStage::new(&resolver, &config, None, None);
        let execute = ExecuteStage::new();

        let _pipeline = ExecutionPipeline::new(resolve, ensure, prepare, execute);
        // Pipeline was constructed successfully
    }

    #[tokio::test]
    async fn test_pipeline_resolve_and_ensure_no_registry() {
        // Without a registry, the pipeline can still resolve and attempt ensure
        let resolver = test_resolver();
        let config = ResolverConfig::default();

        let resolve = ResolveStage::new(&resolver, &config);
        let ensure = EnsureStage::new(&resolver, &config, None, None);
        let prepare = PrepareStage::new(&resolver, &config, None, None);
        let execute = ExecuteStage::new();

        let pipeline = ExecutionPipeline::new(resolve, ensure, prepare, execute);

        let request = ResolveRequest::new("node", vec!["--version".into()]).with_version("20.0.0");

        // This will fail at the Prepare stage because no executable is available
        // without a real registry — which is expected in a unit test
        let result = pipeline.run(request).await;

        // We expect a PrepareError::NoExecutable because without registry
        // the runtime can't be installed
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, PipelineError::Prepare(_)),
            "Expected PrepareError, got: {:?}",
            err
        );
    }
}
