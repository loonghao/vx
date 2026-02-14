//! Core project analyzer

use crate::common::JustfileAnalyzer;
use crate::dependency::InstallMethod;
use crate::ecosystem::Ecosystem;
use crate::error::{AnalyzerError, AnalyzerResult};
use crate::frameworks::{FrameworkDetector, all_framework_detectors};
use crate::languages::{LanguageAnalyzer, all_analyzers};
use crate::sync::{SyncManager, VxConfigSnapshot};
use crate::types::{AuditFinding, AuditSeverity, ProjectAnalysis, RequiredTool, Script};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use vx_paths::project::{CONFIG_FILE_NAME, CONFIG_FILE_NAME_LEGACY};

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
    framework_detectors: Vec<Box<dyn FrameworkDetector>>,
    justfile_analyzer: JustfileAnalyzer,
    sync_manager: SyncManager,
}

impl ProjectAnalyzer {
    /// Create a new project analyzer with default config
    pub fn new(config: AnalyzerConfig) -> Self {
        Self {
            config,
            analyzers: all_analyzers(),
            framework_detectors: all_framework_detectors(),
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
                        ".NET/C#" => Ecosystem::DotNet,
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

        // Detect application frameworks (Electron, Tauri, etc.)
        for detector in &self.framework_detectors {
            if detector.detect(&root) {
                debug!("Detected {} framework", detector.framework());

                // Get framework info
                match detector.get_info(&root).await {
                    Ok(info) => {
                        debug!("Framework info: {:?}", info);
                        analysis.frameworks.push(info);
                    }
                    Err(e) => {
                        debug!("Failed to get framework info: {}", e);
                    }
                }

                // Get framework-specific required tools
                let tools = detector.required_tools(&analysis.dependencies, &analysis.scripts);
                debug!(
                    "Framework required tools: {:?}",
                    tools.iter().map(|t| &t.name).collect::<Vec<_>>()
                );
                analysis.required_tools.extend(tools);

                // Get additional scripts from framework
                match detector.additional_scripts(&root).await {
                    Ok(scripts) => {
                        debug!("Found {} framework-specific scripts", scripts.len());
                        analysis.scripts.extend(scripts);
                    }
                    Err(e) => {
                        debug!("Failed to get framework scripts: {}", e);
                    }
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
            // Prefer existing config file, otherwise use new format
            let vx_config_path = if root.join(CONFIG_FILE_NAME_LEGACY).exists() {
                root.join(CONFIG_FILE_NAME_LEGACY)
            } else {
                root.join(CONFIG_FILE_NAME)
            };
            let existing = VxConfigSnapshot::load(&vx_config_path).await?;
            analysis.sync_actions = self
                .sync_manager
                .generate_actions(&analysis, existing.as_ref());
        }

        // Run audit checks
        self.run_audit_checks(&root, &mut analysis).await;

        Ok(analysis)
    }

    /// Run audit checks on the project
    async fn run_audit_checks(&self, root: &Path, analysis: &mut ProjectAnalysis) {
        // Check for missing lockfiles
        self.audit_missing_lockfiles(root, analysis).await;

        // Check for unpinned dependencies
        self.audit_unpinned_dependencies(analysis);

        // Check for mixed ecosystems
        self.audit_mixed_ecosystems(analysis);
    }

    /// Audit for missing lockfiles when dependencies exist
    async fn audit_missing_lockfiles(&self, root: &Path, analysis: &mut ProjectAnalysis) {
        // Node.js: has package.json with dependencies but no lockfile
        if root.join("package.json").exists() {
            let has_deps = analysis
                .dependencies
                .iter()
                .any(|d| d.ecosystem == Ecosystem::NodeJs);

            if has_deps {
                let has_lockfile = root.join("package-lock.json").exists()
                    || root.join("yarn.lock").exists()
                    || root.join("pnpm-lock.yaml").exists()
                    || root.join("bun.lockb").exists();

                if !has_lockfile {
                    analysis.audit_findings.push(
                        AuditFinding::new(
                            AuditSeverity::Warning,
                            "Missing lockfile for Node.js project",
                            "Project has dependencies but no lockfile (package-lock.json, yarn.lock, pnpm-lock.yaml, or bun.lockb). This can lead to inconsistent builds.",
                        )
                        .with_suggestion("Run 'npm install', 'yarn', 'pnpm install', or 'bun install' to generate a lockfile")
                        .with_file(root.join("package.json")),
                    );
                }
            }
        }

        // Python: has pyproject.toml with dependencies but no lockfile
        if root.join("pyproject.toml").exists() {
            let has_deps = analysis
                .dependencies
                .iter()
                .any(|d| d.ecosystem == Ecosystem::Python);

            if has_deps {
                let has_lockfile = root.join("uv.lock").exists()
                    || root.join("poetry.lock").exists()
                    || root.join("Pipfile.lock").exists()
                    || root.join("pdm.lock").exists();

                if !has_lockfile {
                    analysis.audit_findings.push(
                        AuditFinding::new(
                            AuditSeverity::Warning,
                            "Missing lockfile for Python project",
                            "Project has dependencies but no lockfile. This can lead to inconsistent builds.",
                        )
                        .with_suggestion("Run 'uv lock' or your package manager's lock command")
                        .with_file(root.join("pyproject.toml")),
                    );
                }
            }
        }
    }

    /// Audit for unpinned dependencies (using 'latest', '*', etc.)
    fn audit_unpinned_dependencies(&self, analysis: &mut ProjectAnalysis) {
        let unpinned: Vec<_> = analysis
            .dependencies
            .iter()
            .filter(|d| {
                if let Some(ref version) = d.version {
                    let v = version.to_lowercase();
                    v == "latest" || v == "*" || v.is_empty()
                } else {
                    false
                }
            })
            .collect();

        if !unpinned.is_empty() {
            let names: Vec<_> = unpinned.iter().map(|d| d.name.as_str()).collect();
            analysis.audit_findings.push(
                AuditFinding::new(
                    AuditSeverity::Warning,
                    "Unpinned dependencies detected",
                    format!(
                        "The following dependencies use unpinned versions (latest, *, etc.): {}. This can lead to unexpected breaking changes.",
                        names.join(", ")
                    ),
                )
                .with_suggestion("Pin dependencies to specific versions or version ranges"),
            );
        }
    }

    /// Audit for mixed ecosystems in the same project
    fn audit_mixed_ecosystems(&self, analysis: &mut ProjectAnalysis) {
        // Filter out Unknown ecosystem
        let real_ecosystems: Vec<_> = analysis
            .ecosystems
            .iter()
            .filter(|e| **e != Ecosystem::Unknown)
            .collect();

        if real_ecosystems.len() > 1 {
            let ecosystem_names: Vec<_> =
                real_ecosystems.iter().map(|e| format!("{:?}", e)).collect();
            analysis.audit_findings.push(
                AuditFinding::new(
                    AuditSeverity::Info,
                    "Mixed ecosystem project detected",
                    format!(
                        "This project uses multiple ecosystems: {}. Consider using vx to manage all runtimes consistently.",
                        ecosystem_names.join(", ")
                    ),
                )
                .with_suggestion("Use 'vx sync' to ensure all required tools are available"),
            );
        }
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
            // .NET markers
            "global.json",
            "Directory.Build.props",
        ];

        // File extensions that indicate a project subdirectory (.NET uses variable-named files)
        let project_extensions = ["csproj", "fsproj", "sln"];

        // Common container directories for monorepos (packages/apps/etc.)
        let monorepo_containers = [
            "packages", "apps", "services", "examples", "modules", "libs",
        ];

        // Scan immediate subdirectories
        if let Ok(mut entries) = tokio::fs::read_dir(root).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();

                // Skip hidden directories and common non-project directories
                let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };

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

                if path.is_dir() {
                    let mut pushed = false;
                    // Check if this subdirectory contains any project markers (fixed filenames)
                    for marker in &monorepo_indicators {
                        if path.join(marker).exists() {
                            debug!("Found monorepo subdirectory: {}", path.display());
                            dirs.push(path.clone());
                            pushed = true;
                            break;
                        }
                    }

                    // Check for project files by extension (.csproj, .fsproj, .sln)
                    if !pushed && has_files_with_any_extension(&path, &project_extensions) {
                        debug!(
                            "Found project subdirectory (by extension): {}",
                            path.display()
                        );
                        dirs.push(path.clone());
                        pushed = true;
                    }

                    // If this is a common monorepo container (e.g., packages/), scan one level deeper
                    if !pushed && monorepo_containers.contains(&name) {
                        if let Ok(mut subentries) = tokio::fs::read_dir(&path).await {
                            while let Ok(Some(child)) = subentries.next_entry().await {
                                let child_path = child.path();
                                if !child_path.is_dir() {
                                    continue;
                                }
                                let mut child_pushed = false;
                                for marker in &monorepo_indicators {
                                    if child_path.join(marker).exists() {
                                        debug!(
                                            "Found monorepo subdirectory: {}",
                                            child_path.display()
                                        );
                                        dirs.push(child_path.clone());
                                        child_pushed = true;
                                        break;
                                    }
                                }
                                // Also check extensions in nested container dirs
                                if !child_pushed
                                    && has_files_with_any_extension(
                                        &child_path,
                                        &project_extensions,
                                    )
                                {
                                    debug!(
                                        "Found project subdirectory (by extension): {}",
                                        child_path.display()
                                    );
                                    dirs.push(child_path.clone());
                                }
                            }
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

/// Check if a directory contains files with any of the given extensions (non-recursive)
fn has_files_with_any_extension(dir: &Path, extensions: &[&str]) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.iter().any(|e| ext.eq_ignore_ascii_case(e)) {
                    return true;
                }
            }
        }
    }
    false
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
