//! Command output structures (RFC 0031, RFC 0035)
//!
//! This module defines structured output types for all CLI commands.
//! Each output type implements `CommandOutput` for unified `--json` support.

use crate::cli::OutputFormat;
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;

/// Serialize a value to TOON format using the official toon-format library.
fn to_toon<T: Serialize>(value: &T) -> Result<String> {
    toon_format::encode_default(value).map_err(|e| anyhow::anyhow!("TOON encoding error: {}", e))
}

// ============================================================================
// Core traits and types
// ============================================================================

/// Trait for command output that supports multiple render formats.
///
/// Commands implement this trait to enable `--json` and `--format` support.
/// The command only needs to define "what data to return" — the rendering
/// format is controlled by global CLI arguments.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Serialize)]
/// pub struct ListOutput {
///     pub runtimes: Vec<RuntimeEntry>,
/// }
///
/// impl CommandOutput for ListOutput {
///     fn render_text(&self, writer: &mut dyn std::io::Write) -> Result<()> {
///         writeln!(writer, "Installed Runtimes:")?;
///         for rt in &self.runtimes {
///             writeln!(writer, "  {} {}", rt.name, rt.version)?;
///         }
///         Ok(())
///     }
/// }
/// ```
pub trait CommandOutput: Serialize {
    /// Render human-readable text output to the given writer.
    ///
    /// This is called when `--format text` (default) is used.
    /// Output should include colors, emoji, and formatting for human consumption.
    fn render_text(&self, writer: &mut dyn std::io::Write) -> Result<()>;
}

/// Renders command output in the requested format.
///
/// Selects between text, JSON, and (future) TOON output based on
/// the global `--format` / `--json` flags.
pub struct OutputRenderer {
    format: OutputFormat,
}

impl OutputRenderer {
    /// Create a new renderer with the given format.
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Create a renderer for JSON output.
    pub fn json() -> Self {
        Self::new(OutputFormat::Json)
    }

    /// Create a renderer for text output.
    pub fn text() -> Self {
        Self::new(OutputFormat::Text)
    }

    /// Get the current output format.
    pub fn format(&self) -> OutputFormat {
        self.format
    }

    /// Check if JSON output is active.
    pub fn is_json(&self) -> bool {
        self.format == OutputFormat::Json
    }

    /// Check if text output is active.
    pub fn is_text(&self) -> bool {
        self.format == OutputFormat::Text
    }

    /// Render the output in the selected format.
    ///
    /// - Text: calls `output.render_text()` writing to stdout
    /// - JSON: serializes to pretty JSON and prints to stdout
    /// - TOON: serializes to token-optimized format for LLM prompts
    pub fn render<T: CommandOutput>(&self, output: &T) -> Result<()> {
        match self.format {
            OutputFormat::Text => {
                let mut stdout = std::io::stdout().lock();
                output.render_text(&mut stdout)?;
                Ok(())
            }
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(output)?;
                println!("{json}");
                Ok(())
            }
            OutputFormat::Toon => {
                let toon = to_toon(output)?;
                println!("{toon}");
                Ok(())
            }
        }
    }

    /// Render output to a string (useful for testing).
    pub fn render_to_string<T: CommandOutput>(&self, output: &T) -> Result<String> {
        match self.format {
            OutputFormat::Text => {
                let mut buf = Vec::new();
                output.render_text(&mut buf)?;
                Ok(String::from_utf8(buf)?)
            }
            OutputFormat::Json => Ok(serde_json::to_string_pretty(output)?),
            OutputFormat::Toon => to_toon(output),
        }
    }
}

// ============================================================================
// vx list output
// ============================================================================

/// Output for `vx list` command
#[derive(Serialize)]
pub struct ListOutput {
    /// List of runtimes
    pub runtimes: Vec<RuntimeEntry>,
    /// Total count of runtimes
    pub total: usize,
    /// Count of installed runtimes
    pub installed_count: usize,
    /// Current platform
    pub platform: String,
}

