//! Pipeline Stage trait (RFC 0029)
//!
//! The `Stage` trait defines the contract for each step in the execution pipeline.
//! Stages are composable, testable, and each handles a single responsibility.

use async_trait::async_trait;

/// A pipeline stage that transforms an input into an output.
///
/// Each stage in the execution pipeline implements this trait:
/// - `ResolveStage`: `ResolveRequest` → `ExecutionPlan`
/// - `EnsureStage`: `ExecutionPlan` → `ExecutionPlan` (with installations done)
/// - `PrepareStage`: `ExecutionPlan` → `PreparedExecution`
/// - `ExecuteStage`: `PreparedExecution` → `i32` (exit code)
///
/// The generic error type allows each stage to define its own structured errors
/// while the pipeline wraps them in `PipelineError`.
#[async_trait]
pub trait Stage<Input, Output>: Send + Sync
where
    Input: Send + 'static,
    Output: Send + 'static,
{
    /// The error type for this stage
    type Error: std::error::Error + Send + Sync + 'static;

    /// Execute this stage, transforming input to output.
    ///
    /// # Arguments
    /// * `input` - The input data from the previous stage
    ///
    /// # Returns
    /// The output data for the next stage, or an error
    async fn execute(&self, input: Input) -> Result<Output, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    // Test that the Stage trait can be implemented
    #[derive(Debug)]
    struct TestError(String);

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for TestError {}

    struct DoubleStage;

    #[async_trait]
    impl Stage<i32, i32> for DoubleStage {
        type Error = TestError;

        async fn execute(&self, input: i32) -> Result<i32, Self::Error> {
            if input < 0 {
                Err(TestError("negative input".to_string()))
            } else {
                Ok(input * 2)
            }
        }
    }

    #[tokio::test]
    async fn test_stage_success() {
        let stage = DoubleStage;
        let result = stage.execute(5).await;
        assert_eq!(result.unwrap(), 10);
    }

    #[tokio::test]
    async fn test_stage_error() {
        let stage = DoubleStage;
        let result = stage.execute(-1).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "negative input");
    }

    struct StringifyStage;

    #[async_trait]
    impl Stage<i32, String> for StringifyStage {
        type Error = TestError;

        async fn execute(&self, input: i32) -> Result<String, Self::Error> {
            Ok(format!("value: {}", input))
        }
    }

    #[tokio::test]
    async fn test_stage_type_transform() {
        let stage = StringifyStage;
        let result = stage.execute(42).await.unwrap();
        assert_eq!(result, "value: 42");
    }
}
