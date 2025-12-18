//! VSCode provider implementation

use crate::runtime::VscodeRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// VSCode provider that provides Visual Studio Code editor
#[derive(Debug, Default)]
pub struct VscodeProvider;

impl VscodeProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for VscodeProvider {
    fn name(&self) -> &str {
        "vscode"
    }

    fn description(&self) -> &str {
        "Visual Studio Code - Free, built on open source, runs everywhere"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(VscodeRuntime::new())]
    }
}
