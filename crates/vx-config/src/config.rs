//! Configuration layer management and figment integration

use crate::{types::*, Result};
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use std::path::PathBuf;

/// Build the complete figment with all configuration layers
pub fn build_figment(project_info: &Option<ProjectInfo>) -> Result<Figment> {
    let mut figment = Figment::new();

    // Apply configuration layers in priority order
    figment = add_builtin_defaults(figment);
    figment = add_global_user_config(figment);
    figment = add_project_config(figment, project_info)?;
    figment = add_vx_project_config(figment);
    figment = add_environment_variables(figment);

    Ok(figment)
}

/// Add built-in default configuration (lowest priority)
fn add_builtin_defaults(figment: Figment) -> Figment {
    figment.merge(Serialized::defaults(VxConfig::default()))
}

/// Add global user configuration
fn add_global_user_config(figment: Figment) -> Figment {
    if let Some(config_dir) = dirs::config_dir() {
        let global_config = config_dir.join("vx").join("config.toml");
        if global_config.exists() {
            return figment.merge(Toml::file(global_config));
        }
    }
    figment
}

/// Add project-specific tool versions from detected project files
fn add_project_config(figment: Figment, project_info: &Option<ProjectInfo>) -> Result<Figment> {
    if let Some(project_info) = project_info {
        let project_config = create_project_config_from_info(project_info)?;
        Ok(figment.merge(Serialized::defaults(project_config)))
    } else {
        Ok(figment)
    }
}

/// Add vx-specific project configuration (.vx.toml)
fn add_vx_project_config(figment: Figment) -> Figment {
    let vx_project_config = PathBuf::from(".vx.toml");
    if vx_project_config.exists() {
        if let Ok(project_config) = parse_vx_project_config(&vx_project_config) {
            return figment.merge(Serialized::defaults(project_config));
        }
    }
    figment
}

/// Add environment variables (highest priority)
fn add_environment_variables(figment: Figment) -> Figment {
    figment.merge(Env::prefixed("VX_"))
}

/// Create project configuration from detected project info
fn create_project_config_from_info(project_info: &ProjectInfo) -> Result<VxConfig> {
    let mut config = VxConfig::default();

    for (tool_name, version) in &project_info.tool_versions {
        config.tools.insert(
            tool_name.clone(),
            ToolConfig {
                version: Some(version.clone()),
                install_method: None,
                registry: None,
                custom_sources: None,
            },
        );
    }

    Ok(config)
}

/// Parse .vx.toml project configuration and convert to VxConfig format
fn parse_vx_project_config(path: &PathBuf) -> Result<VxConfig> {
    let content = std::fs::read_to_string(path)?;

    // Parse as project config first
    let project_config: ProjectConfig = toml::from_str(&content)?;

    // Convert to VxConfig format
    let mut vx_config = VxConfig::default();

    // Convert tools from simple string format to ToolConfig format
    for (tool_name, version) in project_config.tools {
        vx_config.tools.insert(
            tool_name,
            ToolConfig {
                version: Some(version),
                install_method: None,
                registry: None,
                custom_sources: None,
            },
        );
    }

    // Apply project settings to defaults
    vx_config.defaults.auto_install = project_config.settings.auto_install;

    Ok(vx_config)
}
