//! Task provider implementation
//!
//! Provides the Task (go-task) runner.

use crate::runtime::TaskRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Task provider
#[derive(Debug, Default)]
pub struct TaskProvider;

impl TaskProvider {
    /// Create a new Task provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for TaskProvider {
    fn name(&self) -> &str {
        "task"
    }

    fn description(&self) -> &str {
        "Task - A task runner / simpler Make alternative written in Go"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(TaskRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "task" || name == "go-task"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "task" || name == "go-task" {
            Some(Arc::new(TaskRuntime::new()))
        } else {
            None
        }
    }
}
