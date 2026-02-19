//! AI tools command - manage AI agent skills and integrations
//!
//! This module implements the `vx ai` command for managing AI agent skills.
//!
//! ## Commands
//!
//! - `vx ai setup` - Auto-install vx skills to AI agent configuration directories
//! - `vx ai skills <args...>` - Proxy to Vercel Skills CLI (npx skills)
//! - `vx ai agents` - List supported AI agents and their config paths
//! - `vx ai context` - Generate AI-friendly project context (RFC 0035)
//! - `vx ai session` - Manage AI session state (RFC 0035)

#![allow(clippy::collapsible_if)]

use crate::cli::{OutputFormat, SessionCommand};
use crate::commands::CommandContext;
use crate::output::{
    AiContextOutput, ConstraintInfo, OutputRenderer, ProjectInfo, ScriptInfo, ToolInfo,
};
use crate::ui::UI;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;

/// Supported AI agent targets: (name, project_skills_dir, global_skills_dir)
const SUPPORTED_AGENTS: &[(&str, &str, &str)] = &[
    ("codebuddy", ".codebuddy/skills", ".codebuddy/skills"),
    ("claude-code", ".claude/skills", ".claude/skills"),
    ("cursor", ".cursor/skills", ".cursor/skills"),
    ("codex", ".agents/skills", ".codex/skills"),
    ("windsurf", ".windsurf/skills", ".codeium/windsurf/skills"),
    ("copilot", ".agents/skills", ".copilot/skills"),
    ("opencode", ".opencode/skills", ".config/opencode/skills"),
    ("trae", ".trae/skills", ".trae/skills"),
    ("gemini-cli", ".agents/skills", ".gemini/skills"),
    ("amp", ".agents/skills", ".config/agents/skills"),
    ("roo", ".roo/skills", ".roo/skills"),
    ("cline", ".cline/skills", ".cline/skills"),
    ("kiro-cli", ".kiro/skills", ".kiro/skills"),
];

/// Embedded vx-usage SKILL.md content for AI agents
const VX_USAGE_SKILL: &str = include_str!("../skills/vx-usage/SKILL.md");

/// Handle `vx ai setup` command
///
/// Installs vx's built-in skills to the target AI agent configuration directories,
/// so that AI coding agents can better understand and use vx.
pub async fn handle_setup(agents: &[String], global: bool, force: bool) -> Result<()> {
    let target_agents = resolve_agents(agents)?;

    UI::header("Setting up vx skills for AI agents");
    println!();

    let home_dir = dirs::home_dir().context("Could not determine home directory")?;
    let cwd = std::env::current_dir().context("Could not determine current directory")?;

    let mut installed_count = 0;

    for (agent_name, project_dir, global_dir) in &target_agents {
        let dirs_to_install: Vec<PathBuf> = if global {
            vec![home_dir.join(global_dir)]
        } else {
            // Install to project-level directory by default
            vec![cwd.join(project_dir)]
        };

        for target_dir in dirs_to_install {
            let skill_dir = target_dir.join("vx-usage");
            let skill_file = skill_dir.join("SKILL.md");

            if skill_file.exists() && !force {
                UI::info(&format!(
                    "  {} - vx-usage already installed at {}",
                    agent_name,
                    skill_dir.display()
                ));
                continue;
            }

            // Create directory and write SKILL.md
            std::fs::create_dir_all(&skill_dir)
                .with_context(|| format!("Failed to create directory: {}", skill_dir.display()))?;

            std::fs::write(&skill_file, VX_USAGE_SKILL)
                .with_context(|| format!("Failed to write SKILL.md to {}", skill_file.display()))?;

            UI::success(&format!(
                "  {} - installed vx-usage to {}",
                agent_name,
                skill_dir.display()
            ));
            installed_count += 1;
        }
    }

    println!();
    if installed_count > 0 {
        UI::success(&format!(
            "Installed vx-usage skill to {} location(s)",
            installed_count
        ));
    } else {
        UI::info("All skills already up to date. Use --force to reinstall.");
    }

    UI::hint("AI agents will now have access to vx usage instructions.");

    Ok(())
}