/// A single runtime entry in the list
#[derive(Serialize)]
pub struct RuntimeEntry {
    /// Runtime name
    pub name: String,
    /// Installed versions (empty if not installed)
    pub versions: Vec<String>,
    /// Whether the runtime is installed
    pub installed: bool,
    /// Runtime description
    pub description: String,
    /// Whether the runtime is supported on current platform
    pub platform_supported: bool,
    /// Ecosystem (nodejs, python, go, etc.)
    pub ecosystem: Option<String>,
    /// Platform label (e.g., "Windows only")
    pub platform_label: Option<String>,
}

impl CommandOutput for ListOutput {
    fn render_text(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        writeln!(writer)?;
        writeln!(writer, "📦 Available Tools ({})", self.platform)?;
        writeln!(writer)?;

        for rt in &self.runtimes {
            let status_icon = if rt.installed {
                "✅"
            } else if !rt.platform_supported {
                "⚠️ "
            } else {
                "❌"
            };

            let platform_note = if !rt.platform_supported {
                if let Some(ref label) = rt.platform_label {
                    format!(" ({})", label)
                } else {
                    " (not supported)".to_string()
                }
            } else {
                String::new()
            };

            writeln!(
                writer,
                "  {} {} - {}{}",
                status_icon, rt.name, rt.description, platform_note
            )?;

            if rt.installed && !rt.versions.is_empty() {
                writeln!(writer, "     Versions: {}", rt.versions.join(", "))?;
            }
        }

        writeln!(writer)?;
        writeln!(
            writer,
            "📊 Summary: {}/{} tools installed",
            self.installed_count, self.total
        )?;

        Ok(())
    }
}

// ============================================================================
// vx versions output
// ============================================================================

/// Output for `vx versions <tool>` command
#[derive(Serialize)]
pub struct VersionsOutput {
    /// Tool name
    pub tool: String,
    /// List of available versions
    pub versions: Vec<VersionEntry>,
    /// Total count
    pub total: usize,
    /// Latest version
    pub latest: Option<String>,
    /// LTS version (if applicable)
    pub lts: Option<String>,
}

/// A single version entry
#[derive(Serialize)]
pub struct VersionEntry {
    /// Version string
    pub version: String,
    /// Whether this version is installed
    pub installed: bool,
    /// Whether this is an LTS version
    pub lts: bool,
    /// LTS name (e.g., "iron", "hydrogen")
    pub lts_name: Option<String>,
    /// Release date
    pub date: Option<String>,
    /// Whether this is a prerelease
    pub prerelease: bool,
    /// Download URL (if available)
    pub download_url: Option<String>,
}

impl CommandOutput for VersionsOutput {
    fn render_text(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        writeln!(writer)?;
        writeln!(
            writer,
            "📦 {} - {} versions available",
            self.tool, self.total
        )?;
        writeln!(writer)?;

        // Limit display to 20 versions in text mode
        let display_count = std::cmp::min(self.versions.len(), 20);

        for (i, v) in self.versions.iter().take(display_count).enumerate() {
            let installed_marker = if v.installed { " ✅" } else { "" };
            let lts_marker = if v.lts {
                if let Some(ref name) = v.lts_name {
                    &format!(" (LTS: {})", name)
                } else {
                    " (LTS)"
                }
            } else {
                ""
            };
            let prerelease_marker = if v.prerelease { " (prerelease)" } else { "" };

            writeln!(
                writer,
                "  {}. {}{}{}{}",
                i + 1,
                v.version,
                installed_marker,
                lts_marker,
                prerelease_marker
            )?;
        }

        if self.versions.len() > 20 {
            writeln!(
                writer,
                "  ... and {} more versions",
                self.versions.len() - 20
            )?;
        }

        writeln!(writer)?;
        if let Some(ref latest) = self.latest {
            writeln!(writer, "  Latest: {}", latest)?;
        }
        if let Some(ref lts) = self.lts {
            writeln!(writer, "  LTS: {}", lts)?;
        }

        Ok(())
    }
}

// ============================================================================
// vx which output
// ============================================================================

