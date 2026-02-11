//! .NET/C# project analyzer implementation

use super::dependencies::{parse_csproj_dependencies, parse_directory_packages_props};
use super::rules::DOTNET_RULES;
use crate::dependency::{Dependency, InstallMethod};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use crate::languages::rules::{apply_rules, merge_scripts};
use crate::languages::LanguageAnalyzer;
use crate::script_parser::ScriptParser;
use crate::types::{RequiredTool, Script, ScriptSource};
use async_trait::async_trait;
use std::path::Path;
use tracing::debug;
use walkdir::WalkDir;

/// .NET/C# project analyzer
pub struct DotNetAnalyzer {
    script_parser: ScriptParser,
}

impl DotNetAnalyzer {
    /// Create a new .NET analyzer
    pub fn new() -> Self {
        Self {
            script_parser: ScriptParser::new(),
        }
    }
}

impl Default for DotNetAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageAnalyzer for DotNetAnalyzer {
    fn detect(&self, root: &Path) -> bool {
        // Check for .sln files
        if has_files_with_extension(root, "sln") {
            return true;
        }
        // Check for .csproj files
        if has_files_with_extension(root, "csproj") {
            return true;
        }
        // Check for .fsproj files (F#)
        if has_files_with_extension(root, "fsproj") {
            return true;
        }
        // Check for global.json (SDK version pinning)
        if root.join("global.json").exists() {
            return true;
        }
        // Check for Directory.Build.props (MSBuild central config)
        if root.join("Directory.Build.props").exists() {
            return true;
        }
        // Check 2-3 levels deep for .csproj, .fsproj, .sln files
        // Handles common .NET layouts where project files are nested
        if has_dotnet_files_recursive(root, 3) {
            return true;
        }
        false
    }

    fn name(&self) -> &'static str {
        ".NET/C#"
    }

    async fn analyze_dependencies(&self, root: &Path) -> AnalyzerResult<Vec<Dependency>> {
        let mut deps = Vec::new();

        // Find all .csproj/.fsproj files (limited depth)
        let project_files: Vec<_> = WalkDir::new(root)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy();
                name.ends_with(".csproj") || name.ends_with(".fsproj")
            })
            .filter(|e| {
                // Skip build output and common non-source directories
                let path = e.path();
                !path.components().any(|c| {
                    matches!(
                        c.as_os_str().to_str(),
                        Some("bin" | "obj" | ".git" | "node_modules" | "packages")
                    )
                })
            })
            .collect();

        debug!("Found {} .csproj/.fsproj files", project_files.len());

        for entry in project_files {
            let project_path = entry.path();
            debug!("Analyzing {}", project_path.display());
            if let Ok(content) = tokio::fs::read_to_string(project_path).await {
                if let Ok(file_deps) = parse_csproj_dependencies(&content, project_path) {
                    deps.extend(file_deps);
                }
            }
        }

        // Check for central package management (Directory.Packages.props)
        let packages_props = root.join("Directory.Packages.props");
        if packages_props.exists() {
            debug!("Analyzing Directory.Packages.props");
            if let Ok(content) = tokio::fs::read_to_string(&packages_props).await {
                if let Ok(central_deps) = parse_directory_packages_props(&content, &packages_props)
                {
                    deps.extend(central_deps);
                }
            }
        }

        // Check if packages are restored (obj/ directories exist)
        let has_obj = root.join("obj").exists()
            || WalkDir::new(root)
                .max_depth(3)
                .into_iter()
                .filter_map(|e| e.ok())
                .any(|e| e.file_name() == "obj" && e.file_type().is_dir());

        if has_obj {
            for dep in &mut deps {
                dep.is_installed = true;
            }
        }

        // Deduplicate by name
        deps.sort_by(|a, b| a.name.cmp(&b.name));
        deps.dedup_by(|a, b| a.name == b.name);

        Ok(deps)
    }

    async fn analyze_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>> {
        // 1. Generate scripts based on .NET project file presence
        let explicit_scripts = generate_dotnet_scripts(root, &self.script_parser);

        // 2. Apply detection rules (for global.json/Directory.Build.props triggers)
        let detected_scripts = apply_rules(root, DOTNET_RULES, &self.script_parser);

        // 3. Merge: explicit scripts take priority
        Ok(merge_scripts(explicit_scripts, detected_scripts))
    }

    fn required_tools(&self, _deps: &[Dependency], _scripts: &[Script]) -> Vec<RequiredTool> {
        // Always need dotnet SDK for .NET projects
        vec![RequiredTool::new(
            "dotnet",
            Ecosystem::DotNet,
            ".NET SDK",
            InstallMethod::vx("dotnet"),
        )]
    }

    fn install_command(&self, dep: &Dependency) -> Option<String> {
        if let Some(ref version) = dep.version {
            Some(format!(
                "dotnet add package {} --version {}",
                dep.name, version
            ))
        } else {
            Some(format!("dotnet add package {}", dep.name))
        }
    }
}