/// Handle `vx ai agents` command - list supported AI agents
pub async fn handle_agents() -> Result<()> {
    UI::header("Supported AI Agents");
    println!();
    let header = format!("  {:<16} {:<24} GLOBAL DIR", "AGENT", "PROJECT DIR");
    println!("{header}");
    println!("  {}", "-".repeat(68));

    for (name, project_path, global_path) in SUPPORTED_AGENTS {
        println!("  {:<16} {:<24} ~/{}", name, project_path, global_path);
    }

    println!();
    UI::info(&format!(
        "Total: {} supported agents",
        SUPPORTED_AGENTS.len()
    ));
    UI::hint("Use `vx ai setup -a <agent>` to install vx skills for a specific agent");

    Ok(())
}

/// Handle `vx ai skills <args...>` - proxy to Vercel Skills CLI via npx
///
/// Uses `vx npx skills` to run the Vercel Skills CLI. This leverages vx's
/// existing runtime dependency resolution — node will be auto-installed if needed.
pub async fn handle_skills(_ctx: &CommandContext, args: &[String]) -> Result<()> {
    // Find the vx binary itself to use `vx npx`
    let vx_exe = std::env::current_exe().context("Could not determine vx executable path")?;

    let mut cmd_args = vec!["npx".to_string(), "skills".to_string()];
    cmd_args.extend_from_slice(args);

    let status = std::process::Command::new(&vx_exe)
        .args(&cmd_args)
        .status()
        .context("Failed to execute 'vx npx skills'. Is node available?")?;

    if !status.success() {
        let code = status.code().unwrap_or(1);
        std::process::exit(code);
    }

    Ok(())
}

/// Resolve agent names: if empty, use all agents; otherwise validate names
fn resolve_agents(agents: &[String]) -> Result<Vec<(&'static str, &'static str, &'static str)>> {
    if agents.is_empty() {
        return Ok(SUPPORTED_AGENTS.to_vec());
    }

    let mut result = Vec::new();
    for name in agents {
        let found = SUPPORTED_AGENTS
            .iter()
            .find(|(n, _, _)| *n == name.as_str());
        match found {
            Some(agent) => result.push(*agent),
            None => {
                let available: Vec<&str> = SUPPORTED_AGENTS.iter().map(|(n, _, _)| *n).collect();
                anyhow::bail!(
                    "Unknown agent '{}'. Available agents: {}",
                    name,
                    available.join(", ")
                );
            }
        }
    }
    Ok(result)
}

// ============================================================================
// vx ai context (RFC 0035)
// ============================================================================

/// Handle `vx ai context` command
///
/// Generates an AI-friendly project context including:
/// - Project information (languages, frameworks, package managers)
/// - Installed tools and versions
/// - Available scripts
/// - Tool constraints
/// - Recommended commands
pub async fn handle_context(
    ctx: &CommandContext,
    minimal: bool,
    format: OutputFormat,
) -> Result<()> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    // Get project analysis
    let project_name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let project_root = current_dir.display().to_string();

    // Collect installed tools
    let tools = collect_installed_tools(ctx).await?;

    // Collect scripts from vx.toml or package.json
    let scripts = collect_project_scripts(&current_dir).await?;

    // Collect tool constraints
    let constraints = collect_tool_constraints(&current_dir)?;

    // Collect important files
    let important_files = collect_important_files(&current_dir);

    // Generate recommended commands
    let recommended_commands = generate_recommended_commands(&tools, &scripts);

    // Environment variables
    let env_vars = if minimal {
        HashMap::new()
    } else {
        collect_relevant_env_vars()
    };

    // Determine languages and frameworks
    let (languages, frameworks, package_managers) =
        detect_languages_and_frameworks(&current_dir, &tools);

    let output = AiContextOutput {
        project: ProjectInfo {
            name: project_name,
            root: project_root,
            languages,
            frameworks,
            package_managers,
        },
        tools,
        scripts,
        env_vars,
        constraints,
        important_files,
        recommended_commands,
    };

    let renderer = OutputRenderer::new(format);
    renderer.render(&output)?;

    Ok(())
}

