//! Configuration synchronization between project files and vx.toml

use crate::error::AnalyzerResult;
use crate::types::ProjectAnalysis;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::info;
use vx_paths::project::{CONFIG_FILE_NAME, CONFIG_FILE_NAME_LEGACY};

/// Sync action to apply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncAction {
    /// Add a tool to vx.toml
    AddTool { name: String, version: String },

    /// Update tool version in vx.toml
    UpdateTool {
        name: String,
        old_version: String,
        new_version: String,
    },

    /// Add a script to vx.toml
    AddScript { name: String, command: String },

    /// Update script command in vx.toml
    UpdateScript {
        name: String,
        old_command: String,
        new_command: String,
    },

    /// Install a dependency
    InstallDependency {
        command: String,
        description: String,
    },

    /// Add dependency to project config
    AddProjectDependency {
        file: std::path::PathBuf,
        section: String,
        content: String,
    },
}

impl std::fmt::Display for SyncAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncAction::AddTool { name, version } => {
                write!(f, "Add tool: {} = \"{}\"", name, version)
            }
            SyncAction::UpdateTool {
                name,
                old_version,
                new_version,
            } => {
                write!(
                    f,
                    "Update tool: {} \"{}\" → \"{}\"",
                    name, old_version, new_version
                )
            }
            SyncAction::AddScript { name, command } => {
                write!(f, "Add script: {} = \"{}\"", name, command)
            }
            SyncAction::UpdateScript {
                name,
                old_command,
                new_command,
            } => {
                write!(
                    f,
                    "Update script: {} \"{}\" → \"{}\"",
                    name, old_command, new_command
                )
            }
            SyncAction::InstallDependency {
                command,
                description,
            } => {
                write!(f, "Install: {} ({})", command, description)
            }
            SyncAction::AddProjectDependency {
                file,
                section,
                content,
            } => {
                write!(f, "Add to {} [{}]: {}", file.display(), section, content)
            }
        }
    }
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Keep the value in vx.toml
    KeepLocal,
    /// Use the value from project config
    UseProject,
    /// Merge values (append scripts, use latest version)
    #[default]
    Merge,
    /// Ask user for resolution
    Ask,
}

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Whether sync is enabled
    pub enabled: bool,

    /// Sources to sync from (in priority order)
    pub sources: Vec<String>,

    /// Script sync settings
    pub scripts: ScriptSyncConfig,

    /// Tool sync settings
    pub tools: ToolSyncConfig,

    /// Dependency sync settings
    pub dependencies: DependencySyncConfig,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sources: vec![
                "pyproject.toml".to_string(),
                "package.json".to_string(),
                "Cargo.toml".to_string(),
            ],
            scripts: ScriptSyncConfig::default(),
            tools: ToolSyncConfig::default(),
            dependencies: DependencySyncConfig::default(),
        }
    }
}

/// Script sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptSyncConfig {
    /// Import scripts from project config
    pub import_from_project: bool,
    /// Overwrite existing scripts
    pub overwrite_existing: bool,
    /// Prefix for imported scripts
    pub prefix: String,
}

impl Default for ScriptSyncConfig {
    fn default() -> Self {
        Self {
            import_from_project: true,
            overwrite_existing: false,
            prefix: String::new(),
        }
    }
}

/// Tool sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSyncConfig {
    /// Auto-detect and add tools
    pub auto_detect: bool,
    /// Version strategy
    pub version_strategy: VersionStrategy,
}

impl Default for ToolSyncConfig {
    fn default() -> Self {
        Self {
            auto_detect: true,
            version_strategy: VersionStrategy::Minor,
        }
    }
}

/// Version strategy for tool sync
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum VersionStrategy {
    /// Use exact version
    Exact,
    /// Allow minor updates
    #[default]
    Minor,
    /// Allow major updates
    Major,
    /// Always use latest
    Latest,
}

/// Dependency sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencySyncConfig {
    /// Auto-install missing dependencies
    pub auto_install: bool,
    /// Confirm before installing
    pub confirm_install: bool,
}

