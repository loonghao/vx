//! Regression tests for command-prefix handling in manifest-driven runtimes.

use std::sync::Arc;

use vx_runtime::{
    ExecutionContext, ManifestDrivenRuntime, ProviderSource, RealCommandExecutor, Runtime,
};

#[tokio::test]
async fn bundled_runtime_prepare_execution_preserves_command_prefix() {
    let runtime = ManifestDrivenRuntime::new("uvx", "uv", ProviderSource::BuiltIn)
        .with_executable("uv")
        .with_bundled_with("uv")
        .with_command_prefix(vec!["tool".to_string(), "run".to_string()]);

    let prep = runtime
        .prepare_execution(
            "0.11.6",
            &ExecutionContext::new(Arc::new(RealCommandExecutor)),
        )
        .await
        .expect("prepare_execution should succeed");

    assert_eq!(prep.command_prefix, vec!["tool", "run"]);
}
