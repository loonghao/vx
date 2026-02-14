// Init command implementation - Smart project initialization
//
// Detects project type and generates appropriate vx configuration

use crate::ui::UI;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use vx_config::config_manager::TomlWriter;
use vx_config::{VxConfig, parse_config};
use vx_paths::project::{CONFIG_FILE_NAME, CONFIG_FILE_NAME_LEGACY};
use vx_project_analyzer::{AnalyzerConfig, ProjectAnalyzer};

/// Project detection result
#[derive(Debug, Clone)]
pub struct ProjectDetection {
    /// Detected project types
    pub project_types: Vec<ProjectType>,
    /// Recommended tools with versions
    pub tools: HashMap<String, String>,
    /// Detected package manager
    pub package_manager: Option<PackageManager>,
    /// Project name (from package.json, pyproject.toml, etc.)
    pub project_name: Option<String>,
    /// Additional configuration hints
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectType {
    NodeJs,
    Python,
    Rust,
    Go,
    DotNet,
    Justfile,
    Mixed,
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::NodeJs => write!(f, "Node.js"),
            ProjectType::Python => write!(f, "Python"),
            ProjectType::Rust => write!(f, "Rust"),
            ProjectType::Go => write!(f, "Go"),
            ProjectType::DotNet => write!(f, ".NET/C#"),
            ProjectType::Justfile => write!(f, "Justfile"),
            ProjectType::Mixed => write!(f, "Mixed"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
    Uv,
    Pip,
    Poetry,
    Cargo,
    GoMod,
    NuGet,
}

impl std::fmt::Display for PackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageManager::Npm => write!(f, "npm"),
            PackageManager::Yarn => write!(f, "yarn"),
            PackageManager::Pnpm => write!(f, "pnpm"),
            PackageManager::Bun => write!(f, "bun"),
            PackageManager::Uv => write!(f, "uv"),
            PackageManager::Pip => write!(f, "pip"),
            PackageManager::Poetry => write!(f, "poetry"),
            PackageManager::Cargo => write!(f, "cargo"),
            PackageManager::GoMod => write!(f, "go"),
            PackageManager::NuGet => write!(f, "nuget"),
        }
    }
}

pub async fn handle(
    interactive: bool,
    template: Option<String>,
    tools: Option<String>,
    force: bool,
    dry_run: bool,
    list_templates: bool,
) -> Result<()> {
    if list_templates {
        return list_available_templates();
    }

    let current_dir = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))?;

    // Check if config already exists (check both vx.toml and vx.toml)
    // Prefer vx.toml but respect existing vx.toml
    let (config_path, existing_config) = find_or_create_config_path(&current_dir);

    if let Some(ref existing) = existing_config {
        if !force {
            UI::warn(&format!(
                "Configuration file {} already exists",
                existing.file_name().unwrap().to_string_lossy()
            ));
            UI::info("Use --force to overwrite or edit the existing file");
            return Ok(());
        }
    }

    // When force is used with existing config, merge with existing configuration
    let existing_vx_config = if force {
        existing_config.as_ref().and_then(|p| parse_config(p).ok())
    } else {
        None
    };

    let config_content = if interactive {
        generate_interactive_config(existing_vx_config.as_ref()).await?
    } else if let Some(template_name) = template {
        generate_template_config(&template_name, existing_vx_config.as_ref())?
    } else if let Some(tools_str) = tools {
        generate_tools_config(&tools_str, existing_vx_config.as_ref())?
    } else {
        generate_auto_detected_config(existing_vx_config.as_ref()).await?
    };

    // Determine which file to write to
    let target_path = existing_config.as_ref().unwrap_or(&config_path);
    let target_filename = target_path.file_name().unwrap().to_string_lossy();

    if dry_run {
        UI::info(&format!("Preview of {} configuration:", target_filename));
        println!();
        println!("{}", config_content);
        return Ok(());
    }

    // Write configuration file
    fs::write(target_path, &config_content)
        .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", target_filename, e))?;

    UI::success(&format!(
        "✅ Created {} configuration file",
        target_filename
    ));

    // Generate vx.lock for reproducible environments
    if !config_content.contains("[tools]") || config_content.contains("# Add your tools here") {
        // No tools defined, skip lock file generation
        debug_lock_skip("No tools defined in configuration");
    } else {
        generate_lock_file();
    }

    // Show next steps
    println!();
    println!("Next steps:");
    println!("  1. Review the configuration: cat {}", target_filename);
    println!("  2. Setup development environment: vx setup");
    println!("  3. Or enter dev shell: vx dev");
    println!();
    println!("Optional:");
    println!(
        "  - Add to version control: git add {} vx.lock",
        target_filename
    );
    println!("  - Customize configuration: vx config edit --local");

    Ok(())
}