impl Default for DependencySyncConfig {
    fn default() -> Self {
        Self {
            auto_install: true,
            confirm_install: true,
        }
    }
}

/// Sync manager for coordinating configuration sync
pub struct SyncManager {
    config: SyncConfig,
}

impl SyncManager {
    /// Create a new sync manager with default config
    pub fn new() -> Self {
        Self {
            config: SyncConfig::default(),
        }
    }

    /// Create a sync manager with custom config
    pub fn with_config(config: SyncConfig) -> Self {
        Self { config }
    }

    /// Generate sync actions from analysis
    pub fn generate_actions(
        &self,
        analysis: &ProjectAnalysis,
        existing_config: Option<&VxConfigSnapshot>,
    ) -> Vec<SyncAction> {
        let mut actions = Vec::new();

        if !self.config.enabled {
            return actions;
        }

        // Generate tool actions
        if self.config.tools.auto_detect {
            for tool in &analysis.required_tools {
                let version = tool.version.clone().unwrap_or_else(|| "latest".to_string());

                if let Some(existing) = existing_config {
                    if let Some(existing_version) = existing.tools.get(&tool.name) {
                        if existing_version != &version {
                            actions.push(SyncAction::UpdateTool {
                                name: tool.name.clone(),
                                old_version: existing_version.clone(),
                                new_version: version,
                            });
                        }
                    } else {
                        actions.push(SyncAction::AddTool {
                            name: tool.name.clone(),
                            version,
                        });
                    }
                } else {
                    actions.push(SyncAction::AddTool {
                        name: tool.name.clone(),
                        version,
                    });
                }
            }
        }

        // Generate script actions
        if self.config.scripts.import_from_project {
            for script in &analysis.scripts {
                let script_name = if self.config.scripts.prefix.is_empty() {
                    script.name.clone()
                } else {
                    format!("{}:{}", self.config.scripts.prefix, script.name)
                };

                if let Some(existing) = existing_config {
                    if let Some(existing_cmd) = existing.scripts.get(&script_name) {
                        if self.config.scripts.overwrite_existing && existing_cmd != &script.command
                        {
                            actions.push(SyncAction::UpdateScript {
                                name: script_name,
                                old_command: existing_cmd.clone(),
                                new_command: script.command.clone(),
                            });
                        }
                    } else {
                        actions.push(SyncAction::AddScript {
                            name: script_name,
                            command: script.command.clone(),
                        });
                    }
                } else {
                    actions.push(SyncAction::AddScript {
                        name: script_name,
                        command: script.command.clone(),
                    });
                }
            }
        }

        // Generate dependency install actions
        if self.config.dependencies.auto_install {
            for dep in analysis.missing_dependencies() {
                if let Some(install_cmd) = dep_install_command(dep) {
                    actions.push(SyncAction::InstallDependency {
                        command: install_cmd,
                        description: format!("Install {} ({})", dep.name, dep.ecosystem),
                    });
                }
            }
        }

        actions
    }

