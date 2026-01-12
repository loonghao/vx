use serde::{Deserialize, Serialize};

/// Custom command definition
///
/// Allows providers to define additional commands that can be invoked via:
/// `vx <runtime> <command> [args...]`
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommandDef {
    /// Command name (required)
    pub name: String,

    /// Human-readable description
    #[serde(default)]
    pub description: Option<String>,

    /// Command to execute (supports template variables)
    /// Template variables: {executable}, {install_dir}, {version}
    #[serde(default)]
    pub command: Option<String>,

    /// Script file to execute (relative to provider directory)
    /// Alternative to `command` - for complex logic
    #[serde(default)]
    pub script: Option<String>,

    /// Whether to pass user arguments to the command
    #[serde(default)]
    pub pass_args: bool,

    /// Command category for help grouping
    #[serde(default)]
    pub category: Option<String>,

    /// Whether to hide from help output
    #[serde(default)]
    pub hidden: bool,
}

impl CommandDef {
    /// Create a new command definition
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            command: None,
            script: None,
            pass_args: false,
            category: None,
            hidden: false,
        }
    }

    /// Set the command to execute
    pub fn with_command(mut self, cmd: impl Into<String>) -> Self {
        self.command = Some(cmd.into());
        self
    }

    /// Set the description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Enable argument passing
    pub fn with_pass_args(mut self) -> Self {
        self.pass_args = true;
        self
    }

    /// Check if this command is valid (has either command or script)
    pub fn is_valid(&self) -> bool {
        self.command.is_some() || self.script.is_some()
    }
}