/// Find existing config or determine path for new config
fn find_or_create_config_path(dir: &Path) -> (PathBuf, Option<PathBuf>) {
    let vx_toml = dir.join(CONFIG_FILE_NAME);
    let legacy_toml = dir.join(CONFIG_FILE_NAME_LEGACY);

    if vx_toml.exists() {
        (vx_toml.clone(), Some(vx_toml))
    } else if legacy_toml.exists() {
        // Respect existing vx.toml
        (legacy_toml.clone(), Some(legacy_toml))
    } else {
        // New config uses vx.toml
        (vx_toml, None)
    }
}

fn list_available_templates() -> Result<()> {
    UI::info("Available templates:");
    println!();
    println!("  node        - Node.js project with npm");
    println!("  node-pnpm   - Node.js project with pnpm");
    println!("  node-yarn   - Node.js project with yarn");
    println!("  node-bun    - Node.js project with bun");
    println!("  python      - Python project with uv");
    println!("  python-pip  - Python project with pip");
    println!("  rust        - Rust project with cargo");
    println!("  go          - Go project");
    println!("  fullstack   - Full-stack project (Node.js + Python)");
    println!("  minimal     - Minimal configuration");
    println!();
    println!("Usage: vx init --template <template>");
    Ok(())
}

async fn generate_interactive_config(existing: Option<&VxConfig>) -> Result<String> {
    UI::header("🚀 VX Project Initialization");

    // First, show auto-detected configuration
    let current_dir = std::env::current_dir()?;
    let detection = detect_project(&current_dir)?;

    if !detection.project_types.is_empty() {
        println!();
        UI::info(&format!(
            "🔍 Detected project type: {}",
            detection
                .project_types
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join(" + ")
        ));
        if let Some(pm) = &detection.package_manager {
            UI::info(&format!("📦 Package manager: {}", pm));
        }
        if let Some(name) = &detection.project_name {
            UI::info(&format!("📁 Project name: {}", name));
        }
        println!();
    }

    // Get project name
    print!("Project name (optional, press Enter to skip): ");
    io::stdout().flush().unwrap();
    let mut project_name = String::new();
    io::stdin().read_line(&mut project_name).unwrap();
    let project_name = project_name.trim();

    // Get description
    print!("Description (optional): ");
    io::stdout().flush().unwrap();
    let mut description = String::new();
    io::stdin().read_line(&mut description).unwrap();
    let description = description.trim();

    // Use detected tools or ask for selection
    println!();
    println!("Select tools to include (detected tools are pre-selected):");
    let available_tools = vec![
        ("node", "20", "Node.js JavaScript runtime"),
        ("npm", "latest", "Node.js package manager"),
        (
            "pnpm",
            "latest",
            "Fast, disk space efficient package manager",
        ),
        ("yarn", "latest", "Package manager for JavaScript"),
        ("bun", "latest", "Fast JavaScript runtime & bundler"),
        ("python", "3.12", "Python interpreter"),
        ("uv", "latest", "Fast Python package manager"),
        ("go", "latest", "Go programming language"),
        ("rust", "latest", "Rust programming language"),
        ("just", "latest", "Command runner"),
    ];

    let mut selected_tools = HashMap::new();

    // Pre-select detected tools
    for (tool, version) in &detection.tools {
        selected_tools.insert(tool.clone(), version.clone());
    }

    for (tool, default_version, desc) in &available_tools {
        let is_detected = detection.tools.contains_key(*tool);
        let marker = if is_detected { " [detected]" } else { "" };
        let default = if is_detected { "Y" } else { "n" };

        print!(
            "Include {} ({}){}? (y/N, default: {}): ",
            tool, desc, marker, default
        );
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();

        let should_include = if input.is_empty() {
            is_detected
        } else {
            input.starts_with('y')
        };

        if should_include {
            if !selected_tools.contains_key(*tool) {
                selected_tools.insert(tool.to_string(), default_version.to_string());
            }
        } else {
            selected_tools.remove(*tool);
        }
    }

    if selected_tools.is_empty() {
        selected_tools.insert("node".to_string(), "20".to_string());
        UI::info("No tools selected, adding Node.js as default");
    }

    generate_config_content(
        project_name,
        description,
        &selected_tools,
        &HashMap::new(),
        true,
        existing,
    )
}