    /// Apply sync actions to vx.toml
    pub async fn apply_actions(
        &self,
        root: &Path,
        actions: &[SyncAction],
        dry_run: bool,
    ) -> AnalyzerResult<ApplyResult> {
        // Prefer existing config file, otherwise use new format
        let vx_config_path = if root.join(CONFIG_FILE_NAME_LEGACY).exists() {
            root.join(CONFIG_FILE_NAME_LEGACY)
        } else {
            root.join(CONFIG_FILE_NAME)
        };
        let mut result = ApplyResult::default();

        if dry_run {
            for action in actions {
                info!("[DRY RUN] Would apply: {}", action);
                result.would_apply.push(action.clone());
            }
            return Ok(result);
        }

        // Load existing config or create new
        let config_content = if vx_config_path.exists() {
            tokio::fs::read_to_string(&vx_config_path).await?
        } else {
            String::new()
        };

        let mut doc: toml::Value = if config_content.is_empty() {
            toml::Value::Table(toml::map::Map::new())
        } else {
            toml::from_str(&config_content)?
        };

        for action in actions {
            match action {
                SyncAction::AddTool { name, version }
                | SyncAction::UpdateTool {
                    name,
                    new_version: version,
                    ..
                } => {
                    ensure_table(&mut doc, "tools");
                    if let Some(tools) = doc.get_mut("tools").and_then(|t| t.as_table_mut()) {
                        tools.insert(name.clone(), toml::Value::String(version.clone()));
                        result.applied.push(action.clone());
                    }
                }
                SyncAction::AddScript { name, command }
                | SyncAction::UpdateScript {
                    name,
                    new_command: command,
                    ..
                } => {
                    ensure_table(&mut doc, "scripts");
                    if let Some(scripts) = doc.get_mut("scripts").and_then(|t| t.as_table_mut()) {
                        scripts.insert(name.clone(), toml::Value::String(command.clone()));
                        result.applied.push(action.clone());
                    }
                }
                SyncAction::InstallDependency { command, .. } => {
                    // These are executed separately, not written to config
                    result.install_commands.push(command.clone());
                }
                SyncAction::AddProjectDependency { .. } => {
                    // These modify project files, not vx.toml
                    result.skipped.push(action.clone());
                }
            }
        }

        // Write updated config
        let new_content = toml::to_string_pretty(&doc)?;
        tokio::fs::write(&vx_config_path, new_content).await?;

        Ok(result)
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of applying sync actions
#[derive(Debug, Default)]
pub struct ApplyResult {
    /// Actions that were applied
    pub applied: Vec<SyncAction>,
    /// Actions that would be applied (dry run)
    pub would_apply: Vec<SyncAction>,
    /// Actions that were skipped
    pub skipped: Vec<SyncAction>,
    /// Install commands to run
    pub install_commands: Vec<String>,
}

/// Snapshot of existing vx.toml for comparison
#[derive(Debug, Default)]
pub struct VxConfigSnapshot {
    pub tools: HashMap<String, String>,
    pub scripts: HashMap<String, String>,
}

impl VxConfigSnapshot {
    /// Load from vx.toml file
    pub async fn load(path: &Path) -> AnalyzerResult<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(path).await?;
        let doc: toml::Value = toml::from_str(&content)?;

        let mut snapshot = Self::default();

        // Extract tools
        if let Some(tools) = doc.get("tools").and_then(|t| t.as_table()) {
            for (name, value) in tools {
                if let Some(version) = value.as_str() {
                    snapshot.tools.insert(name.clone(), version.to_string());
                }
            }
        }

        // Extract scripts
        if let Some(scripts) = doc.get("scripts").and_then(|t| t.as_table()) {
            for (name, value) in scripts {
                if let Some(cmd) = value.as_str() {
                    snapshot.scripts.insert(name.clone(), cmd.to_string());
                }
            }
        }

        Ok(Some(snapshot))
    }
}

/// Helper to ensure a table exists in the TOML document
fn ensure_table(doc: &mut toml::Value, key: &str) {
    if let Some(table) = doc.as_table_mut()
        && !table.contains_key(key)
    {
        table.insert(key.to_string(), toml::Value::Table(toml::map::Map::new()));
    }
}

/// Generate install command for a dependency
fn dep_install_command(dep: &crate::dependency::Dependency) -> Option<String> {
    use crate::ecosystem::Ecosystem;

    match dep.ecosystem {
        Ecosystem::Python => {
            if dep.is_dev {
                Some(format!("uv add --group dev {}", dep.name))
            } else {
                Some(format!("uv add {}", dep.name))
            }
        }
        Ecosystem::NodeJs => {
            if dep.is_dev {
                Some(format!("npm install --save-dev {}", dep.name))
            } else {
                Some(format!("npm install {}", dep.name))
            }
        }
        Ecosystem::Rust => Some(format!("cargo add {}", dep.name)),
        Ecosystem::Go => Some(format!("go get {}", dep.name)),
        _ => None,
    }
}
