//! Core project analyzer

use crate::common::JustfileAnalyzer;
use crate::dependency::InstallMethod;
use crate::ecosystem::Ecosystem;
use crate::error::{AnalyzerError, AnalyzerResult};
use crate::languages::{all_analyzers, LanguageAnalyzer};
use crate::sync::{SyncManager, VxConfigSnapshot};
use crate::types::{ProjectAnalysis, RequiredTool, Script};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
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
    justfile_analyzer: JustfileAnalyzer,
    sync_manager: SyncManager,
}

impl ProjectAnalyzer {
    /// Create a new project analyzer with default config
    pub fn new(config: AnalyzerConfig) -> Self {
        Self {
            config,
            analyzers: all_analyzers(),
            justfile_analyzer: JustfileAnalyzer::new(),
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

        // Collect directories to analyze (root + immediate subdirectories for monorepo support)
        let dirs_to_analyze = self.collect_analysis_dirs(&root).await;

        // Detect ecosystems and run language-specific analyzers
        for analyzer in &self.analyzers {
            for dir in &dirs_to_analyze {
                if analyzer.detect(dir) {
                    let is_subdir = dir != &root;
                    if is_subdir {
                        debug!(
                            "Detected {} project in subdirectory: {}",
                            analyzer.name(),
                            dir.display()
                        );
                    } else {
                        debug!("Detected {} project", analyzer.name());
                    }

                    let ecosystem = match analyzer.name() {
                        "Python" => Ecosystem::Python,
                        "Node.js" => Ecosystem::NodeJs,
                        "Rust" => Ecosystem::Rust,
                        "Go" => Ecosystem::Go,
                        "C++" => Ecosystem::Cpp,
                        _ => Ecosystem::Unknown,
                    };

                    if !analysis.ecosystems.contains(&ecosystem) {
                        analysis.ecosystems.push(ecosystem);
                    }

                    // Analyze dependencies
                    match analyzer.analyze_dependencies(dir).await {
                        Ok(deps) => {
                            debug!("Found {} dependencies in {}", deps.len(), dir.display());
                            analysis.dependencies.extend(deps);
                        }
                        Err(e) => {
                            debug!("Failed to analyze dependencies in {}: {}", dir.display(), e);
                        }
                    }

                    // Analyze scripts (only from root directory to avoid duplicates)
                    if !is_subdir {
                        match analyzer.analyze_scripts(dir).await {
                            Ok(scripts) => {
                                debug!("Found {} scripts", scripts.len());
                                analysis.scripts.extend(scripts);
                            }
                            Err(e) => {
                                debug!("Failed to analyze scripts: {}", e);
                            }
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
        }

        // Run common/cross-language analyzers
        // Justfile analyzer - runs once regardless of detected languages
        if self.justfile_analyzer.detect(&root) {
            debug!("Detected justfile");
            match self.justfile_analyzer.analyze_scripts(&root).await {
                Ok(scripts) => {
                    debug!("Found {} justfile recipes", scripts.len());
                    analysis.scripts.extend(scripts);

                    // Add 'just' as a required tool
                    analysis.required_tools.push(RequiredTool::new(
                        "just",
                        Ecosystem::Unknown, // just is language-agnostic
                        "Command runner (justfile)",
                        InstallMethod::vx("just"),
                    ));
                }
                Err(e) => {
                    debug!("Failed to analyze justfile: {}", e);
                }
            }
        }

        // Deduplicate scripts - keep first occurrence (higher priority sources)
        analysis.scripts = deduplicate_scripts(analysis.scripts);

        // Deduplicate required tools
        analysis.required_tools.sort_by(|a, b| a.name.cmp(&b.name));
        analysis.required_tools.dedup_by(|a, b| a.name == b.name);

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

    /// Collect directories to analyze for monorepo support.
    ///
    /// Returns the root directory plus any immediate subdirectories that might
    /// contain language-specific project files (e.g., `codex-rs/` containing Cargo.toml).
    async fn collect_analysis_dirs(&self, root: &Path) -> Vec<PathBuf> {
        let mut dirs = vec![root.to_path_buf()];

        // Only scan subdirectories if max_depth > 1
        if self.config.max_depth <= 1 {
            return dirs;
        }

        // Common monorepo subdirectory patterns
        let monorepo_indicators = [
            // Language-specific markers
            "Cargo.toml",
            "go.mod",
            "package.json",
            "pyproject.toml",
            // Common monorepo directory names
        ];

        // Scan immediate subdirectories
        if let Ok(mut entries) = tokio::fs::read_dir(root).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();

                // Skip hidden directories and common non-project directories
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.')
                        || name == "node_modules"
                        || name == "target"
                        || name == "dist"
                        || name == "build"
                        || name == "vendor"
                        || name == "__pycache__"
                        || name == ".git"
                    {
                        continue;
                    }
                }

                if path.is_dir() {
                    // Check if this subdirectory contains any project markers
                    for marker in &monorepo_indicators {
                        if path.join(marker).exists() {
                            debug!("Found monorepo subdirectory: {}", path.display());
                            dirs.push(path.clone());
                            break;
                        }
                    }
                }
            }
        }

        dirs
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

/// Deduplicate scripts, keeping the first occurrence of each name.
///
/// This preserves priority order: explicit config > detected scripts.
fn deduplicate_scripts(scripts: Vec<Script>) -> Vec<Script> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut result = Vec::new();

    for script in scripts {
        if !seen.contains(&script.name) {
            seen.insert(script.name.clone());
            result.push(script);
        }
    }

    result
}
