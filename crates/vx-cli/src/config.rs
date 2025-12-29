//! VX Project Configuration
//!
//! This module provides configuration parsing for `vx.toml` files with support for:
//! - Project metadata
//! - Tool version management
//! - Custom setup tasks (cargo install, pip, npm, winget, custom scripts)
//! - Python environment configuration (version, venv, dependencies)
//! - Required/optional environment variables
//! - Script definitions with arguments and environment

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// VX Project Configuration
///
/// Example `vx.toml`:
/// ```toml
/// [project]
/// name = "my-project"
/// description = "A sample project"
///
/// [tools]
/// node = "20"
/// uv = "latest"
///
/// # Custom setup tasks - flexible installation commands
/// [[setup]]
/// name = "Rust dev tools"
/// run = "cargo install mdbook cargo-watch cargo-nextest"
/// check = "mdbook --version"  # Skip if this succeeds
///
/// [[setup]]
/// name = "MSVC Build Tools"
/// run = "winget install Microsoft.VisualStudio.2022.BuildTools --override '--add Microsoft.VisualStudio.Workload.VCTools'"
/// check = "cl.exe"
/// os = "windows"  # Only run on Windows
///
/// [[setup]]
/// name = "Node global packages"
/// run = "npm install -g typescript eslint"
/// check = "tsc --version"
///
/// [python]
/// version = "3.11"
/// venv = ".venv"
///
/// [env]
/// NODE_ENV = "development"
///
/// [scripts]
/// dev = "npm run dev"
/// test = "pytest"
///
/// [settings]
/// auto_install = true
/// ```
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VxConfig {
    /// Project metadata
    #[serde(default)]
    pub project: ProjectConfig,

    /// Tool versions (e.g., node = "20", uv = "latest")
    #[serde(default)]
    pub tools: HashMap<String, String>,

    /// Custom setup tasks
    #[serde(default)]
    pub setup: Vec<SetupTask>,

    /// Python environment configuration
    #[serde(default)]
    pub python: Option<PythonConfig>,

    /// Environment variables
    #[serde(default, rename = "env")]
    pub env_vars: EnvConfig,

    /// Scripts
    #[serde(default)]
    pub scripts: ScriptsConfig,

    /// Settings
    #[serde(default)]
    pub settings: SettingsConfig,
}

/// Project metadata
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project name
    #[serde(default)]
    pub name: Option<String>,

    /// Project description
    #[serde(default)]
    pub description: Option<String>,

    /// Project version
    #[serde(default)]
    pub version: Option<String>,
}

/// Custom setup task - flexible installation command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupTask {
    /// Task name for display
    pub name: String,

    /// Command to run for installation
    pub run: String,

    /// Optional check command - if this succeeds, skip the task
    #[serde(default)]
    pub check: Option<String>,

    /// Only run on specific OS: "windows", "macos", "linux"
    #[serde(default)]
    pub os: Option<String>,

    /// Working directory for the command
    #[serde(default)]
    pub cwd: Option<String>,

    /// Environment variables for this task
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Continue even if this task fails
    #[serde(default)]
    pub optional: bool,

    /// Run in shell (default: true)
    #[serde(default = "default_true")]
    pub shell: bool,
}

impl SetupTask {
    /// Check if this task should run on the current OS
    pub fn should_run_on_current_os(&self) -> bool {
        match &self.os {
            None => true,
            Some(os) => {
                let current_os = if cfg!(target_os = "windows") {
                    "windows"
                } else if cfg!(target_os = "macos") {
                    "macos"
                } else {
                    "linux"
                };
                os.to_lowercase() == current_os
            }
        }
    }

    /// Check if the task is already satisfied (check command succeeds)
    pub fn is_satisfied(&self) -> bool {
        match &self.check {
            None => false,
            Some(check_cmd) => {
                let result = if cfg!(windows) {
                    std::process::Command::new("cmd")
                        .args(["/C", check_cmd])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status()
                } else {
                    std::process::Command::new("sh")
                        .args(["-c", check_cmd])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status()
                };
                result.map(|s| s.success()).unwrap_or(false)
            }
        }
    }
}

/// Python environment configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PythonConfig {
    /// Python version (e.g., "3.8", "3.11")
    #[serde(default)]
    pub version: Option<String>,

    /// Virtual environment directory (default: ".venv")
    #[serde(default = "default_venv")]
    pub venv: String,

    /// Python dependencies
    #[serde(default)]
    pub dependencies: PythonDependencies,
}

fn default_venv() -> String {
    ".venv".to_string()
}

/// Python dependencies configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PythonDependencies {
    /// Requirements files to install from
    #[serde(default)]
    pub requirements: Vec<String>,

    /// Git dependencies (URLs)
    #[serde(default)]
    pub git: Vec<String>,

    /// Direct package specifications
    #[serde(default)]
    pub packages: Vec<String>,

    /// Development dependencies
    #[serde(default)]
    pub dev: Vec<String>,
}

