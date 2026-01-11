//! Configuration migration utilities
//!
//! This module provides tools for migrating `vx.toml` configurations
//! from v1 format to v2 format.
//!
//! ## Migration Process
//!
//! 1. Detect current config version
//! 2. Parse the existing configuration
//! 3. Transform to v2 format
//! 4. Validate the result
//! 5. Write back (with optional backup)
//!
//! ## Example
//!
//! ```ignore
//! use vx_config::migration::{ConfigMigrator, MigrationOptions};
//!
//! let migrator = ConfigMigrator::new();
//! let result = migrator.migrate_file("path/to/vx.toml", &MigrationOptions::default())?;
//! ```

use crate::config_manager::{escape_toml_key, escape_toml_string};
use crate::error::{ConfigError, ConfigResult};
use crate::types::*;
use crate::validation::validate_config;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Configuration version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigVersion {
    /// Version 1 (simple key-value tools)
    V1,
    /// Version 2 (structured configuration)
    V2,
    /// Unknown or invalid
    Unknown,
}

impl std::fmt::Display for ConfigVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigVersion::V1 => write!(f, "v1"),
            ConfigVersion::V2 => write!(f, "v2"),
            ConfigVersion::Unknown => write!(f, "unknown"),
        }
    }
}

/// Migration options
#[derive(Debug, Clone)]
pub struct MigrationOptions {
    /// Create backup before migration
    pub backup: bool,
    /// Force migration even if already v2
    pub force: bool,
    /// Dry run (don't write changes)
    pub dry_run: bool,
    /// Add comments to migrated config
    pub add_comments: bool,
}

impl Default for MigrationOptions {
    fn default() -> Self {
        Self {
            backup: true,
            force: false,
            dry_run: false,
            add_comments: true,
        }
    }
}

/// Migration result
#[derive(Debug)]
pub struct MigrationResult {
    /// Whether migration was performed
    pub migrated: bool,
    /// Source version
    pub from_version: ConfigVersion,
    /// Target version
    pub to_version: ConfigVersion,
    /// Backup file path (if created)
    pub backup_path: Option<String>,
    /// Migrated content
    pub content: String,
    /// Warnings during migration
    pub warnings: Vec<String>,
    /// Changes made
    pub changes: Vec<String>,
}

/// Configuration migrator
pub struct ConfigMigrator;

impl ConfigMigrator {
    /// Create a new migrator
    pub fn new() -> Self {
        Self
    }

    /// Detect the version of a configuration file
    pub fn detect_version(&self, content: &str) -> ConfigVersion {
        // Parse as TOML to inspect structure
        let table: Result<toml::Table, _> = toml::from_str(content);
        let table = match table {
            Ok(t) => t,
            Err(_) => return ConfigVersion::Unknown,
        };

        // Check for v2 indicators
        let has_v2_fields = table.contains_key("min_version")
            || table.contains_key("project")
            || table.contains_key("hooks")
            || table.contains_key("services")
            || table.contains_key("dependencies")
            || table.contains_key("ai")
            || table.contains_key("security")
            || table.contains_key("container");

        // Check tools structure
        let has_detailed_tools = if let Some(tools) = table.get("tools") {
            if let Some(tools_table) = tools.as_table() {
                tools_table.values().any(|v| v.is_table())
            } else {
                false
            }
        } else {
            false
        };

        // Check scripts structure
        let has_detailed_scripts = if let Some(scripts) = table.get("scripts") {
            if let Some(scripts_table) = scripts.as_table() {
                scripts_table.values().any(|v| v.is_table())
            } else {
                false
            }
        } else {
            false
        };

        if has_v2_fields || has_detailed_tools || has_detailed_scripts {
            ConfigVersion::V2
        } else if table.contains_key("tools") || table.contains_key("scripts") {
            ConfigVersion::V1
        } else {
            ConfigVersion::Unknown
        }
    }

    /// Migrate a configuration file
    pub fn migrate_file<P: AsRef<Path>>(
        &self,
        path: P,
        options: &MigrationOptions,
    ) -> ConfigResult<MigrationResult> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;

