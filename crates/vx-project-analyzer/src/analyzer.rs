//! Core project analyzer

use crate::ecosystem::Ecosystem;
use crate::error::{AnalyzerError, AnalyzerResult};
use crate::languages::{all_analyzers, LanguageAnalyzer};
use crate::sync::{SyncManager, VxConfigSnapshot};
use crate::types::ProjectAnalysis;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info};

/// Configuration for the project analyzer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzerConfig {
    /// Whether to check if dependencies are installed
    pub check_installed: bool,

    /// Whether to check if tools are available
    pub check_tools: bool,

    /// Whether to generate sync actions
    pub generate_sync_actions: bool,

    /// Maximum depth to search for project files
    pub max_depth: usize,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            check_installed: true,
            check_tools: true,
            generate_sync_actions: true,
            max_depth: 3,
        }
    }
}

/// Project analyzer for detecting dependencies, scripts, and tools
pub struct ProjectAnalyzer {
    config: AnalyzerConfig,
    analyzers: Vec<Box<dyn LanguageAnalyzer>>,
    sync_manager: SyncManager,
}

impl ProjectAnalyzer {
    /// Create a new project analyzer with default config
    pub fn new(config: AnalyzerConfig) -> Self {
        Self {
            config,
            analyzers: all_analyzers(),
            sync_manager: SyncManager::new(),
        }
    }

    /// Analyze a project directory
    pub async fn analyze(&self, root: &Path) -> AnalyzerResult<ProjectAnalysis> {
        if !root.exists() {
            return Err(AnalyzerError::ProjectNotFound {
                path: root.to_path_buf(),
            });
        }

        let root = root.canonicalize()?;
        info!("Analyzing project at: {}", root.display());

        let mut analysis = ProjectAnalysis::new(root.clone());

        // Detect ecosystems and run language-specific analyzers
        for analyzer in &self.analyzers {
            if analyzer.detect(&root) {
                debug!("Detected {} project", analyzer.name());

                let ecosystem = match analyzer.name() {
                    "Python" => Ecosystem::Python,
                    "Node.js" => Ecosystem::NodeJs,
                    "Rust" => Ecosystem::Rust,
                    "Go" => Ecosystem::Go,
                    _ => Ecosystem::Unknown,
                };

                if !analysis.ecosystems.contains(&ecosystem) {
                    analysis.ecosystems.push(ecosystem);
                }

                // Analyze dependencies
                match analyzer.analyze_dependencies(&root).await {
                    Ok(deps) => {
                        debug!("Found {} dependencies", deps.len());
                        analysis.dependencies.extend(deps);
                    }
                    Err(e) => {
                        debug!("Failed to analyze dependencies: {}", e);
                    }
                }

                // Analyze scripts
                match analyzer.analyze_scripts(&root).await {
                    Ok(scripts) => {
                        debug!("Found {} scripts", scripts.len());
                        analysis.scripts.extend(scripts);
                    }
                    Err(e) => {
                        debug!("Failed to analyze scripts: {}", e);
                    }
                }

                // Get required tools
                let tools = analyzer.required_tools(&analysis.dependencies, &analysis.scripts);
                debug!(
                    "Required tools: {:?}",
                    tools.iter().map(|t| &t.name).collect::<Vec<_>>()
                );
                analysis.required_tools.extend(tools);
            }
        }

        // Check tool availability
        if self.config.check_tools {
            self.check_tool_availability(&mut analysis).await;
        }

        // Generate sync actions
        if self.config.generate_sync_actions {
            let vx_config_path = root.join(".vx.toml");
            let existing = VxConfigSnapshot::load(&vx_config_path).await?;
            analysis.sync_actions = self
                .sync_manager
                .generate_actions(&analysis, existing.as_ref());
        }

        Ok(analysis)
    }

    /// Check if required tools are available
    async fn check_tool_availability(&self, analysis: &mut ProjectAnalysis) {
        for tool in &mut analysis.required_tools {
            tool.is_available = is_tool_available(&tool.name).await;
        }

        // Also check tools in scripts
        for script in &mut analysis.scripts {
            for tool in &mut script.tools {
                tool.is_available = is_tool_available(&tool.name).await;
            }
        }
    }

    /// Get the sync manager
    pub fn sync_manager(&self) -> &SyncManager {
        &self.sync_manager
    }
}

/// Check if a tool is available in PATH or via vx
async fn is_tool_available(name: &str) -> bool {
    // First check PATH
    if which::which(name).is_ok() {
        return true;
    }

    // Check if it's a vx-managed tool
    // This would need integration with vx-paths to check installed tools
    // For now, just check PATH
    false
}

impl Default for ProjectAnalyzer {
    fn default() -> Self {
        Self::new(AnalyzerConfig::default())
    }
}