/// Environment variables configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EnvConfig {
    /// Static environment variables
    #[serde(flatten)]
    pub vars: HashMap<String, EnvValue>,

    /// Required environment variables (must be set)
    #[serde(default)]
    pub required: HashMap<String, String>,

    /// Optional environment variables (with descriptions)
    #[serde(default)]
    pub optional: HashMap<String, String>,
}

/// Environment variable value (can be string or object)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EnvValue {
    /// Simple string value
    String(String),
    /// Object with source specification
    Object {
        /// Source: "env", "file", "command"
        from: Option<String>,
        /// Default value if not found
        default: Option<String>,
        /// Whether this variable is required
        required: Option<bool>,
    },
}

impl EnvValue {
    /// Get the resolved value
    pub fn resolve(&self) -> Option<String> {
        match self {
            EnvValue::String(s) => Some(s.clone()),
            EnvValue::Object { from, default, .. } => {
                if let Some(source) = from {
                    if source == "env" {
                        // Try to get from environment
                        std::env::var(source).ok().or_else(|| default.clone())
                    } else {
                        default.clone()
                    }
                } else {
                    default.clone()
                }
            }
        }
    }
}

/// Scripts configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ScriptsConfig {
    /// Simple string scripts
    #[serde(flatten)]
    pub scripts: HashMap<String, ScriptValue>,
}

/// Script value (can be string or object)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ScriptValue {
    /// Simple command string
    Simple(String),
    /// Object with command and arguments
    Object(ScriptObject),
}

/// Script object with command and arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptObject {
    /// Command to run
    pub command: String,

    /// Description of the script
    #[serde(default)]
    pub description: Option<String>,

    /// Arguments to pass
    #[serde(default)]
    pub args: Vec<String>,

    /// Working directory
    #[serde(default)]
    pub cwd: Option<String>,

    /// Environment variables for this script
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl ScriptValue {
    /// Get the full command string
    pub fn to_command(&self) -> String {
        match self {
            ScriptValue::Simple(cmd) => cmd.clone(),
            ScriptValue::Object(obj) => {
                if obj.args.is_empty() {
                    obj.command.clone()
                } else {
                    format!("{} {}", obj.command, obj.args.join(" "))
                }
            }
        }
    }

    /// Get description if available
    pub fn description(&self) -> Option<&str> {
        match self {
            ScriptValue::Simple(_) => None,
            ScriptValue::Object(obj) => obj.description.as_deref(),
        }
    }
}

/// Settings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsConfig {
    /// Automatically install missing tools
    #[serde(default = "default_true")]
    pub auto_install: bool,

    /// Install tools in parallel
    #[serde(default = "default_true")]
    pub parallel_install: bool,

    /// Cache duration (e.g., "7d", "24h")
    #[serde(default = "default_cache_duration")]
    pub cache_duration: String,
}