        let version = self.detect_version(&content);

        // Check if migration is needed
        if version == ConfigVersion::V2 && !options.force {
            return Ok(MigrationResult {
                migrated: false,
                from_version: version,
                to_version: ConfigVersion::V2,
                backup_path: None,
                content,
                warnings: vec!["Configuration is already v2 format".to_string()],
                changes: vec![],
            });
        }

        if version == ConfigVersion::Unknown {
            return Err(ConfigError::Validation {
                message: "Cannot detect configuration version".to_string(),
            });
        }

        // Perform migration
        let result = self.migrate_content(&content, options)?;

        // Create backup if requested
        let backup_path = if options.backup && !options.dry_run {
            let backup = format!("{}.bak", path.display());
            fs::copy(path, &backup)?;
            Some(backup)
        } else {
            None
        };

        // Write migrated content
        if !options.dry_run {
            fs::write(path, &result.content)?;
        }

        Ok(MigrationResult {
            backup_path,
            ..result
        })
    }

    /// Migrate configuration content
    pub fn migrate_content(
        &self,
        content: &str,
        options: &MigrationOptions,
    ) -> ConfigResult<MigrationResult> {
        let version = self.detect_version(content);
        let mut warnings = Vec::new();
        let mut changes = Vec::new();

        // Parse the content
        let table: toml::Table = toml::from_str(content)?;

        // Build v2 config
        let mut v2_config = VxConfig::default();

        // Migrate tools
        if let Some(tools) = table.get("tools") {
            if let Some(tools_table) = tools.as_table() {
                for (name, value) in tools_table {
                    let tool_version = if let Some(s) = value.as_str() {
                        // Simple string version - convert to detailed if needed
                        changes.push(format!("tools.{}: kept as simple version", name));
                        ToolVersion::Simple(s.to_string())
                    } else if let Some(t) = value.as_table() {
                        // Already detailed
                        let version = t
                            .get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("latest")
                            .to_string();
                        let postinstall = t
                            .get("postinstall")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let os = t.get("os").and_then(|v| {
                            v.as_array().map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                        });

                        changes.push(format!("tools.{}: migrated detailed config", name));
                        ToolVersion::Detailed(ToolConfig {
                            version,
                            postinstall,
                            os,
                            install_env: None,
                        })
                    } else {
                        warnings.push(format!("tools.{}: invalid value, skipped", name));
                        continue;
                    };
                    v2_config.tools.insert(name.clone(), tool_version);
                }
            }
        }

        // Migrate settings
        if let Some(settings) = table.get("settings") {
            if let Some(settings_table) = settings.as_table() {
                let mut settings_config = SettingsConfig::default();

                if let Some(v) = settings_table.get("auto_install") {
                    if let Some(b) = v.as_bool() {
                        settings_config.auto_install = Some(b);
                    } else if let Some(s) = v.as_str() {
                        settings_config.auto_install = Some(s == "true");
                    }
                }

                if let Some(v) = settings_table.get("parallel_install") {
                    if let Some(b) = v.as_bool() {
                        settings_config.parallel_install = Some(b);
                    }
                }

                if let Some(v) = settings_table.get("cache_duration") {
                    if let Some(s) = v.as_str() {
                        settings_config.cache_duration = Some(s.to_string());
                    }
                }

                if let Some(v) = settings_table.get("shell") {
                    if let Some(s) = v.as_str() {
                        settings_config.shell = Some(s.to_string());
                    }
                }

                if let Some(v) = settings_table.get("log_level") {
                    if let Some(s) = v.as_str() {
                        settings_config.log_level = Some(s.to_string());
                    }
                }

                v2_config.settings = Some(settings_config);
                changes.push("settings: migrated to structured format".to_string());
            }
        }

        // Migrate env
        if let Some(env) = table.get("env") {
            if let Some(env_table) = env.as_table() {
                let mut vars = HashMap::new();
                for (key, value) in env_table {
                    if let Some(s) = value.as_str() {
                        vars.insert(key.clone(), s.to_string());
                    }
                }
                if !vars.is_empty() {
                    v2_config.env = Some(EnvConfig {
                        vars,
                        required: None,
                        optional: None,
                        secrets: None,
                    });
                    changes.push("env: migrated environment variables".to_string());
                }
            }
        }

        // Migrate scripts
        if let Some(scripts) = table.get("scripts") {
            if let Some(scripts_table) = scripts.as_table() {
                for (name, value) in scripts_table {
                    let script_config = if let Some(s) = value.as_str() {
                        changes.push(format!("scripts.{}: kept as simple command", name));
                        ScriptConfig::Simple(s.to_string())
                    } else if let Some(t) = value.as_table() {
                        let command = t
                            .get("command")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let description = t
                            .get("description")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let cwd = t.get("cwd").and_then(|v| v.as_str()).map(|s| s.to_string());

                        changes.push(format!("scripts.{}: migrated detailed config", name));
                        ScriptConfig::Detailed(ScriptDetails {
                            command,
                            description,
                            cwd,
                            args: vec![],
                            env: HashMap::new(),
                            depends: vec![],
                        })
                    } else {
                        warnings.push(format!("scripts.{}: invalid value, skipped", name));
                        continue;
                    };
                    v2_config.scripts.insert(name.clone(), script_config);
                }
            }
        }

        // Preserve existing v2 fields if present
        if let Some(hooks) = table.get("hooks") {
            if let Ok(h) =
                toml::from_str::<HooksConfig>(&toml::to_string(hooks).unwrap_or_default())
            {
                v2_config.hooks = Some(h);
            }
        }

        if let Some(services) = table.get("services") {
            if let Some(services_table) = services.as_table() {
                for (name, value) in services_table {
                    if let Ok(s) =
                        toml::from_str::<ServiceConfig>(&toml::to_string(value).unwrap_or_default())
                    {
                        v2_config.services.insert(name.clone(), s);
                    }
                }
            }
        }

        if let Some(project) = table.get("project") {
            if let Ok(p) =
                toml::from_str::<ProjectConfig>(&toml::to_string(project).unwrap_or_default())
            {
                v2_config.project = Some(p);
            }
        }

        if let Some(min_version) = table.get("min_version") {
            if let Some(s) = min_version.as_str() {
                v2_config.min_version = Some(s.to_string());
            }
        }

        // Validate the migrated config
        let validation = validate_config(&v2_config);
        for error in &validation.errors {
            warnings.push(format!("Validation error: {}", error));
        }
        for warning in &validation.warnings {
            warnings.push(format!("Validation warning: {}", warning));
        }

        // Generate output
        let output = self.generate_toml(&v2_config, options)?;

        Ok(MigrationResult {
            migrated: true,
            from_version: version,
            to_version: ConfigVersion::V2,
            backup_path: None,
            content: output,
            warnings,
            changes,
        })
    }

    /// Generate TOML output from config
    fn generate_toml(&self, config: &VxConfig, options: &MigrationOptions) -> ConfigResult<String> {
        let mut output = String::new();

        // Add header comment
        if options.add_comments {
            output.push_str("# VX Project Configuration (v2)\n");
            output.push_str("# This file defines the tools and environment for this project.\n");
            output.push_str("# Run 'vx setup' to install all required tools.\n");
            output.push_str("# Documentation: https://github.com/loonghao/vx/docs/config\n");
            output.push('\n');
        }

        // min_version
        if let Some(min_version) = &config.min_version {
            if options.add_comments {
                output.push_str("# Minimum vx version required\n");
            }
            output.push_str(&format!("min_version = \"{}\"\n\n", min_version));
        }

        // project
        if let Some(project) = &config.project {
            if options.add_comments {
                output.push_str("# Project metadata\n");
            }
            output.push_str("[project]\n");
            if let Some(name) = &project.name {
                output.push_str(&format!("name = \"{}\"\n", name));
            }
            if let Some(description) = &project.description {
                output.push_str(&format!("description = \"{}\"\n", description));
            }
            if let Some(version) = &project.version {
                output.push_str(&format!("version = \"{}\"\n", version));
            }
            output.push('\n');
        }

        // tools
        if !config.tools.is_empty() {
            if options.add_comments {
                output.push_str("# Tool versions\n");
            }
            output.push_str("[tools]\n");
            for (name, version) in &config.tools {
                match version {
                    ToolVersion::Simple(v) => {
                        output.push_str(&format!("{} = \"{}\"\n", name, v));
                    }
                    ToolVersion::Detailed(d) => {
                        output.push_str(&format!("{} = {{ version = \"{}\"", name, d.version));
                        if let Some(postinstall) = &d.postinstall {
                            output.push_str(&format!(", postinstall = \"{}\"", postinstall));
                        }
                        if let Some(os) = &d.os {
                            let os_str: Vec<_> = os.iter().map(|s| format!("\"{}\"", s)).collect();
                            output.push_str(&format!(", os = [{}]", os_str.join(", ")));
                        }
                        output.push_str(" }\n");
                    }
                }
            }
            output.push('\n');
        }

        // settings
        if let Some(settings) = &config.settings {
            if options.add_comments {
                output.push_str("# Behavior settings\n");
            }
            output.push_str("[settings]\n");
            if let Some(auto_install) = settings.auto_install {
                output.push_str(&format!("auto_install = {}\n", auto_install));
            }
            if let Some(parallel) = settings.parallel_install {
                output.push_str(&format!("parallel_install = {}\n", parallel));
            }
            if let Some(cache) = &settings.cache_duration {
                output.push_str(&format!("cache_duration = \"{}\"\n", cache));
            }
            if let Some(shell) = &settings.shell {
                output.push_str(&format!("shell = \"{}\"\n", shell));
            }
            output.push('\n');
        }

        // env
        if let Some(env) = &config.env {
            if !env.vars.is_empty() {
                if options.add_comments {
                    output.push_str("# Environment variables\n");
                }
                output.push_str("[env]\n");
                for (key, value) in &env.vars {
                    output.push_str(&format!("{} = \"{}\"\n", key, value));
                }
                output.push('\n');
            }
        }

        // scripts
        if !config.scripts.is_empty() {
            if options.add_comments {
                output.push_str("# Script definitions\n");
            }
            output.push_str("[scripts]\n");
            for (name, script) in &config.scripts {
                let escaped_name = escape_toml_key(name);
                match script {
                    ScriptConfig::Simple(cmd) => {
                        output.push_str(&format!(
                            "{} = \"{}\"\n",
                            escaped_name,
                            escape_toml_string(cmd)
                        ));
                    }
                    ScriptConfig::Detailed(d) => {
                        output.push_str(&format!("[scripts.{}]\n", escaped_name));
                        output.push_str(&format!(
                            "command = \"{}\"\n",
                            escape_toml_string(&d.command)
                        ));
                        if let Some(desc) = &d.description {
                            output.push_str(&format!(
                                "description = \"{}\"\n",
                                escape_toml_string(desc)
                            ));
                        }
                        if let Some(cwd) = &d.cwd {
                            output.push_str(&format!("cwd = \"{}\"\n", escape_toml_string(cwd)));
                        }
                    }
                }
            }
            output.push('\n');
        }

        // hooks
        if let Some(hooks) = &config.hooks {
            if options.add_comments {
                output.push_str("# Lifecycle hooks\n");
            }
            output.push_str("[hooks]\n");
            if let Some(pre) = &hooks.pre_setup {
                match pre {
                    HookCommand::Single(cmd) => {
                        output.push_str(&format!("pre_setup = \"{}\"\n", cmd));
                    }
                    HookCommand::Multiple(cmds) => {
                        let cmds_str: Vec<_> = cmds.iter().map(|s| format!("\"{}\"", s)).collect();
                        output.push_str(&format!("pre_setup = [{}]\n", cmds_str.join(", ")));
                    }
                }
            }
            if let Some(post) = &hooks.post_setup {
                match post {
                    HookCommand::Single(cmd) => {
                        output.push_str(&format!("post_setup = \"{}\"\n", cmd));
                    }
                    HookCommand::Multiple(cmds) => {
                        let cmds_str: Vec<_> = cmds.iter().map(|s| format!("\"{}\"", s)).collect();
                        output.push_str(&format!("post_setup = [{}]\n", cmds_str.join(", ")));
                    }
                }
            }
            output.push('\n');
        }

        // services
        if !config.services.is_empty() {
            if options.add_comments {
                output.push_str("# Development services (Docker/Podman)\n");
            }
            for (name, service) in &config.services {
                output.push_str(&format!("[services.{}]\n", name));
                if let Some(image) = &service.image {
                    output.push_str(&format!("image = \"{}\"\n", image));
                }
                if !service.ports.is_empty() {
                    let ports_str: Vec<_> =
                        service.ports.iter().map(|s| format!("\"{}\"", s)).collect();
                    output.push_str(&format!("ports = [{}]\n", ports_str.join(", ")));
                }
                if !service.env.is_empty() {
                    output.push_str("env = { ");
                    let env_str: Vec<_> = service
                        .env
                        .iter()
                        .map(|(k, v)| format!("{} = \"{}\"", k, v))
                        .collect();
                    output.push_str(&env_str.join(", "));
                    output.push_str(" }\n");
                }
                if let Some(healthcheck) = &service.healthcheck {
                    output.push_str(&format!("healthcheck = \"{}\"\n", healthcheck));
                }
                output.push('\n');
            }
        }

        Ok(output)
    }
}

