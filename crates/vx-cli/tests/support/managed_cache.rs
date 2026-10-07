//! Isolated real-filesystem context for managed installation completeness tests.

use std::sync::Arc;

use vx_runtime::{RealFileSystem, RealPathProvider, RuntimeContext, testing::mock_context};

pub fn managed_cache_context() -> (tempfile::TempDir, RuntimeContext) {
    let directory = tempfile::tempdir().unwrap();
    let mut ctx = mock_context();
    ctx.paths = Arc::new(RealPathProvider::with_base_dir(directory.path()));
    ctx.fs = Arc::new(RealFileSystem::new());
    (directory, ctx)
}