impl Default for SettingsConfig {
    fn default() -> Self {
        Self {
            auto_install: true,
            parallel_install: true,
            cache_duration: "7d".to_string(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_cache_duration() -> String {
    "7d".to_string()
}

impl VxConfig {
    /// Parse configuration from a file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        Self::parse(&content)
    }

    /// Parse configuration from a string
    pub fn parse(content: &str) -> Result<Self> {
        toml::from_str(content).context("Failed to parse vx.toml")
    }

    /// Write configuration to a file
    pub fn to_file(&self, path: &Path) -> Result<()> {
        let content = self.to_string_pretty()?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Convert to pretty TOML string
    pub fn to_string_pretty(&self) -> Result<String> {
        let mut content = String::new();

        // Header
        content.push_str("# VX Project Configuration\n");
        content.push_str("# Run 'vx setup' to install all required tools.\n");
        content.push_str("# Run 'vx dev' to enter the development environment.\n\n");

        // Project section
        if self.project.name.is_some() || self.project.description.is_some() {
            content.push_str("[project]\n");
            if let Some(name) = &self.project.name {
                content.push_str(&format!("name = \"{}\"\n", name));
            }
            if let Some(desc) = &self.project.description {
                content.push_str(&format!("description = \"{}\"\n", desc));
            }
            if let Some(ver) = &self.project.version {
                content.push_str(&format!("version = \"{}\"\n", ver));
            }
            content.push('\n');
        }

        // Tools section
        if !self.tools.is_empty() {
            content.push_str("[tools]\n");
            let mut tools: Vec<_> = self.tools.iter().collect();
            tools.sort_by_key(|(k, _)| *k);
            for (name, version) in tools {
                content.push_str(&format!("{} = \"{}\"\n", name, version));
            }
            content.push('\n');
        }

        // Setup tasks section
        for task in &self.setup {
            content.push_str("[[setup]]\n");
            content.push_str(&format!("name = \"{}\"\n", task.name));
            content.push_str(&format!("run = \"{}\"\n", task.run));
            if let Some(check) = &task.check {
                content.push_str(&format!("check = \"{}\"\n", check));
            }
            if let Some(os) = &task.os {
                content.push_str(&format!("os = \"{}\"\n", os));
            }
            if task.optional {
                content.push_str("optional = true\n");
            }
            content.push('\n');
        }

        // Python section
        if let Some(python) = &self.python {
            content.push_str("[python]\n");
            if let Some(ver) = &python.version {
                content.push_str(&format!("version = \"{}\"\n", ver));
            }
            content.push_str(&format!("venv = \"{}\"\n", python.venv));
            content.push('\n');

            // Python dependencies
            if !python.dependencies.requirements.is_empty()
                || !python.dependencies.git.is_empty()
                || !python.dependencies.packages.is_empty()
            {
                content.push_str("[python.dependencies]\n");
                if !python.dependencies.requirements.is_empty() {
                    content.push_str(&format!(
                        "requirements = [{}]\n",
                        python
                            .dependencies
                            .requirements
                            .iter()
                            .map(|s| format!("\"{}\"", s))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                if !python.dependencies.git.is_empty() {
                    content.push_str("git = [\n");
                    for url in &python.dependencies.git {
                        content.push_str(&format!("    \"{}\",\n", url));
                    }
                    content.push_str("]\n");
                }
                if !python.dependencies.packages.is_empty() {
                    content.push_str(&format!(
                        "packages = [{}]\n",
                        python
                            .dependencies
                            .packages
                            .iter()
                            .map(|s| format!("\"{}\"", s))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                content.push('\n');
            }
        }

        // Environment variables
        let has_env = !self.env_vars.vars.is_empty()
            || !self.env_vars.required.is_empty()
            || !self.env_vars.optional.is_empty();

        if has_env {
            // Simple env vars
            if !self.env_vars.vars.is_empty() {
                content.push_str("[env]\n");
                for (key, value) in &self.env_vars.vars {
                    if let EnvValue::String(v) = value {
                        content.push_str(&format!("{} = \"{}\"\n", key, v));
                    }
                }
                content.push('\n');
            }

            // Required env vars
            if !self.env_vars.required.is_empty() {
                content.push_str("[env.required]\n");
                for (key, desc) in &self.env_vars.required {
                    content.push_str(&format!("{} = \"{}\"  # Required\n", key, desc));
                }
                content.push('\n');
            }

            // Optional env vars
            if !self.env_vars.optional.is_empty() {
                content.push_str("[env.optional]\n");
                for (key, desc) in &self.env_vars.optional {
                    content.push_str(&format!("{} = \"{}\"  # Optional\n", key, desc));
                }
                content.push('\n');
            }
        }

        // Scripts section
        if !self.scripts.scripts.is_empty() {
            content.push_str("[scripts]\n");
            let mut scripts: Vec<_> = self.scripts.scripts.iter().collect();
            scripts.sort_by_key(|(k, _)| *k);

            for (name, script) in scripts {
                match script {
                    ScriptValue::Simple(cmd) => {
                        content.push_str(&format!("{} = \"{}\"\n", name, cmd));
                    }
                    ScriptValue::Object(obj) => {
                        content.push_str(&format!("\n[scripts.{}]\n", name));
                        content.push_str(&format!("command = \"{}\"\n", obj.command));
                        if let Some(desc) = &obj.description {
                            content.push_str(&format!("description = \"{}\"\n", desc));
                        }
                        if !obj.args.is_empty() {
                            content.push_str(&format!(
                                "args = [{}]\n",
                                obj.args
                                    .iter()
                                    .map(|s| format!("\"{}\"", s))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ));
                        }
                    }
                }
            }
            content.push('\n');
        }

        // Settings section
        content.push_str("[settings]\n");
        content.push_str(&format!("auto_install = {}\n", self.settings.auto_install));
        content.push_str(&format!(
            "cache_duration = \"{}\"\n",
            self.settings.cache_duration
        ));

        Ok(content)
    }

    /// Get all required environment variables that are not set
    pub fn missing_required_env_vars(&self) -> Vec<(&str, &str)> {
        self.env_vars
            .required
            .iter()
            .filter(|(key, _)| std::env::var(key).is_err())
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect()
    }

    /// Check if Python setup is needed
    pub fn needs_python_setup(&self) -> bool {
        self.python.is_some()
    }

    /// Check if there are setup tasks to run
    pub fn has_setup_tasks(&self) -> bool {
        !self.setup.is_empty()
    }

    /// Get setup tasks that should run on the current OS
    pub fn get_applicable_setup_tasks(&self) -> Vec<&SetupTask> {
        self.setup
            .iter()
            .filter(|t| t.should_run_on_current_os())
            .collect()
    }

    /// Get all scripts as name -> command pairs
    pub fn get_scripts(&self) -> HashMap<String, String> {
        self.scripts
            .scripts
            .iter()
            .map(|(name, script)| (name.clone(), script.to_command()))
            .collect()
    }
}