fn generate_template_config(template_name: &str, existing: Option<&VxConfig>) -> Result<String> {
    let tools = match template_name {
        "node" => {
            let mut tools = HashMap::new();
            tools.insert("node".to_string(), "20".to_string());
            tools.insert("npm".to_string(), "latest".to_string());
            tools
        }
        "node-pnpm" => {
            let mut tools = HashMap::new();
            tools.insert("node".to_string(), "20".to_string());
            tools.insert("pnpm".to_string(), "latest".to_string());
            tools
        }
        "node-yarn" => {
            let mut tools = HashMap::new();
            tools.insert("node".to_string(), "20".to_string());
            tools.insert("yarn".to_string(), "latest".to_string());
            tools
        }
        "node-bun" => {
            let mut tools = HashMap::new();
            tools.insert("bun".to_string(), "latest".to_string());
            tools
        }
        "python" => {
            let mut tools = HashMap::new();
            tools.insert("python".to_string(), "3.12".to_string());
            tools.insert("uv".to_string(), "latest".to_string());
            tools
        }
        "python-pip" => {
            let mut tools = HashMap::new();
            tools.insert("python".to_string(), "3.12".to_string());
            tools
        }
        "rust" => {
            let mut tools = HashMap::new();
            tools.insert("rust".to_string(), "latest".to_string());
            tools
        }
        "go" => {
            let mut tools = HashMap::new();
            tools.insert("go".to_string(), "latest".to_string());
            tools
        }
        "fullstack" => {
            let mut tools = HashMap::new();
            tools.insert("node".to_string(), "20".to_string());
            tools.insert("pnpm".to_string(), "latest".to_string());
            tools.insert("python".to_string(), "3.12".to_string());
            tools.insert("uv".to_string(), "latest".to_string());
            tools
        }
        "minimal" => HashMap::new(),
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown template: {}. Use --list-templates to see available templates.",
                template_name
            ));
        }
    };

    generate_config_content("", "", &tools, &HashMap::new(), false, existing)
}

fn generate_tools_config(tools_str: &str, existing: Option<&VxConfig>) -> Result<String> {
    let mut tools = HashMap::new();

    for tool_spec in tools_str.split(',') {
        let tool_spec = tool_spec.trim();
        if tool_spec.contains('@') {
            let parts: Vec<&str> = tool_spec.split('@').collect();
            if parts.len() == 2 {
                tools.insert(parts[0].to_string(), parts[1].to_string());
            }
        } else {
            tools.insert(tool_spec.to_string(), "latest".to_string());
        }
    }

    generate_config_content("", "", &tools, &HashMap::new(), false, existing)
}