/// Output for `vx which <tool>` command
#[derive(Serialize)]
pub struct WhichOutput {
    /// Tool name
    pub tool: String,
    /// Resolved version (if found)
    pub version: Option<String>,
    /// Executable path
    pub path: Option<String>,
    /// Source of the tool
    pub source: ToolSource,
    /// All matching paths (when --all is used)
    pub all_paths: Vec<ToolPathEntry>,
}

/// Source of a tool
#[derive(Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolSource {
    /// vx-managed installation
    Vx,
    /// System PATH
    System,
    /// Global package (npm/pip/etc.)
    GlobalPackage,
    /// Detected via provider manifest
    Detected,
    /// Not found
    NotFound,
}

/// A single path entry for `--all` mode
#[derive(Serialize)]
pub struct ToolPathEntry {
    /// Executable path
    pub path: String,
    /// Version (if determinable)
    pub version: Option<String>,
    /// Source of this path
    pub source: ToolSource,
}

impl CommandOutput for WhichOutput {
    fn render_text(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        if !self.all_paths.is_empty() {
            // Multiple paths (--all mode)
            for entry in &self.all_paths {
                let source_label = match entry.source {
                    ToolSource::Vx => "",
                    ToolSource::System => " (system)",
                    ToolSource::GlobalPackage => " (global package)",
                    ToolSource::Detected => " (detected)",
                    ToolSource::NotFound => "",
                };
                writeln!(writer, "{}{}", entry.path, source_label)?;
            }
        } else if let Some(ref path) = self.path {
            // Single path
            let source_label = match self.source {
                ToolSource::Vx => "",
                ToolSource::System => " (system)",
                ToolSource::GlobalPackage => " (global package)",
                ToolSource::Detected => " (detected)",
                ToolSource::NotFound => "",
            };
            writeln!(writer, "{}{}", path, source_label)?;
        } else {
            writeln!(writer, "Tool '{}' not found", self.tool)?;
        }
        Ok(())
    }
}

// ============================================================================
// vx check output
// ============================================================================

/// Output for `vx check` command
#[derive(Serialize)]
pub struct CheckOutput {
    /// Project configuration file path
    pub project_file: Option<String>,
    /// Tool requirements status
    pub requirements: Vec<RequirementStatus>,
    /// Whether all requirements are satisfied
    pub all_satisfied: bool,
    /// List of missing tools
    pub missing_tools: Vec<String>,
    /// List of warnings
    pub warnings: Vec<String>,
    /// List of errors
    pub errors: Vec<String>,
}

/// Status of a tool requirement
#[derive(Serialize)]
pub struct RequirementStatus {
    /// Tool name
    pub runtime: String,
    /// Required version constraint
    pub required: String,
    /// Installed version (if any)
    pub installed: Option<String>,
    /// Whether the requirement is satisfied
    pub satisfied: bool,
    /// Status type
    pub status: RequirementStatusType,
    /// Action to take if not satisfied
    pub action: Option<String>,
    /// Path to the installed tool
    pub path: Option<String>,
}

/// Type of requirement status
#[derive(Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RequirementStatusType {
    /// Tool is installed and meets requirements
    Installed,
    /// Using system fallback version
    SystemFallback,
    /// Tool is not installed
    NotInstalled,
    /// Version mismatch
    VersionMismatch,
}

