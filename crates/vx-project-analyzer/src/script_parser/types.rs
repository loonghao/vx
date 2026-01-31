//! Core types for script parsing

use serde::{Deserialize, Serialize};

/// Tool invocation method
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolInvocation {
    /// uv run <tool>
    UvRun,
    /// uvx <tool> (temporary installation)
    Uvx,
    /// npx <tool>
    Npx,
    /// python -m <module>
    PythonModule,
    /// Direct command invocation
    Direct,
    /// pnpm exec <tool>
    PnpmExec,
    /// yarn <tool> or yarn exec <tool>
    YarnExec,
    /// bunx <tool>
    Bunx,
}

impl std::fmt::Display for ToolInvocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolInvocation::UvRun => write!(f, "uv run"),
            ToolInvocation::Uvx => write!(f, "uvx"),
            ToolInvocation::Npx => write!(f, "npx"),
            ToolInvocation::PythonModule => write!(f, "python -m"),
            ToolInvocation::Direct => write!(f, "direct"),
            ToolInvocation::PnpmExec => write!(f, "pnpm exec"),
            ToolInvocation::YarnExec => write!(f, "yarn"),
            ToolInvocation::Bunx => write!(f, "bunx"),
        }
    }
}

/// Context for script parsing
pub struct ParseContext<'a> {
    /// Known script names in the project (to filter out internal references)
    pub known_scripts: &'a [&'a str],
}

/// Tool detected in a script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptTool {
    /// Tool name
    pub name: String,

    /// How the tool is invoked
    pub invocation: ToolInvocation,

    /// Whether the tool is available
    pub is_available: bool,

    /// Arguments passed to the tool
    pub args: Vec<String>,

    /// Whether this is likely a project-specific tool/module
    /// (e.g., `python -m myproject` where myproject is the project itself)
    pub is_project_specific: bool,
}

impl ScriptTool {
    /// Create a new script tool
    pub fn new(name: impl Into<String>, invocation: ToolInvocation) -> Self {
        Self {
            name: name.into(),
            invocation,
            is_available: false,
            args: Vec::new(),
            is_project_specific: false,
        }
    }

    /// Set arguments
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Mark as available
    pub fn available(mut self) -> Self {
        self.is_available = true;
        self
    }

    /// Mark as project-specific
    pub fn project_specific(mut self) -> Self {
        self.is_project_specific = true;
        self
    }
}
