//! Skills provider implementation
//!
//! Provides the Vercel Skills CLI tool for managing AI agent skills.

use crate::runtime::SkillsRuntime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// Skills provider
#[derive(Debug, Default)]
pub struct SkillsProvider;

impl SkillsProvider {
    /// Create a new Skills provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for SkillsProvider {
    fn name(&self) -> &str {
        "skills"
    }

    fn description(&self) -> &str {
        "Vercel Skills - The open agent skills tool"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(SkillsRuntime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "skills"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "skills" {
            Some(Arc::new(SkillsRuntime::new()))
        } else {
            None
        }
    }
}
