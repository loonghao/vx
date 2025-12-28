//! Script command parsing to detect tool dependencies

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

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
}

impl ScriptTool {
    /// Create a new script tool
    pub fn new(name: impl Into<String>, invocation: ToolInvocation) -> Self {
        Self {
            name: name.into(),
            invocation,
            is_available: false,
            args: Vec::new(),
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
}

/// Script parser for detecting tool dependencies
pub struct ScriptParser {
    /// Patterns for different invocation methods
    patterns: ScriptPatterns,
}

struct ScriptPatterns {
    uv_run: Regex,
    uvx: Regex,
    npx: Regex,
    python_m: Regex,
    pnpm_exec: Regex,
    yarn_exec: Regex,
    bunx: Regex,
}

// Pre-compiled regex patterns
static PATTERNS: LazyLock<ScriptPatterns> = LazyLock::new(|| ScriptPatterns {
    // uv run <tool> [args...]
    uv_run: Regex::new(r"uv\s+run\s+([a-zA-Z0-9_-]+)(?:\s+(.*))?").unwrap(),
    // uvx <tool> [args...]
    uvx: Regex::new(r"uvx\s+([a-zA-Z0-9_-]+)(?:\s+(.*))?").unwrap(),
    // npx <tool> [args...]
    npx: Regex::new(r"npx\s+(?:--yes\s+)?([a-zA-Z0-9@/_-]+)(?:\s+(.*))?").unwrap(),
    // python -m <module> [args...]
    python_m: Regex::new(r"python(?:3)?\s+-m\s+([a-zA-Z0-9_]+)(?:\s+(.*))?").unwrap(),
    // pnpm exec <tool> [args...]
    pnpm_exec: Regex::new(r"pnpm\s+(?:exec\s+)?([a-zA-Z0-9_-]+)(?:\s+(.*))?").unwrap(),
    // yarn [exec] <tool> [args...]
    yarn_exec: Regex::new(r"yarn\s+(?:exec\s+)?([a-zA-Z0-9_-]+)(?:\s+(.*))?").unwrap(),
    // bunx <tool> [args...]
    bunx: Regex::new(r"bunx?\s+([a-zA-Z0-9@/_-]+)(?:\s+(.*))?").unwrap(),
});

impl Default for ScriptParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptParser {
    /// Create a new script parser
    pub fn new() -> Self {
        Self {
            patterns: ScriptPatterns {
                uv_run: PATTERNS.uv_run.clone(),
                uvx: PATTERNS.uvx.clone(),
                npx: PATTERNS.npx.clone(),
                python_m: PATTERNS.python_m.clone(),
                pnpm_exec: PATTERNS.pnpm_exec.clone(),
                yarn_exec: PATTERNS.yarn_exec.clone(),
                bunx: PATTERNS.bunx.clone(),
            },
        }
    }

    /// Parse a script command and extract tool dependencies
    pub fn parse(&self, command: &str) -> Vec<ScriptTool> {
        let mut tools = Vec::new();

        // Split by common command separators
        let parts = self.split_commands(command);

        for part in parts {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            // Try each pattern
            if let Some(tool) = self.try_uv_run(part) {
                tools.push(tool);
            } else if let Some(tool) = self.try_uvx(part) {
                tools.push(tool);
            } else if let Some(tool) = self.try_npx(part) {
                tools.push(tool);
            } else if let Some(tool) = self.try_python_m(part) {
                tools.push(tool);
            } else if let Some(tool) = self.try_pnpm_exec(part) {
                tools.push(tool);
            } else if let Some(tool) = self.try_yarn_exec(part) {
                tools.push(tool);
            } else if let Some(tool) = self.try_bunx(part) {
                tools.push(tool);
            }
            // Note: Direct invocations are not detected here as they require
            // more context about what tools are expected
        }

        tools
    }

    /// Split command by common separators (&&, ||, ;, |)
    fn split_commands<'a>(&self, command: &'a str) -> Vec<&'a str> {
        // Simple split - could be improved to handle quotes
        let mut parts = Vec::new();
        let mut current = command;

        while !current.is_empty() {
            // Find next separator
            let next_sep = current
                .find("&&")
                .map(|i| (i, 2))
                .into_iter()
                .chain(current.find("||").map(|i| (i, 2)))
                .chain(current.find(';').map(|i| (i, 1)))
                .min_by_key(|(i, _)| *i);

            if let Some((idx, len)) = next_sep {
                let part = &current[..idx];
                if !part.trim().is_empty() {
                    parts.push(part.trim());
                }
                current = &current[idx + len..];
            } else {
                if !current.trim().is_empty() {
                    parts.push(current.trim());
                }
                break;
            }
        }

        parts
    }