/// Detect project type and recommended tools from the current directory
pub fn detect_project(dir: &Path) -> Result<ProjectDetection> {
    let mut detection = ProjectDetection {
        project_types: Vec::new(),
        tools: HashMap::new(),
        package_manager: None,
        project_name: None,
        hints: Vec::new(),
    };

    // Check for Node.js project
    if let Some(node_info) = detect_nodejs_project(dir)? {
        detection.project_types.push(ProjectType::NodeJs);
        detection.tools.extend(node_info.tools);
        if detection.package_manager.is_none() {
            detection.package_manager = node_info.package_manager;
        }
        if detection.project_name.is_none() {
            detection.project_name = node_info.project_name;
        }
        detection.hints.extend(node_info.hints);
    }

    // Check for Python project
    if let Some(python_info) = detect_python_project(dir)? {
        detection.project_types.push(ProjectType::Python);
        detection.tools.extend(python_info.tools);
        if detection.package_manager.is_none() {
            detection.package_manager = python_info.package_manager;
        }
        if detection.project_name.is_none() {
            detection.project_name = python_info.project_name;
        }
        detection.hints.extend(python_info.hints);
    }

    // Check for Go project
    if dir.join("go.mod").exists() {
        detection.project_types.push(ProjectType::Go);
        detection
            .tools
            .insert("go".to_string(), "latest".to_string());
        if detection.package_manager.is_none() {
            detection.package_manager = Some(PackageManager::GoMod);
        }

        // Try to get module name
        if let Ok(content) = fs::read_to_string(dir.join("go.mod")) {
            if let Some(line) = content.lines().next() {
                if let Some(module_name) = line.strip_prefix("module ") {
                    if detection.project_name.is_none() {
                        detection.project_name = Some(module_name.trim().to_string());
                    }
                }
            }
        }
    }

    // Check for Rust project
    if dir.join("Cargo.toml").exists() {
        detection.project_types.push(ProjectType::Rust);
        if detection.package_manager.is_none() {
            detection.package_manager = Some(PackageManager::Cargo);
        }

        // Try to get rust version from rust-toolchain.toml first, then Cargo.toml
        let mut rust_version = None;

        // Check rust-toolchain.toml
        let toolchain_path = dir.join("rust-toolchain.toml");
        if toolchain_path.exists() {
            if let Ok(content) = fs::read_to_string(&toolchain_path) {
                rust_version = extract_rust_toolchain_version(&content);
            }
        }

        // Check rust-toolchain (legacy format)
        if rust_version.is_none() {
            let toolchain_legacy_path = dir.join("rust-toolchain");
            if toolchain_legacy_path.exists() {
                if let Ok(content) = fs::read_to_string(&toolchain_legacy_path) {
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        rust_version = Some(trimmed.to_string());
                    }
                }
            }
        }

        // Check Cargo.toml for rust-version or package.rust-version
        if let Ok(content) = fs::read_to_string(dir.join("Cargo.toml")) {
            // Get project name
            for line in content.lines() {
                if let Some(name) = line.strip_prefix("name = ") {
                    let name = name.trim().trim_matches('"');
                    if detection.project_name.is_none() {
                        detection.project_name = Some(name.to_string());
                    }
                    break;
                }
            }

            // Get rust-version if not already found
            if rust_version.is_none() {
                rust_version = extract_cargo_rust_version(&content);
            }
        }

        // Use detected version or default to "stable"
        let version = rust_version.unwrap_or_else(|| "stable".to_string());
        detection.tools.insert("rust".to_string(), version);
    }

    // Check for .NET/C# project
    if let Some(dotnet_info) = detect_dotnet_project(dir)? {
        detection.project_types.push(ProjectType::DotNet);
        detection.tools.extend(dotnet_info.tools);
        if detection.package_manager.is_none() {
            detection.package_manager = Some(PackageManager::NuGet);
        }
        if detection.project_name.is_none() {
            detection.project_name = dotnet_info.project_name;
        }
        detection.hints.extend(dotnet_info.hints);
    }

    // Check for Justfile
    if dir.join("justfile").exists() || dir.join("Justfile").exists() {
        detection.project_types.push(ProjectType::Justfile);
        detection
            .tools
            .insert("just".to_string(), "latest".to_string());
        detection
            .hints
            .push("Justfile detected - 'just' command runner will be available".to_string());
    }

    // Mark as mixed if multiple project types
    if detection.project_types.len() > 1 {
        detection.project_types.insert(0, ProjectType::Mixed);
    }

    Ok(detection)
}

#[derive(Debug)]
struct NodeJsDetection {
    tools: HashMap<String, String>,
    package_manager: Option<PackageManager>,
    project_name: Option<String>,
    hints: Vec<String>,
}

