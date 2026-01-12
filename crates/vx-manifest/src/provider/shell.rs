use serde::{Deserialize, Serialize};

/// Shell integration configuration
///
/// Supports shell prompt customization, completion scripts, and aliases.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ShellConfig {
    /// Prompt format when runtime is activated
    /// Template variables: {name}, {version}
    #[serde(default)]
    pub prompt_format: Option<String>,

    /// Path to activation script template
    #[serde(default)]
    pub activate_template: Option<String>,

    /// Path to deactivation script template
    #[serde(default)]
    pub deactivate_template: Option<String>,

    /// Shell completion scripts
    #[serde(default)]
    pub completions: Option<ShellCompletionsConfig>,

    /// Shell aliases to set when activated
    #[serde(default)]
    pub aliases: std::collections::HashMap<String, String>,
}

impl ShellConfig {
    /// Get the prompt format for a specific version
    pub fn format_prompt(&self, version: &str, name: &str) -> Option<String> {
        self.prompt_format
            .as_ref()
            .map(|fmt| fmt.replace("{version}", version).replace("{name}", name))
    }

    /// Check if there are any shell integrations configured
    pub fn is_empty(&self) -> bool {
        self.prompt_format.is_none()
            && self.activate_template.is_none()
            && self.deactivate_template.is_none()
            && self.completions.is_none()
            && self.aliases.is_empty()
    }
}

/// Shell completion script paths
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ShellCompletionsConfig {
    /// Bash completion script
    #[serde(default)]
    pub bash: Option<String>,

    /// Zsh completion script
    #[serde(default)]
    pub zsh: Option<String>,

    /// Fish completion script
    #[serde(default)]
    pub fish: Option<String>,

    /// PowerShell completion script
    #[serde(default)]
    pub powershell: Option<String>,
}

impl ShellCompletionsConfig {
    /// Get the completion script for a shell type
    pub fn for_shell(&self, shell: &str) -> Option<&str> {
        match shell.to_lowercase().as_str() {
            "bash" => self.bash.as_deref(),
            "zsh" => self.zsh.as_deref(),
            "fish" => self.fish.as_deref(),
            "powershell" | "pwsh" => self.powershell.as_deref(),
            _ => None,
        }
    }

    /// Get all configured shells
    pub fn configured_shells(&self) -> Vec<&str> {
        let mut shells = Vec::new();
        if self.bash.is_some() {
            shells.push("bash");
        }
        if self.zsh.is_some() {
            shells.push("zsh");
        }
        if self.fish.is_some() {
            shells.push("fish");
        }
        if self.powershell.is_some() {
            shells.push("powershell");
        }
        shells
    }
}