impl CommandOutput for CheckOutput {
    fn render_text(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        writeln!(writer)?;

        if let Some(ref path) = self.project_file {
            writeln!(writer, "Checking project: {}", path)?;
            writeln!(writer)?;
        }

        for req in &self.requirements {
            let status_icon = match req.status {
                RequirementStatusType::Installed => "✓",
                RequirementStatusType::SystemFallback => "⚠",
                RequirementStatusType::NotInstalled => "✗",
                RequirementStatusType::VersionMismatch => "✗",
            };

            let version_info = if let Some(ref ver) = req.installed {
                format!(" ({})", ver)
            } else {
                String::new()
            };

            let path_info = if let Some(ref path) = req.path {
                format!(" at {}", path)
            } else {
                String::new()
            };

            writeln!(
                writer,
                "{} {} {}{}{}",
                status_icon, req.runtime, req.required, version_info, path_info
            )?;
        }

        writeln!(writer)?;

        if self.all_satisfied && self.warnings.is_empty() {
            writeln!(writer, "✓ All version constraints satisfied")?;
        } else if self.all_satisfied {
            writeln!(
                writer,
                "⚠ All constraints satisfied with {} warning(s)",
                self.warnings.len()
            )?;
            for warn in &self.warnings {
                writeln!(writer, "  - {}", warn)?;
            }
        } else {
            writeln!(
                writer,
                "✗ {} error(s) and {} warning(s) found",
                self.errors.len(),
                self.warnings.len()
            )?;
            for err in &self.errors {
                writeln!(writer, "  - {}", err)?;
            }
        }

        Ok(())
    }
}

// ============================================================================
// vx sync output
// ============================================================================

/// Output for `vx sync` command
#[derive(Serialize)]
pub struct SyncOutput {
    /// Successfully installed tools
    pub installed: Vec<InstallResult>,
    /// Skipped tools (already installed)
    pub skipped: Vec<SkippedResult>,
    /// Failed installations
    pub failed: Vec<FailedResult>,
    /// Total duration in milliseconds
    pub duration_ms: u64,
    /// Whether all tools were synced successfully
    pub success: bool,
}

/// Result of a successful installation
#[derive(Serialize)]
pub struct InstallResult {
    /// Tool name
    pub runtime: String,
    /// Installed version
    pub version: String,
    /// Installation path
    pub path: String,
    /// Duration of installation in milliseconds
    pub duration_ms: u64,
}

/// Result of a skipped installation
#[derive(Serialize)]
pub struct SkippedResult {
    /// Tool name
    pub runtime: String,
    /// Reason for skipping
    pub reason: String,
    /// Current version (if applicable)
    pub current_version: Option<String>,
}

/// Result of a failed installation
#[derive(Serialize)]
pub struct FailedResult {
    /// Tool name
    pub runtime: String,
    /// Requested version
    pub version: String,
    /// Error message
    pub error: String,
}

impl CommandOutput for SyncOutput {
    fn render_text(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        writeln!(writer)?;

        if !self.installed.is_empty() {
            writeln!(writer, "✓ Installed {} tool(s):", self.installed.len())?;
            for result in &self.installed {
                writeln!(
                    writer,
                    "  - {}@{} ({})",
                    result.runtime, result.version, result.path
                )?;
            }
        }

        if !self.skipped.is_empty() {
            writeln!(writer, "○ Skipped {} tool(s):", self.skipped.len())?;
            for result in &self.skipped {
                if let Some(ref ver) = result.current_version {
                    writeln!(writer, "  - {}@{} ({})", result.runtime, ver, result.reason)?;
                } else {
                    writeln!(writer, "  - {} ({})", result.runtime, result.reason)?;
                }
            }
        }

        if !self.failed.is_empty() {
            writeln!(writer, "✗ Failed to install {} tool(s):", self.failed.len())?;
            for result in &self.failed {
                writeln!(
                    writer,
                    "  - {}@{}: {}",
                    result.runtime, result.version, result.error
                )?;
            }
        }

        writeln!(writer)?;
        writeln!(writer, "Duration: {}ms", self.duration_ms)?;

        if self.success {
            writeln!(writer, "✓ Sync completed successfully")?;
        } else {
            writeln!(writer, "✗ Sync completed with errors")?;
        }

        Ok(())
    }
}

// ============================================================================
// vx install output
// ============================================================================

/// Output for `vx install` command
#[derive(Serialize)]
pub struct InstallOutput {
    /// Tool name
    pub runtime: String,
    /// Installed version
    pub version: String,
    /// Installation path
    pub path: String,
    /// Whether the tool was already installed
    pub already_installed: bool,
    /// Installation duration in milliseconds
    pub duration_ms: u64,
    /// Dependencies that were also installed
    pub dependencies_installed: Vec<DependencyInstalled>,
}