fn detect_nodejs_project(dir: &Path) -> Result<Option<NodeJsDetection>> {
    let package_json_path = dir.join("package.json");
    if !package_json_path.exists() {
        return Ok(None);
    }

    let mut detection = NodeJsDetection {
        tools: HashMap::new(),
        package_manager: None,
        project_name: None,
        hints: Vec::new(),
    };

    // Parse package.json
    if let Ok(content) = fs::read_to_string(&package_json_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            // Get project name
            if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
                detection.project_name = Some(name.to_string());
            }

            // Check for packageManager field (corepack)
            if let Some(pm) = json.get("packageManager").and_then(|v| v.as_str()) {
                if pm.starts_with("pnpm") {
                    detection.package_manager = Some(PackageManager::Pnpm);
                    detection
                        .tools
                        .insert("pnpm".to_string(), "latest".to_string());
                } else if pm.starts_with("yarn") {
                    detection.package_manager = Some(PackageManager::Yarn);
                    detection
                        .tools
                        .insert("yarn".to_string(), "latest".to_string());
                } else if pm.starts_with("npm") {
                    detection.package_manager = Some(PackageManager::Npm);
                    detection
                        .tools
                        .insert("npm".to_string(), "latest".to_string());
                } else if pm.starts_with("bun") {
                    detection.package_manager = Some(PackageManager::Bun);
                    detection
                        .tools
                        .insert("bun".to_string(), "latest".to_string());
                }
            }

            // Check engines field for Node.js version
            if let Some(engines) = json.get("engines") {
                if let Some(node_version) = engines.get("node").and_then(|v| v.as_str()) {
                    // Parse version constraint (e.g., ">=18.0.0", "^20.0.0", "20.x")
                    let version = parse_node_version_constraint(node_version);
                    detection.tools.insert("node".to_string(), version);
                }
            }
        }
    }

    // Detect package manager from lock files
    if detection.package_manager.is_none() {
        if dir.join("pnpm-lock.yaml").exists() {
            detection.package_manager = Some(PackageManager::Pnpm);
            detection
                .tools
                .insert("pnpm".to_string(), "latest".to_string());
            detection
                .hints
                .push("Detected pnpm from pnpm-lock.yaml".to_string());
        } else if dir.join("yarn.lock").exists() {
            detection.package_manager = Some(PackageManager::Yarn);
            detection
                .tools
                .insert("yarn".to_string(), "latest".to_string());
            detection
                .hints
                .push("Detected yarn from yarn.lock".to_string());
        } else if dir.join("bun.lockb").exists() || dir.join("bun.lock").exists() {
            detection.package_manager = Some(PackageManager::Bun);
            detection
                .tools
                .insert("bun".to_string(), "latest".to_string());
            detection
                .hints
                .push("Detected bun from bun.lockb".to_string());
        } else if dir.join("package-lock.json").exists() {
            detection.package_manager = Some(PackageManager::Npm);
            detection
                .tools
                .insert("npm".to_string(), "latest".to_string());
            detection
                .hints
                .push("Detected npm from package-lock.json".to_string());
        } else {
            // Default to npm
            detection.package_manager = Some(PackageManager::Npm);
            detection
                .tools
                .insert("npm".to_string(), "latest".to_string());
        }
    }

    // Ensure node is added if not specified
    if !detection.tools.contains_key("node") && !detection.tools.contains_key("bun") {
        detection.tools.insert("node".to_string(), "20".to_string());
    }

    Ok(Some(detection))
}

fn parse_node_version_constraint(constraint: &str) -> String {
    // Handle common version constraints
    let constraint = constraint.trim();

    // Remove operators
    let version = constraint
        .trim_start_matches(">=")
        .trim_start_matches("<=")
        .trim_start_matches('>')
        .trim_start_matches('<')
        .trim_start_matches('^')
        .trim_start_matches('~')
        .trim_start_matches('=')
        .trim();

    // Handle .x notation (e.g., "20.x" -> "20")
    if let Some(base) = version.split('.').next() {
        if let Ok(major) = base.parse::<u32>() {
            return major.to_string();
        }
    }

    // Return as-is if we can't parse
    version.to_string()
}

#[derive(Debug)]
struct PythonDetection {
    tools: HashMap<String, String>,
    package_manager: Option<PackageManager>,
    project_name: Option<String>,
    hints: Vec<String>,
}

