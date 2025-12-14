//! Standard implementations for common patterns
//!
//! This module provides ready-to-use implementations for common tool patterns.

use crate::{
    traits::tool::{UrlBuilder, VersionParser},
    Ecosystem, PackageManager, PackageSpec, Result, Tool, ToolBundle, ToolMetadata, VersionInfo,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;

/// Standard bundle implementation for single-tool bundles
pub struct StandardBundle {
    name: String,
    description: String,
    version: String,
    author: Option<String>,
    homepage: Option<String>,
    tool_factory: Box<dyn Fn() -> Box<dyn Tool> + Send + Sync>,
}

impl StandardBundle {
    /// Create a new standard bundle
    pub fn new<F>(
        name: impl Into<String>,
        description: impl Into<String>,
        version: impl Into<String>,
        tool_factory: F,
    ) -> Self
    where
        F: Fn() -> Box<dyn Tool> + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            description: description.into(),
            version: version.into(),
            author: None,
            homepage: None,
            tool_factory: Box::new(tool_factory),
        }
    }

    /// Set the bundle author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set the bundle homepage
    pub fn with_homepage(mut self, homepage: impl Into<String>) -> Self {
        self.homepage = Some(homepage.into());
        self
    }
}

#[async_trait]
impl ToolBundle for StandardBundle {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }

    fn homepage(&self) -> Option<&str> {
        self.homepage.as_deref()
    }

    fn tools(&self) -> Vec<Box<dyn Tool>> {
        vec![(self.tool_factory)()]
    }
}

/// Standard package manager implementation
pub struct StandardPackageManager {
    name: String,
    description: String,
    ecosystem: Ecosystem,
    config_files: Vec<String>,
    install_command: Vec<String>,
    remove_command: Vec<String>,
    update_command: Vec<String>,
    list_command: Vec<String>,
    search_command: Vec<String>,
}

impl StandardPackageManager {
    /// Create a new standard package manager
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        ecosystem: Ecosystem,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            ecosystem,
            config_files: Vec::new(),
            install_command: vec!["install".to_string()],
            remove_command: vec!["remove".to_string()],
            update_command: vec!["update".to_string()],
            list_command: vec!["list".to_string()],
            search_command: vec!["search".to_string()],
        }
    }

    /// Add a configuration file
    pub fn with_config_file(mut self, config_file: impl Into<String>) -> Self {
        self.config_files.push(config_file.into());
        self
    }

    /// Set custom install command
    pub fn with_install_command(mut self, command: Vec<String>) -> Self {
        self.install_command = command;
        self
    }

    /// Set custom remove command
    pub fn with_remove_command(mut self, command: Vec<String>) -> Self {
        self.remove_command = command;
        self
    }

    /// Set custom update command
    pub fn with_update_command(mut self, command: Vec<String>) -> Self {
        self.update_command = command;
        self
    }
}

#[async_trait]
impl PackageManager for StandardPackageManager {
    fn name(&self) -> &str {
        &self.name
    }

    fn ecosystem(&self) -> Ecosystem {
        self.ecosystem
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn get_config_files(&self) -> Vec<&str> {
        self.config_files.iter().map(|s| s.as_str()).collect()
    }

    async fn install_packages(&self, packages: &[PackageSpec], project_path: &Path) -> Result<()> {
        let package_names: Vec<String> = packages
            .iter()
            .map(|p| {
                if let Some(version) = &p.version {
                    format!("{}@{}", p.name, version)
                } else {
                    p.name.clone()
                }
            })
            .collect();

        let command_strs: Vec<&str> = self.install_command.iter().map(|s| s.as_str()).collect();
        self.run_command(&command_strs, &package_names, project_path)
            .await
    }

    fn get_install_command(&self) -> Vec<&str> {
        self.install_command.iter().map(|s| s.as_str()).collect()
    }

    fn get_remove_command(&self) -> Vec<&str> {
        self.remove_command.iter().map(|s| s.as_str()).collect()
    }

    fn get_update_command(&self) -> Vec<&str> {
        self.update_command.iter().map(|s| s.as_str()).collect()
    }

    fn get_list_command(&self) -> Vec<&str> {
        self.list_command.iter().map(|s| s.as_str()).collect()
    }

    fn get_search_command(&self) -> Vec<&str> {
        self.search_command.iter().map(|s| s.as_str()).collect()
    }
}

/// Configuration-driven tool implementation
pub struct ConfigurableTool {
    metadata: ToolMetadata,
    url_builder: Box<dyn UrlBuilder>,
    #[allow(dead_code)]
    version_parser: Box<dyn VersionParser>,
}

impl ConfigurableTool {
    /// Create a new configurable tool
    pub fn new(
        metadata: ToolMetadata,
        url_builder: Box<dyn UrlBuilder>,
        version_parser: Box<dyn VersionParser>,
    ) -> Self {
        Self {
            metadata,
            url_builder,
            version_parser,
        }
    }

    /// Get the tool metadata
    pub fn get_metadata(&self) -> &ToolMetadata {
        &self.metadata
    }
}

#[async_trait]
impl Tool for ConfigurableTool {
    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn description(&self) -> &str {
        &self.metadata.description
    }

    fn aliases(&self) -> Vec<&str> {
        self.metadata.aliases.iter().map(|s| s.as_str()).collect()
    }

    async fn fetch_versions(&self, _include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        // Placeholder implementation
        Ok(vec![
            VersionInfo::new("1.0.0"),
            VersionInfo::new("1.1.0"),
            VersionInfo::new("2.0.0"),
        ])
    }

    async fn get_download_url(&self, version: &str) -> Result<Option<String>> {
        Ok(self.url_builder.download_url(version))
    }

    fn metadata(&self) -> HashMap<String, String> {
        self.metadata.metadata.clone()
    }
}