    fn try_uv_run(&self, command: &str) -> Option<ScriptTool> {
        self.patterns.uv_run.captures(command).map(|caps| {
            let name = caps.get(1).unwrap().as_str().to_string();
            let args = caps
                .get(2)
                .map(|m| self.parse_args(m.as_str()))
                .unwrap_or_default();
            ScriptTool::new(name, ToolInvocation::UvRun).with_args(args)
        })
    }

    fn try_uvx(&self, command: &str) -> Option<ScriptTool> {
        self.patterns.uvx.captures(command).map(|caps| {
            let name = caps.get(1).unwrap().as_str().to_string();
            let args = caps
                .get(2)
                .map(|m| self.parse_args(m.as_str()))
                .unwrap_or_default();
            ScriptTool::new(name, ToolInvocation::Uvx).with_args(args)
        })
    }

    fn try_npx(&self, command: &str) -> Option<ScriptTool> {
        self.patterns.npx.captures(command).map(|caps| {
            let name = caps.get(1).unwrap().as_str().to_string();
            let args = caps
                .get(2)
                .map(|m| self.parse_args(m.as_str()))
                .unwrap_or_default();
            ScriptTool::new(name, ToolInvocation::Npx).with_args(args)
        })
    }

    fn try_python_m(&self, command: &str) -> Option<ScriptTool> {
        self.patterns.python_m.captures(command).map(|caps| {
            let name = caps.get(1).unwrap().as_str().to_string();
            let args = caps
                .get(2)
                .map(|m| self.parse_args(m.as_str()))
                .unwrap_or_default();
            ScriptTool::new(name, ToolInvocation::PythonModule).with_args(args)
        })
    }

    fn try_pnpm_exec(&self, command: &str) -> Option<ScriptTool> {
        // Skip if it's a pnpm built-in command
        let builtins = [
            "install", "add", "remove", "update", "run", "exec", "init", "publish", "pack",
        ];

        self.patterns.pnpm_exec.captures(command).and_then(|caps| {
            let name = caps.get(1).unwrap().as_str();
            if builtins.contains(&name) {
                return None;
            }
            let args = caps
                .get(2)
                .map(|m| self.parse_args(m.as_str()))
                .unwrap_or_default();
            Some(ScriptTool::new(name.to_string(), ToolInvocation::PnpmExec).with_args(args))
        })
    }

    fn try_yarn_exec(&self, command: &str) -> Option<ScriptTool> {
        // Skip if it's a yarn built-in command
        let builtins = [
            "install", "add", "remove", "upgrade", "run", "exec", "init", "publish", "pack",
        ];

        self.patterns.yarn_exec.captures(command).and_then(|caps| {
            let name = caps.get(1).unwrap().as_str();
            if builtins.contains(&name) {
                return None;
            }
            let args = caps
                .get(2)
                .map(|m| self.parse_args(m.as_str()))
                .unwrap_or_default();
            Some(ScriptTool::new(name.to_string(), ToolInvocation::YarnExec).with_args(args))
        })
    }

    fn try_bunx(&self, command: &str) -> Option<ScriptTool> {
        self.patterns.bunx.captures(command).map(|caps| {
            let name = caps.get(1).unwrap().as_str().to_string();
            let args = caps
                .get(2)
                .map(|m| self.parse_args(m.as_str()))
                .unwrap_or_default();
            ScriptTool::new(name, ToolInvocation::Bunx).with_args(args)
        })
    }

    fn parse_args(&self, args_str: &str) -> Vec<String> {
        // Simple argument parsing - split by whitespace
        // Could be improved to handle quotes
        args_str.split_whitespace().map(|s| s.to_string()).collect()
    }
}