fn detect_python_project(dir: &Path) -> Result<Option<PythonDetection>> {
    let pyproject_path = dir.join("pyproject.toml");
    let requirements_path = dir.join("requirements.txt");
    let setup_py_path = dir.join("setup.py");

    if !pyproject_path.exists() && !requirements_path.exists() && !setup_py_path.exists() {
        return Ok(None);
    }

    let mut detection = PythonDetection {
        tools: HashMap::new(),
        package_manager: None,
        project_name: None,
        hints: Vec::new(),
    };

    // Default Python version
    detection
        .tools
        .insert("python".to_string(), "3.12".to_string());

    // Parse pyproject.toml
    if pyproject_path.exists() {
        if let Ok(content) = fs::read_to_string(&pyproject_path) {
            // Check for uv
            if content.contains("[tool.uv]") || dir.join("uv.lock").exists() {
                detection.package_manager = Some(PackageManager::Uv);
                detection
                    .tools
                    .insert("uv".to_string(), "latest".to_string());
                detection
                    .hints
                    .push("Detected uv from pyproject.toml or uv.lock".to_string());
            }
            // Check for poetry
            else if content.contains("[tool.poetry]") || dir.join("poetry.lock").exists() {
                detection.package_manager = Some(PackageManager::Poetry);
                detection
                    .hints
                    .push("Detected poetry from pyproject.toml".to_string());
            }

            // Try to get project name
            for line in content.lines() {
                if let Some(name) = line.strip_prefix("name = ") {
                    let name = name.trim().trim_matches('"');
                    detection.project_name = Some(name.to_string());
                    break;
                }
            }

            // Try to get Python version requirement
            for line in content.lines() {
                if line.contains("requires-python") || line.contains("python_requires") {
                    if let Some(version) = extract_python_version(line) {
                        detection.tools.insert("python".to_string(), version);
                        break;
                    }
                }
            }
        }
    }

    // Check for uv.lock
    if detection.package_manager.is_none() && dir.join("uv.lock").exists() {
        detection.package_manager = Some(PackageManager::Uv);
        detection
            .tools
            .insert("uv".to_string(), "latest".to_string());
        detection.hints.push("Detected uv from uv.lock".to_string());
    }

    // Default to uv if no package manager detected (it's the fastest)
    if detection.package_manager.is_none() {
        detection.package_manager = Some(PackageManager::Uv);
        detection
            .tools
            .insert("uv".to_string(), "latest".to_string());
        detection
            .hints
            .push("Recommending uv as default Python package manager".to_string());
    }

    Ok(Some(detection))
}

#[derive(Debug)]
struct DotNetDetection {
    tools: HashMap<String, String>,
    project_name: Option<String>,
    hints: Vec<String>,
}

fn detect_dotnet_project(dir: &Path) -> Result<Option<DotNetDetection>> {
    // Check for .sln, .csproj, .fsproj, or global.json at root level
    let has_sln = has_files_with_extension(dir, "sln");
    let has_csproj = has_files_with_extension(dir, "csproj");
    let has_fsproj = has_files_with_extension(dir, "fsproj");
    let has_global_json = dir.join("global.json").exists();
    let has_dir_build_props = dir.join("Directory.Build.props").exists();

    // Also check 2-3 levels deep for .csproj, .fsproj, .sln files
    // This handles common .NET solution layouts like:
    //   MyProject/
    //     src/
    //       MyApp/
    //         MyApp.csproj
    //       MyLib/
    //         MyLib.csproj
    let has_deep_dotnet = if !has_sln && !has_csproj && !has_fsproj {
        has_dotnet_files_recursive(dir, 3)
    } else {
        false
    };

    if !has_sln
        && !has_csproj
        && !has_fsproj
        && !has_global_json
        && !has_dir_build_props
        && !has_deep_dotnet
    {
        return Ok(None);
    }

    let mut detection = DotNetDetection {
        tools: HashMap::new(),
        project_name: None,
        hints: Vec::new(),
    };

    // Default dotnet SDK version
    detection
        .tools
        .insert("dotnet".to_string(), "latest".to_string());

    // Try to get SDK version from global.json
    if has_global_json {
        if let Ok(content) = fs::read_to_string(dir.join("global.json")) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(version) = json
                    .get("sdk")
                    .and_then(|sdk| sdk.get("version"))
                    .and_then(|v| v.as_str())
                {
                    detection
                        .tools
                        .insert("dotnet".to_string(), version.to_string());
                    detection.hints.push(format!(
                        ".NET SDK version {} pinned in global.json",
                        version
                    ));
                }
            }
        }
    }

    // Try to get project name from .sln or .csproj
    if has_sln {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("sln"))
                {
                    if let Some(stem) = path.file_stem() {
                        detection.project_name = Some(stem.to_string_lossy().to_string());
                    }
                    break;
                }
            }
        }
        detection
            .hints
            .push("Solution file detected - multi-project .NET solution".to_string());
    } else if has_csproj {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("csproj"))
                {
                    if let Some(stem) = path.file_stem() {
                        detection.project_name = Some(stem.to_string_lossy().to_string());
                    }
                    break;
                }
            }
        }
        detection.hints.push("C# project detected".to_string());
    } else if has_fsproj {
        detection.hints.push("F# project detected".to_string());
    }

    Ok(Some(detection))
}

/// Check if directory has files with the given extension (non-recursive, root level only)
fn has_files_with_extension(dir: &Path, ext: &str) -> bool {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case(ext))
            {
                return true;
            }
        }
    }
    false
}