impl Default for ConfigMigrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_version_v1() {
        let migrator = ConfigMigrator::new();

        let v1_config = r#"
[tools]
node = "20"
python = "3.11"

[settings]
auto_install = true

[scripts]
test = "npm test"
"#;

        assert_eq!(migrator.detect_version(v1_config), ConfigVersion::V1);
    }

    #[test]
    fn test_detect_version_v2() {
        let migrator = ConfigMigrator::new();

        let v2_config = r#"
min_version = "0.6.0"

[project]
name = "my-app"

[tools]
node = { version = "20", postinstall = "npm install" }

[hooks]
pre_setup = "echo hello"
"#;

        assert_eq!(migrator.detect_version(v2_config), ConfigVersion::V2);
    }

    #[test]
    fn test_migrate_v1_to_v2() {
        let migrator = ConfigMigrator::new();

        let v1_config = r#"
[tools]
node = "20"
python = "3.11"

[settings]
auto_install = true
cache_duration = "7d"

[env]
NODE_ENV = "development"

[scripts]
test = "npm test"
build = "npm run build"
"#;

        let options = MigrationOptions {
            add_comments: false,
            ..Default::default()
        };

        let result = migrator.migrate_content(v1_config, &options).unwrap();

        assert!(result.migrated);
        assert_eq!(result.from_version, ConfigVersion::V1);
        assert_eq!(result.to_version, ConfigVersion::V2);
        assert!(result.content.contains("[tools]"));
        assert!(result.content.contains("node = \"20\""));
        assert!(result.content.contains("[settings]"));
        assert!(result.content.contains("auto_install = true"));
    }

    #[test]
    fn test_migrate_preserves_v2_fields() {
        let migrator = ConfigMigrator::new();

        let config = r#"
[tools]
node = "20"

[hooks]
pre_setup = "echo hello"
post_setup = ["npm install", "npm run build"]

[services.postgres]
image = "postgres:15"
ports = ["5432:5432"]
"#;

        let options = MigrationOptions {
            add_comments: false,
            force: true,
            ..Default::default()
        };

        let result = migrator.migrate_content(config, &options).unwrap();

        assert!(result.content.contains("[hooks]"));
        assert!(result.content.contains("pre_setup"));
        assert!(result.content.contains("[services.postgres]"));
    }

    #[test]
    fn test_migration_result_changes() {
        let migrator = ConfigMigrator::new();

        let v1_config = r#"
[tools]
node = "20"

[scripts]
test = "npm test"
"#;

        let options = MigrationOptions::default();
        let result = migrator.migrate_content(v1_config, &options).unwrap();

        assert!(!result.changes.is_empty());
        assert!(result.changes.iter().any(|c| c.contains("tools.node")));
        assert!(result.changes.iter().any(|c| c.contains("scripts.test")));
    }
}