/// A dependency that was installed
#[derive(Serialize)]
pub struct DependencyInstalled {
    /// Dependency name
    pub runtime: String,
    /// Dependency version
    pub version: String,
}

impl CommandOutput for InstallOutput {
    fn render_text(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        writeln!(writer)?;

        if self.already_installed {
            writeln!(
                writer,
                "○ {}@{} already installed at {}",
                self.runtime, self.version, self.path
            )?;
        } else {
            writeln!(
                writer,
                "✓ Installed {}@{} to {}",
                self.runtime, self.version, self.path
            )?;
        }

        if !self.dependencies_installed.is_empty() {
            writeln!(writer, "  Dependencies installed:")?;
            for dep in &self.dependencies_installed {
                writeln!(writer, "    - {}@{}", dep.runtime, dep.version)?;
            }
        }

        writeln!(writer, "  Duration: {}ms", self.duration_ms)?;

        Ok(())
    }
}

// ============================================================================
// vx env output
// ============================================================================

/// Output for `vx env` command
#[derive(Serialize)]
pub struct EnvOutput {
    /// Environment variables
    pub variables: HashMap<String, String>,
    /// PATH entries to prepend
    pub path_prepend: Vec<String>,
    /// Active runtimes
    pub active_runtimes: Vec<ActiveRuntime>,
}

/// An active runtime in the environment
#[derive(Serialize)]
pub struct ActiveRuntime {
    /// Runtime name
    pub name: String,
    /// Active version
    pub version: String,
    /// Path to the runtime
    pub path: String,
}

impl CommandOutput for EnvOutput {
    fn render_text(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        writeln!(writer)?;

        if !self.path_prepend.is_empty() {
            writeln!(writer, "PATH prepend:")?;
            for path in &self.path_prepend {
                writeln!(writer, "  {}", path)?;
            }
        }

        if !self.variables.is_empty() {
            writeln!(writer, "Environment variables:")?;
            let mut vars: Vec<_> = self.variables.iter().collect();
            vars.sort_by_key(|(k, _)| *k);
            for (key, value) in vars {
                writeln!(writer, "  {}={}", key, value)?;
            }
        }

        if !self.active_runtimes.is_empty() {
            writeln!(writer, "Active runtimes:")?;
            for rt in &self.active_runtimes {
                writeln!(writer, "  {}@{} ({})", rt.name, rt.version, rt.path)?;
            }
        }

        Ok(())
    }
}

// ============================================================================
// vx analyze output
// ============================================================================

/// Output for `vx analyze` command
#[derive(Serialize)]
pub struct AnalyzeOutput {
    /// Detected ecosystems
    pub ecosystems: Vec<EcosystemInfo>,
    /// Project dependencies
    pub dependencies: Vec<DependencyInfo>,
    /// Available scripts
    pub scripts: Vec<ScriptInfo>,
    /// Required tools
    pub required_tools: Vec<ToolRequirement>,
    /// Sync actions suggested
    pub sync_actions: Vec<SyncActionInfo>,
    /// Project root path
    pub project_root: String,
}

/// Ecosystem information
#[derive(Serialize)]
pub struct EcosystemInfo {
    /// Ecosystem name (e.g., "nodejs", "python", "rust")
    pub name: String,
    /// Display name
    pub display_name: String,
    /// Package manager (if detected)
    pub package_manager: Option<String>,
}

/// Dependency information
#[derive(Serialize)]
pub struct DependencyInfo {
    /// Dependency name
    pub name: String,
    /// Version constraint
    pub version: Option<String>,
    /// Ecosystem
    pub ecosystem: String,
    /// Whether it's a dev dependency
    pub is_dev: bool,
    /// Whether it's installed
    pub is_installed: bool,
}

/// Script information
#[derive(Serialize)]
pub struct ScriptInfo {
    /// Script name
    pub name: String,
    /// Command to execute
    pub command: String,
    /// Description (if available)
    pub description: Option<String>,
    /// Tools required by this script
    pub tools: Vec<String>,
}