/// Check if directory contains .NET project files (.csproj, .fsproj, .sln) up to max_depth levels deep.
///
/// This handles common .NET solution layouts where project files are nested in subdirectories:
/// ```text
/// MyProject/
///   src/
///     MyApp/
///       MyApp.csproj
///     MyLib/
///       MyLib.csproj
/// ```
fn has_dotnet_files_recursive(dir: &Path, max_depth: usize) -> bool {
    has_dotnet_files_recursive_inner(dir, max_depth, 0)
}

fn has_dotnet_files_recursive_inner(dir: &Path, max_depth: usize, current_depth: usize) -> bool {
    if current_depth > max_depth {
        return false;
    }
    if let Ok(entries) = fs::read_dir(dir) {
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

fn extract_python_version(line: &str) -> Option<String> {
    // Handle formats like: requires-python = ">=3.10"
    if let Some(start) = line.find('"') {
        if let Some(end) = line.rfind('"') {
            if start < end {
                let version_str = &line[start + 1..end];
                // Parse version constraint
                let version = version_str
                    .trim_start_matches(">=")
                    .trim_start_matches("<=")
                    .trim_start_matches('>')
                    .trim_start_matches('<')
                    .trim_start_matches('^')
                    .trim_start_matches('~')
                    .trim_start_matches('=')
                    .split(',')
                    .next()
                    .unwrap_or("3.12")
                    .trim();
                return Some(version.to_string());
            }
        }
    }
    None
}

async fn generate_auto_detected_config(existing: Option<&VxConfig>) -> Result<String> {
    let current_dir = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))?;

    let detection = detect_project(&current_dir)?;

    if detection.project_types.is_empty() {
        UI::info("No project type detected, creating minimal configuration");
        let mut tools = HashMap::new();
        tools.insert("node".to_string(), "20".to_string());
        return generate_config_content("", "", &tools, &HashMap::new(), false, existing);
    }

    // Show detection results
    let project_types_str = detection
        .project_types
        .iter()
        .filter(|t| **t != ProjectType::Mixed)
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join(" + ");

    UI::info(&format!("🔍 Detected project type: {}", project_types_str));

    if let Some(pm) = &detection.package_manager {
        UI::info(&format!("📦 Package manager: {}", pm));
    }

    if let Some(name) = &detection.project_name {
        UI::info(&format!("📁 Project: {}", name));
    }

    for hint in &detection.hints {
        UI::hint(hint);
    }

    // Use ProjectAnalyzer to detect scripts
    let analyzer_config = AnalyzerConfig {
        check_installed: false,
        check_tools: false,
        generate_sync_actions: false,
        max_depth: 1,
    };
    let analyzer = ProjectAnalyzer::new(analyzer_config);
    let analysis_result = analyzer.analyze(&current_dir).await;

    // Extract scripts from analysis
    let detected_scripts: HashMap<String, String> = match analysis_result {
        Ok(a) => a.scripts.into_iter().map(|s| (s.name, s.command)).collect(),
        Err(e) => {
            // Log error but continue without scripts
            tracing::debug!("Failed to analyze project for scripts: {}", e);
            HashMap::new()
        }
    };

    if !detected_scripts.is_empty() {
        UI::info(&format!("📜 Detected {} script(s)", detected_scripts.len()));
    }

    generate_config_content(
        detection.project_name.as_deref().unwrap_or(""),
        "",
        &detection.tools,
        &detected_scripts,
        false,
        existing,
    )
}

