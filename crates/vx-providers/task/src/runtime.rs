//! Task runtime implementation
//!
//! Task is a task runner / simpler Make alternative written in Go.
//! https://github.com/go-task/task

use crate::config::TaskUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo};
use vx_version_fetcher::VersionFetcherBuilder;

/// Task runtime implementation
#[derive(Debug, Clone, Default)]
pub struct TaskRuntime;

impl TaskRuntime {
    /// Create a new Task runtime
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for TaskRuntime {
    fn name(&self) -> &str {
        "task"
    }

    fn description(&self) -> &str {
        "Task - A task runner / simpler Make alternative written in Go"
    }

    fn aliases(&self) -> &[&str] {
        &["go-task"]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://taskfile.dev/".to_string());
        meta.insert(
            "documentation".to_string(),
            "https://taskfile.dev/usage/".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/go-task/task".to_string(),
        );
        meta.insert("category".to_string(), "task-runner".to_string());
        meta
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // Task extracts directly without subdirectory
        TaskUrlBuilder::get_executable_name(platform).to_string()
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Task uses 'v' prefix in tags, strip it for version display
        VersionFetcherBuilder::jsdelivr("go-task", "task")
            .tool_name("task")
            .strip_v_prefix()
            .skip_prereleases()
            .limit(50)
            .build()
            .fetch(ctx)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(TaskUrlBuilder::download_url(version, platform))
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_name = TaskUrlBuilder::get_executable_name(platform);
        let exe_path = install_path.join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "Task executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling the runtime".to_string()],
            )
        }
    }
}