/// Collect installed tools information
async fn collect_installed_tools(ctx: &CommandContext) -> Result<Vec<ToolInfo>> {
    let mut tools = Vec::new();

    for name in ctx.registry().runtime_names() {
        if let Some(runtime) = ctx.registry().get_runtime(&name) {
            // Get installed versions
            if let Ok(versions) = runtime.installed_versions(ctx.runtime_context()).await
                && let Some(version) = versions.into_iter().max()
            {
                let path = runtime
                    .get_executable_path_for_version(&version, ctx.runtime_context())
                    .await
                    .ok()
                    .flatten()
                    .map(|p| p.display().to_string());

                let ecosystem = runtime.ecosystem().to_string();
                let ecosystem_str = if ecosystem == "Unknown" {
                    None
                } else {
                    Some(ecosystem)
                };

                tools.push(ToolInfo {
                    name: runtime.name().to_string(),
                    version,
                    source: "vx".to_string(),
                    ecosystem: ecosystem_str,
                    path,
                });
            }
        }
    }

    // Sort by name
    tools.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(tools)
}

/// Collect project scripts
async fn collect_project_scripts(project_root: &std::path::Path) -> Result<Vec<ScriptInfo>> {
    let mut scripts = Vec::new();

    // Check for vx.toml scripts
    let vx_toml = project_root.join("vx.toml");
    if vx_toml.exists() {
        if let Ok(content) = std::fs::read_to_string(&vx_toml) {
            // Simple parsing - look for [scripts] section
            let mut in_scripts = false;
            for line in content.lines() {
                let line = line.trim();
                if line == "[scripts]" {
                    in_scripts = true;
                    continue;
                }
                if line.starts_with('[') {
                    in_scripts = false;
                }
                if in_scripts && line.contains('=') {
                    if let Some((name, command)) = line.split_once('=') {
                        let name = name.trim().to_string();
                        let command = command.trim().trim_matches('"').to_string();
                        scripts.push(ScriptInfo {
                            name,
                            command,
                            description: None,
                            tools: vec![],
                        });
                    }
                }
            }
        }
    }

    // Check for package.json scripts
    let package_json = project_root.join("package.json");
    if package_json.exists() {
        if let Ok(content) = std::fs::read_to_string(&package_json) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(obj) = json.as_object() {
                    if let Some(scripts_obj) = obj.get("scripts").and_then(|s| s.as_object()) {
                        for (name, cmd_value) in scripts_obj {
                            if let Some(command) = cmd_value.as_str() {
                                scripts.push(ScriptInfo {
                                    name: name.clone(),
                                    command: command.to_string(),
                                    description: None,
                                    tools: vec![],
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(scripts)
}

/// Collect tool constraints from vx.toml
fn collect_tool_constraints(project_root: &std::path::Path) -> Result<Vec<ConstraintInfo>> {
    let mut constraints = Vec::new();

    let vx_toml = project_root.join("vx.toml");
    if !vx_toml.exists() {
        return Ok(constraints);
    }

    let content = std::fs::read_to_string(&vx_toml).context("Failed to read vx.toml")?;

    // Simple parsing - look for [tools] section
    let mut in_tools = false;
    for line in content.lines() {
        let line = line.trim();
        if line == "[tools]" {
            in_tools = true;
            continue;
        }
        if line.starts_with('[') {
            in_tools = false;
        }
        if in_tools && line.contains('=') {
            if let Some((tool, version)) = line.split_once('=') {
                let tool = tool.trim().to_string();
                let version = version.trim().trim_matches('"').to_string();
                constraints.push(ConstraintInfo {
                    tool,
                    constraint: version,
                    reason: None,
                    satisfied: true, // TODO: Check if actually satisfied
                });
            }
        }
    }

    Ok(constraints)
}

/// Collect important files in the project
fn collect_important_files(project_root: &std::path::Path) -> Vec<String> {
    let mut files = Vec::new();

    let important_files = [
        "vx.toml",
        "vx.lock",
        "package.json",
        "package-lock.json",
        "yarn.lock",
        "pnpm-lock.yaml",
        "Cargo.toml",
        "Cargo.lock",
        "go.mod",
        "go.sum",
        "pyproject.toml",
        "requirements.txt",
        "justfile",
        "Makefile",
        "README.md",
        ".env.example",
    ];

    for file_name in important_files {
        let path = project_root.join(file_name);
        if path.exists() {
            files.push(file_name.to_string());
        }
    }

    files
}

/// Generate recommended commands based on tools and scripts
fn generate_recommended_commands(tools: &[ToolInfo], scripts: &[ScriptInfo]) -> Vec<String> {
    let mut commands = Vec::new();

    // Common development commands
    let has_node = tools.iter().any(|t| t.name == "node");
    let has_python = tools.iter().any(|t| t.name == "python" || t.name == "uv");
    let has_go = tools.iter().any(|t| t.name == "go");
    let has_rust = tools.iter().any(|t| t.name == "rust" || t.name == "cargo");

    if has_node {
        commands.push("vx npm install".to_string());
        commands.push("vx npm test".to_string());
    }

    if has_python {
        commands.push("vx uv sync".to_string());
        commands.push("vx uv run pytest".to_string());
    }

    if has_go {
        commands.push("vx go mod download".to_string());
        commands.push("vx go test ./...".to_string());
    }

    if has_rust {
        commands.push("vx cargo build".to_string());
        commands.push("vx cargo test".to_string());
    }

    // Add script commands
    for script in scripts.iter().take(5) {
        commands.push(format!("vx run {}", script.name));
    }

    commands
}

/// Collect relevant environment variables
fn collect_relevant_env_vars() -> HashMap<String, String> {
    let mut vars = HashMap::new();
    let relevant_vars = [
        "NODE_ENV",
        "PYTHONPATH",
        "GOPATH",
        "CARGO_HOME",
        "PATH",
        "VX_HOME",
    ];

    for var_name in relevant_vars {
        if let Ok(value) = std::env::var(var_name) {
            vars.insert(var_name.to_string(), value);
        }
    }

    vars
}

/// Detect languages and frameworks from project files
fn detect_languages_and_frameworks(
    project_root: &std::path::Path,
    tools: &[ToolInfo],
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut languages = Vec::new();
    let mut frameworks = Vec::new();
    let mut package_managers = Vec::new();

    // Check for Node.js
    if project_root.join("package.json").exists() {
        languages.push("JavaScript/TypeScript".to_string());

        // Try to detect framework
        if let Ok(content) = std::fs::read_to_string(project_root.join("package.json")) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
                    if deps.contains_key("next") {
                        frameworks.push("Next.js".to_string());
                    }
                    if deps.contains_key("react") {
                        frameworks.push("React".to_string());
                    }
                    if deps.contains_key("vue") {
                        frameworks.push("Vue.js".to_string());
                    }
                    if deps.contains_key("svelte") {
                        frameworks.push("Svelte".to_string());
                    }
                    if deps.contains_key("express") {
                        frameworks.push("Express".to_string());
                    }
                }
            }
        }

        // Check for package manager
        if tools.iter().any(|t| t.name == "npm") {
            package_managers.push("npm".to_string());
        }
        if tools.iter().any(|t| t.name == "yarn") {
            package_managers.push("yarn".to_string());
        }
        if tools.iter().any(|t| t.name == "pnpm") {
            package_managers.push("pnpm".to_string());
        }
        if tools.iter().any(|t| t.name == "bun") {
            package_managers.push("bun".to_string());
        }
    }

    // Check for Python
    if project_root.join("pyproject.toml").exists()
        || project_root.join("requirements.txt").exists()
        || project_root.join("setup.py").exists()
    {
        languages.push("Python".to_string());

        if tools.iter().any(|t| t.name == "uv") {
            package_managers.push("uv".to_string());
        }
        if tools.iter().any(|t| t.name == "pip") {
            package_managers.push("pip".to_string());
        }
    }

    // Check for Rust
    if project_root.join("Cargo.toml").exists() {
        languages.push("Rust".to_string());
        package_managers.push("cargo".to_string());
    }

    // Check for Go
    if project_root.join("go.mod").exists() {
        languages.push("Go".to_string());
        package_managers.push("go mod".to_string());
    }

    (languages, frameworks, package_managers)
}

// ============================================================================
// vx ai session (RFC 0035)
// ============================================================================

/// Handle `vx ai session` command
pub async fn handle_session(ctx: &CommandContext, command: &SessionCommand) -> Result<()> {
    match command {
        SessionCommand::Init => handle_session_init(ctx).await,
        SessionCommand::Status => handle_session_status(ctx).await,
        SessionCommand::Cleanup => handle_session_cleanup(ctx).await,
    }
}

/// Session state file path
fn session_file_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| std::env::current_dir().unwrap());
    home.join(".vx").join("ai-session.json")
}

/// Handle session init
async fn handle_session_init(_ctx: &CommandContext) -> Result<()> {
    let session_path = session_file_path();
    let session_dir = session_path.parent().unwrap();

    std::fs::create_dir_all(session_dir).context("Failed to create vx session directory")?;

    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    let session = serde_json::json!({
        "session_id": uuid::Uuid::new_v4().to_string(),
        "project_root": current_dir.display().to_string(),
        "created_at": chrono::Utc::now().to_rfc3339(),
        "active_tools": {},
        "last_check": chrono::Utc::now().to_rfc3339(),
    });

    let content = serde_json::to_string_pretty(&session)?;
    std::fs::write(&session_path, &content).context("Failed to write session file")?;

    UI::success(&format!(
        "AI session initialized: {}",
        session_path.display()
    ));
    UI::hint("Session state will be persisted across AI interactions");

    Ok(())
}

/// Handle session status
async fn handle_session_status(_ctx: &CommandContext) -> Result<()> {
    let session_path = session_file_path();

    if !session_path.exists() {
        UI::info("No active AI session");
        UI::hint("Run 'vx ai session init' to initialize a session");
        return Ok(());
    }

    let content = std::fs::read_to_string(&session_path).context("Failed to read session file")?;

    let session: serde_json::Value =
        serde_json::from_str(&content).context("Failed to parse session file")?;

    println!();
    UI::header("AI Session Status");
    println!();

    if session.get("session_id").is_some() {
        // Never expose any portion of the session ID for security
        println!("  Session ID: <redacted>");
    }

    if let Some(project_root) = session.get("project_root").and_then(|s| s.as_str()) {
        println!("  Project: {}", project_root);
    }

    if let Some(created_at) = session.get("created_at").and_then(|s| s.as_str()) {
        println!("  Created: {}", created_at);
    }

    if let Some(last_check) = session.get("last_check").and_then(|s| s.as_str()) {
        println!("  Last check: {}", last_check);
    }

    println!();

    Ok(())
}

/// Handle session cleanup
async fn handle_session_cleanup(_ctx: &CommandContext) -> Result<()> {
    let session_path = session_file_path();

    if !session_path.exists() {
        UI::info("No active AI session to clean up");
        return Ok(());
    }

    std::fs::remove_file(&session_path).context("Failed to remove session file")?;

    UI::success("AI session cleaned up");

    Ok(())
}
