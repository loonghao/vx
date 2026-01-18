use std::sync::Arc;
use vx_runtime::Provider;

use crate::runtime::GitHubRuntime;

/// GitHub Provider implementation
pub struct GitHubProvider;

impl GitHubProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for GitHubProvider {
    fn name(&self) -> &str {
        "gh"
    }

    fn description(&self) -> &str {
        "GitHub CLI provider"
    }

    fn runtimes(&self) -> Vec<Arc<dyn vx_runtime::Runtime>> {
        vec![Arc::new(GitHubRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "gh" || name == "github"
    }
}