fn generate_config_content(
    project_name: &str,
    description: &str,
    detected_tools: &HashMap<String, String>,
    detected_scripts: &HashMap<String, String>,
    include_extras: bool,
    existing: Option<&VxConfig>,
) -> Result<String> {
    let mut writer = TomlWriter::new();

    // Header comments
    writer = writer
        .comment("VX Project Configuration")
        .comment("This file defines the tools and versions required for this project.")
        .comment("Run 'vx setup' to install all required tools.")
        .comment("Run 'vx dev' to enter the development environment.");

    if !project_name.is_empty() {
        writer = writer.comment(&format!("Project: {}", project_name));
    }
    if !description.is_empty() {
        writer = writer.comment(&format!("Description: {}", description));
    }

    // Merge tools: existing config takes priority for version numbers
    let mut tools = detected_tools.clone();
    if let Some(existing_config) = existing {
        for (name, version) in existing_config.tools_as_hashmap() {
            tools.insert(name, version);
        }
    }

    // Tools section
    writer = writer.section("tools");
    if tools.is_empty() {
        writer = writer
            .comment("Add your tools here, for example:")
            .comment("node = \"20\"")
            .comment("python = \"3.12\"")
            .comment("uv = \"latest\"");
    } else {
        writer = writer.kv_map_sorted(&tools);
    }

    // Settings section
    writer = writer.section("settings");
    if let Some(existing_config) = existing {
        let settings = existing_config.settings_as_hashmap();
        if !settings.is_empty() {
            for (key, value) in settings.iter() {
                writer = writer.kv_raw(key, &format_value(value));
            }
        } else {
            writer = writer
                .comment("Automatically install missing tools when entering dev environment")
                .kv_bool("auto_install", true)
                .comment("Cache duration for version checks")
                .kv("cache_duration", "7d");
        }
    } else {
        writer = writer
            .comment("Automatically install missing tools when entering dev environment")
            .kv_bool("auto_install", true)
            .comment("Cache duration for version checks")
            .kv("cache_duration", "7d");
    }

    if include_extras {
        writer = writer
            .comment("Install tools in parallel")
            .kv_bool("parallel_install", true);
    }

    // Scripts section
    let mut scripts = detected_scripts.clone();
    if let Some(existing_config) = existing {
        for (name, cmd) in existing_config.scripts_as_hashmap() {
            scripts.insert(name, cmd);
        }
    }

    if !scripts.is_empty() {
        writer = writer.section("scripts").kv_map_sorted(&scripts);
    } else if include_extras {
        writer = writer
            .section("scripts")
            .comment("Define custom scripts that can be run with 'vx run <script>'")
            .comment("dev = \"npm run dev\"")
            .comment("test = \"npm test\"")
            .comment("build = \"npm run build\"");
    }

    // Env section
    let env_vars = existing.map(|c| c.env_as_hashmap()).unwrap_or_default();
    if !env_vars.is_empty() {
        writer = writer.section("env").kv_map_sorted(&env_vars);
    } else if include_extras {
        writer = writer
            .section("env")
            .comment("Environment variables to set in the dev environment")
            .comment("NODE_ENV = \"development\"")
            .comment("DEBUG = \"true\"");
    }

    Ok(writer.build())
}

/// Format a value for TOML output (bool/number as raw, string quoted)
fn format_value(value: &str) -> String {
    if value == "true"
        || value == "false"
        || value.parse::<i64>().is_ok()
        || value.parse::<f64>().is_ok()
    {
        value.to_string()
    } else {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

/// Extract Rust version from rust-toolchain.toml content
///
/// Supports formats like:
/// - `channel = "stable"`
/// - `channel = "nightly"`
/// - `channel = "1.83.0"`
/// - `[toolchain]\nchannel = "stable"`
fn extract_rust_toolchain_version(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if let Some(channel) = line.strip_prefix("channel") {
            // Handle: channel = "stable" or channel="stable"
            let value = channel.trim_start_matches([' ', '=']).trim();
            let value = value.trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Extract rust-version from Cargo.toml content
///
/// Supports formats like:
/// - `rust-version = "1.83.0"`
/// - `rust-version = "1.83"`
/// - `[package]\nrust-version = "1.83.0"`
fn extract_cargo_rust_version(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("rust-version") {
            // Handle: rust-version = "1.83.0" or rust-version="1.83.0"
            let value = value.trim_start_matches([' ', '=']).trim();
            let value = value.trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Generate vx.lock after creating vx.toml
fn generate_lock_file() {
    use std::process::Command;

    UI::info("Generating vx.lock for reproducible environments...");

    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(e) => {
            tracing::debug!("Failed to get current exe for lock generation: {}", e);
            UI::hint("Run 'vx lock' manually to generate the lock file");
            return;
        }
    };

    match Command::new(exe)
        .args(["lock"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
    {
        Ok(output) if output.status.success() => {
            UI::success("✅ Generated vx.lock");
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::debug!("vx lock failed: {}", stderr);
            UI::hint("Run 'vx lock' manually to generate the lock file");
        }
        Err(e) => {
            tracing::debug!("Failed to run vx lock: {}", e);
            UI::hint("Run 'vx lock' manually to generate the lock file");
        }
    }
}

fn debug_lock_skip(reason: &str) {
    tracing::debug!("Skipping lock file generation: {}", reason);
}
