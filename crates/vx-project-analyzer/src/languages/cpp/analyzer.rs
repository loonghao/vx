//! C++ project analyzer implementation

use super::dependencies::parse_cmake_dependencies;
use super::rules::CPP_RULES;
use crate::dependency::{Dependency, InstallMethod};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use crate::languages::LanguageAnalyzer;
use crate::languages::rules::apply_rules;
use crate::script_parser::ScriptParser;
use crate::types::{RequiredTool, Script};
use async_trait::async_trait;
use std::path::Path;
use tracing::debug;
use walkdir::WalkDir;

/// C++ project analyzer
pub struct CppAnalyzer {
    script_parser: ScriptParser,
}

impl CppAnalyzer {
    /// Create a new C++ analyzer
    pub fn new() -> Self {
        Self {
            script_parser: ScriptParser::new(),
        }
    }
}

impl Default for CppAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageAnalyzer for CppAnalyzer {
    fn detect(&self, root: &Path) -> bool {
        root.join("CMakeLists.txt").exists()
            || root.join("meson.build").exists()
            || (root.join("Makefile").exists() && has_cpp_sources(root))
    }

    fn name(&self) -> &'static str {
        "C++"
    }

    async fn analyze_dependencies(&self, root: &Path) -> AnalyzerResult<Vec<Dependency>> {
        let mut deps = Vec::new();

        // Find all CMakeLists.txt files (limited depth to avoid too deep recursion)
        let cmake_files: Vec<_> = WalkDir::new(root)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() == "CMakeLists.txt")
            .filter(|e| {
                // Skip build directories and other non-source directories
                let path = e.path();
                !path.components().any(|c| {
                    matches!(
                        c.as_os_str().to_str(),
                        Some("build" | "cmake-build-*" | ".git" | "node_modules")
                    )
                })
            })
            .collect();

        debug!("Found {} CMakeLists.txt files", cmake_files.len());

        for entry in cmake_files {
            let cmake_path = entry.path();
            debug!("Analyzing {}", cmake_path.display());
            if let Ok(content) = tokio::fs::read_to_string(cmake_path).await {
                if let Ok(file_deps) = parse_cmake_dependencies(&content, cmake_path) {
                    deps.extend(file_deps);
                }
            }
        }

        // Deduplicate by name
        deps.sort_by(|a, b| a.name.cmp(&b.name));
        deps.dedup_by(|a, b| a.name == b.name);

        Ok(deps)
    }

    async fn analyze_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>> {
        // Apply detection rules for common scripts
        let scripts = apply_rules(root, CPP_RULES, &self.script_parser);
        Ok(scripts)
    }

    fn required_tools(&self, _deps: &[Dependency], _scripts: &[Script]) -> Vec<RequiredTool> {
        // CMake is always needed for CMake projects
        vec![RequiredTool::new(
            "cmake",
            Ecosystem::Cpp,
            "CMake build system",
            InstallMethod::System {
                instructions: "Install via package manager (apt, brew, choco)".to_string(),
            },
        )]
    }

    fn install_command(&self, _dep: &Dependency) -> Option<String> {
        // C++ dependencies are typically managed by CMake/vcpkg/conan
        None
    }
}

/// Check if directory has C++ source files
fn has_cpp_sources(root: &Path) -> bool {
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if matches!(ext.as_str(), "cpp" | "cxx" | "cc" | "c" | "hpp" | "h") {
                    return true;
                }
            }
        }
    }
    false
}