/// Tool requirement
#[derive(Serialize)]
pub struct ToolRequirement {
    /// Tool name
    pub name: String,
    /// Version constraint (if specified)
    pub version: Option<String>,
    /// Whether the tool is available
    pub is_available: bool,
    /// Source of availability (vx-managed, system, etc.)
    pub source: Option<String>,
}

/// Sync action suggestion
#[derive(Serialize)]
pub struct SyncActionInfo {
    /// Action type (install, skip, etc.)
    pub action: String,
    /// Tool name
    pub tool: String,
    /// Version to install
    pub version: Option<String>,
    /// Reason for the action
    pub reason: String,
}

impl CommandOutput for AnalyzeOutput {
    fn render_text(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        writeln!(writer)?;
        writeln!(writer, "📊 Project Analysis")?;
        writeln!(writer)?;

        // Ecosystems
        if !self.ecosystems.is_empty() {
            let ecosystems: Vec<_> = self
                .ecosystems
                .iter()
                .map(|e| e.display_name.as_str())
                .collect();
            writeln!(writer, "Ecosystems: {}", ecosystems.join(", "))?;
            writeln!(writer)?;
        }

        // Dependencies
        if !self.dependencies.is_empty() {
            writeln!(writer, "📦 Dependencies:")?;
            let mut by_ecosystem: std::collections::HashMap<&str, Vec<_>> =
                std::collections::HashMap::new();
            for dep in &self.dependencies {
                by_ecosystem.entry(&dep.ecosystem).or_default().push(dep);
            }

            for (ecosystem, deps) in by_ecosystem {
                writeln!(writer, "  {} ({}):", ecosystem, deps.len())?;
                for dep in deps {
                    let status = if dep.is_installed { "✅" } else { "⚠️ " };
                    let version = dep
                        .version
                        .as_ref()
                        .map(|v| format!(" = \"{}\"", v))
                        .unwrap_or_default();
                    let dev_marker = if dep.is_dev { " (dev)" } else { "" };
                    let installed_marker = if !dep.is_installed {
                        " - not installed"
                    } else {
                        ""
                    };
                    writeln!(
                        writer,
                        "    {} {}{}{}{}",
                        status, dep.name, version, dev_marker, installed_marker
                    )?;
                }
            }
            writeln!(writer)?;
        }

        // Scripts
        if !self.scripts.is_empty() {
            writeln!(writer, "📜 Scripts:")?;
            for script in &self.scripts {
                writeln!(writer, "  {}: {}", script.name, script.command)?;
            }
            writeln!(writer)?;
        }

        // Required tools
        if !self.required_tools.is_empty() {
            writeln!(writer, "🔧 Required Tools:")?;
            for tool in &self.required_tools {
                let status = if tool.is_available { "✅" } else { "❌" };
                let version = tool
                    .version
                    .as_ref()
                    .map(|v| format!("@{}", v))
                    .unwrap_or_default();
                writeln!(writer, "  {} {}{}", status, tool.name, version)?;
            }
            writeln!(writer)?;
        }

        // Sync actions
        if !self.sync_actions.is_empty() {
            writeln!(writer, "💡 Suggested Actions:")?;
            for action in &self.sync_actions {
                let version_suffix = action
                    .version
                    .as_ref()
                    .map(|v| format!("@{}", v))
                    .unwrap_or_default();
                writeln!(
                    writer,
                    "  - {}: {}{} ({})",
                    action.action, action.tool, version_suffix, action.reason
                )?;
            }
            writeln!(writer)?;
        }

        Ok(())
    }
}

// ============================================================================
// vx ai context output
// ============================================================================

/// Output for `vx ai context` command
#[derive(Serialize)]
pub struct AiContextOutput {
    /// Project information
    pub project: ProjectInfo,
    /// Installed tools
    pub tools: Vec<ToolInfo>,
    /// Available scripts
    pub scripts: Vec<ScriptInfo>,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    /// Tool constraints
    pub constraints: Vec<ConstraintInfo>,
    /// Important files
    pub important_files: Vec<String>,
    /// Recommended commands
    pub recommended_commands: Vec<String>,
}