/// Generate scripts based on .NET project file detection
fn generate_dotnet_scripts(root: &Path, parser: &ScriptParser) -> Vec<Script> {
    let has_sln = has_files_with_extension(root, "sln");
    let has_csproj = has_files_with_extension(root, "csproj");
    let has_fsproj = has_files_with_extension(root, "fsproj");

    if !has_sln && !has_csproj && !has_fsproj {
        return Vec::new();
    }

    let reason = if has_sln {
        ".sln file detected"
    } else if has_csproj {
        ".csproj file detected"
    } else {
        ".fsproj file detected"
    };

    let source = ScriptSource::Detected {
        reason: reason.to_string(),
    };

    let mut scripts = Vec::new();

    // Build
    let mut build = Script::new("build", "dotnet build", source.clone());
    build.tools = parser.parse("dotnet build");
    build.description = Some("Build the .NET project".to_string());
    scripts.push(build);

    // Test
    let mut test = Script::new("test", "dotnet test", source.clone());
    test.tools = parser.parse("dotnet test");
    test.description = Some("Run .NET tests".to_string());
    scripts.push(test);

    // Run (only for non-solution single projects)
    if has_csproj && !has_sln {
        let mut run = Script::new("run", "dotnet run", source.clone());
        run.tools = parser.parse("dotnet run");
        run.description = Some("Run the .NET application".to_string());
        scripts.push(run);
    }

    // Restore
    let mut restore = Script::new("restore", "dotnet restore", source.clone());
    restore.tools = parser.parse("dotnet restore");
    restore.description = Some("Restore NuGet packages".to_string());
    scripts.push(restore);

    // Clean
    let mut clean = Script::new("clean", "dotnet clean", source.clone());
    clean.tools = parser.parse("dotnet clean");
    clean.description = Some("Clean build output".to_string());
    scripts.push(clean);

    // Format
    let mut format = Script::new("format", "dotnet format", source);
    format.tools = parser.parse("dotnet format");
    format.description = Some("Format code".to_string());
    scripts.push(format);

    scripts
}

/// Check if directory has files with the given extension (non-recursive, root level only)
fn has_files_with_extension(root: &Path, ext: &str) -> bool {
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_ext) = path.extension() {
                if file_ext.to_string_lossy().eq_ignore_ascii_case(ext) {
                    return true;
                }
            }
        }
    }
    false
}

/// Check if directory contains .NET project files (.csproj, .fsproj, .sln) up to max_depth levels deep.
///
/// This supports common .NET solution layouts where project files are in subdirectories:
/// ```text
/// MyProject/
///   src/
///     MyApp/
///       MyApp.csproj
/// ```
fn has_dotnet_files_recursive(root: &Path, max_depth: usize) -> bool {
    has_dotnet_files_recursive_inner(root, max_depth, 0)
}

fn has_dotnet_files_recursive_inner(dir: &Path, max_depth: usize, current_depth: usize) -> bool {
    if current_depth > max_depth {
        return false;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext.eq_ignore_ascii_case("csproj")
                        || ext.eq_ignore_ascii_case("fsproj")
                        || ext.eq_ignore_ascii_case("sln")
                    {
                        return true;
                    }
                }
            } else if path.is_dir() && current_depth < max_depth {
                // Skip common non-project directories
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.')
                        || name == "node_modules"
                        || name == "bin"
                        || name == "obj"
                        || name == "target"
                        || name == "dist"
                        || name == "packages"
                    {
                        continue;
                    }
                }
                if has_dotnet_files_recursive_inner(&path, max_depth, current_depth + 1) {
                    return true;
                }
            }
        }
    }
    false
}