/// Project information for AI context
#[derive(Serialize)]
pub struct ProjectInfo {
    /// Project name
    pub name: String,
    /// Project root path
    pub root: String,
    /// Detected languages
    pub languages: Vec<String>,
    /// Detected frameworks
    pub frameworks: Vec<String>,
    /// Package managers in use
    pub package_managers: Vec<String>,
}

/// Tool information for AI context
#[derive(Serialize)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,
    /// Installed version
    pub version: String,
    /// Source (vx, system, project)
    pub source: String,
    /// Ecosystem
    pub ecosystem: Option<String>,
    /// Path to executable
    pub path: Option<String>,
}

/// Constraint information for AI context
#[derive(Serialize)]
pub struct ConstraintInfo {
    /// Tool name
    pub tool: String,
    /// Version constraint
    pub constraint: String,
    /// Reason for the constraint
    pub reason: Option<String>,
    /// Whether the constraint is satisfied
    pub satisfied: bool,
}

impl CommandOutput for AiContextOutput {
    fn render_text(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        // Markdown format for AI prompts
        writeln!(writer, "# Project Context")?;
        writeln!(writer)?;

        // Project info
        writeln!(writer, "## Project: {}", self.project.name)?;
        writeln!(writer)?;
        writeln!(writer, "**Root**: {}", self.project.root)?;
        writeln!(writer)?;

        if !self.project.languages.is_empty() {
            writeln!(
                writer,
                "**Languages**: {}",
                self.project.languages.join(", ")
            )?;
        }

        if !self.project.frameworks.is_empty() {
            writeln!(
                writer,
                "**Frameworks**: {}",
                self.project.frameworks.join(", ")
            )?;
        }

        if !self.project.package_managers.is_empty() {
            writeln!(
                writer,
                "**Package Managers**: {}",
                self.project.package_managers.join(", ")
            )?;
        }
        writeln!(writer)?;

        // Tools
        if !self.tools.is_empty() {
            writeln!(writer, "## Environment")?;
            writeln!(writer)?;
            for tool in &self.tools {
                let source_label = match tool.source.as_str() {
                    "vx" => "vx-managed",
                    "system" => "system",
                    "project" => "project-local",
                    _ => &tool.source,
                };
                writeln!(
                    writer,
                    "- Runtime: {}@{} ({})",
                    tool.name, tool.version, source_label
                )?;
            }
            writeln!(writer)?;
        }

        // Scripts
        if !self.scripts.is_empty() {
            writeln!(writer, "## Available Scripts")?;
            writeln!(writer)?;
            for script in &self.scripts {
                writeln!(writer, "- `vx run {}` - {}", script.name, script.command)?;
            }
            writeln!(writer)?;
        }

        // Constraints
        if !self.constraints.is_empty() {
            writeln!(writer, "## Tool Constraints")?;
            writeln!(writer)?;
            for constraint in &self.constraints {
                let status = if constraint.satisfied { "✓" } else { "✗" };
                let reason = constraint
                    .reason
                    .as_ref()
                    .map(|r| format!(" ({})", r))
                    .unwrap_or_default();
                writeln!(
                    writer,
                    "- {} {} {}{}",
                    status, constraint.tool, constraint.constraint, reason
                )?;
            }
            writeln!(writer)?;
        }

        // Important files
        if !self.important_files.is_empty() {
            writeln!(writer, "## Important Files")?;
            writeln!(writer)?;
            for file in &self.important_files {
                writeln!(writer, "- {}", file)?;
            }
            writeln!(writer)?;
        }

        // Recommended commands
        if !self.recommended_commands.is_empty() {
            writeln!(writer, "## Quick Commands")?;
            writeln!(writer)?;
            for cmd in &self.recommended_commands {
                writeln!(writer, "```bash")?;
                writeln!(writer, "{}", cmd)?;
                writeln!(writer, "```")?;
                writeln!(writer)?;
            }
        }

        Ok(())
    }
}
