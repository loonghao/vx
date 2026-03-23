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

use crate::cli::{OutputFormat, SessionCommand};
use crate::commands::CommandContext;
use crate::output::{
    AiContextOutput, ConstraintInfo, OutputRenderer, ProjectInfo, ScriptInfo, ToolInfo,
};
use crate::ui::UI;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for a single supported AI agent
struct AgentConfig {
    /// Agent identifier used in CLI arguments
    name: &'static str,
    /// Skills directory relative to the project root
    project_skills_dir: &'static str,
    /// Skills directory relative to the home directory
    global_skills_dir: &'static str,
}

/// All supported AI agent targets
const SUPPORTED_AGENTS: &[AgentConfig] = &[
    AgentConfig {
        name: "codebuddy",
        project_skills_dir: ".codebuddy/skills",
        global_skills_dir: ".codebuddy/skills",
    },
    AgentConfig {
        name: "claude-code",
        project_skills_dir: ".claude/skills",
        global_skills_dir: ".claude/skills",
    },
    AgentConfig {
        name: "cursor",
        project_skills_dir: ".cursor/skills",
        global_skills_dir: ".cursor/skills",
    },
    AgentConfig {
        name: "codex",
        project_skills_dir: ".agents/skills",
        global_skills_dir: ".codex/skills",
    },
    AgentConfig {
        name: "windsurf",
        project_skills_dir: ".windsurf/skills",
        global_skills_dir: ".codeium/windsurf/skills",
    },
    AgentConfig {
        name: "copilot",
        project_skills_dir: ".agents/skills",
        global_skills_dir: ".copilot/skills",
    },
    AgentConfig {
        name: "opencode",
        project_skills_dir: ".opencode/skills",
        global_skills_dir: ".config/opencode/skills",
    },
    AgentConfig {
        name: "trae",
        project_skills_dir: ".trae/skills",
        global_skills_dir: ".trae/skills",
    },
    AgentConfig {
        name: "gemini-cli",
        project_skills_dir: ".agents/skills",
        global_skills_dir: ".gemini/skills",
    },
    AgentConfig {
        name: "amp",
        project_skills_dir: ".agents/skills",
        global_skills_dir: ".config/agents/skills",
    },
    AgentConfig {
        name: "roo",
        project_skills_dir: ".roo/skills",
        global_skills_dir: ".roo/skills",
    },
    AgentConfig {
        name: "cline",
        project_skills_dir: ".cline/skills",
        global_skills_dir: ".cline/skills",
    },
    AgentConfig {
        name: "kiro-cli",
        project_skills_dir: ".kiro/skills",
        global_skills_dir: ".kiro/skills",
    },
];

/// All built-in vx skills, embedded at compile time.
///
/// Each tuple is `(skill_name, skill_content)`.
const VX_SKILLS: &[(&str, &str)] = &[
    ("vx-usage", include_str!("../skills/vx-usage/SKILL.md")),
    (
        "vx-commands",
        include_str!("../skills/vx-commands/SKILL.md"),
    ),
    ("vx-project", include_str!("../skills/vx-project/SKILL.md")),
    (
        "vx-troubleshooting",
        include_str!("../skills/vx-troubleshooting/SKILL.md"),
    ),
    (
        "vx-best-practices",
        include_str!("../skills/vx-best-practices/SKILL.md"),
    ),
];

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

    for agent in &target_agents {
        let dirs_to_install: Vec<PathBuf> = if global {
            vec![home_dir.join(agent.global_skills_dir)]
        } else {
            // Install to project-level directory by default
            vec![cwd.join(agent.project_skills_dir)]
        };

        for target_dir in dirs_to_install {
            let mut all_up_to_date = true;

            for (skill_name, skill_content) in VX_SKILLS {
                let skill_dir = target_dir.join(skill_name);
                let skill_file = skill_dir.join("SKILL.md");

                if skill_file.exists() && !force {
                    UI::info(&format!(
                        "  {} - {} already installed",
                        agent.name, skill_name
                    ));
                    continue;
                }

                // Create directory and write SKILL.md
                std::fs::create_dir_all(&skill_dir).with_context(|| {
                    format!("Failed to create directory: {}", skill_dir.display())
                })?;

                std::fs::write(&skill_file, skill_content).with_context(|| {
                    format!("Failed to write SKILL.md to {}", skill_file.display())
                })?;

                UI::success(&format!(
                    "  {} - installed {} to {}",
                    agent.name,
                    skill_name,
                    skill_dir.display()
                ));
                all_up_to_date = false;
                installed_count += 1;
            }

            if all_up_to_date {
                UI::info(&format!(
                    "  {} - all skills up to date at {}",
                    agent.name,
                    target_dir.display()
                ));
            }
        }
    }

    println!();
    if installed_count > 0 {
        UI::success(&format!(
            "Installed {} skill file(s) across {} skill(s)",
            installed_count,
            VX_SKILLS.len()
        ));
    } else {
        UI::info("All skills already up to date. Use --force to reinstall.");
    }

    UI::hint("AI agents will now have access to all vx skills.");

    Ok(())
}

/// Handle `vx ai agents` command - list supported AI agents
pub async fn handle_agents() -> Result<()> {
    UI::header("Supported AI Agents");
    println!();
    let header = format!("  {:<16} {:<24} GLOBAL DIR", "AGENT", "PROJECT DIR");
    println!("{header}");
    println!("  {}", "-".repeat(68));

    for agent in SUPPORTED_AGENTS {
        println!(
            "  {:<16} {:<24} ~/{}",
            agent.name, agent.project_skills_dir, agent.global_skills_dir
        );
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

    let mut cmd_args = vec!["npx".to_string(), "-y".to_string(), "skills".to_string()];
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
fn resolve_agents(agents: &[String]) -> Result<Vec<&'static AgentConfig>> {
    if agents.is_empty() {
        return Ok(SUPPORTED_AGENTS.iter().collect());
    }

    let mut result = Vec::new();
    for name in agents {
        match SUPPORTED_AGENTS.iter().find(|a| a.name == name.as_str()) {
            Some(agent) => result.push(agent),
            None => {
                let available: Vec<&str> = SUPPORTED_AGENTS.iter().map(|a| a.name).collect();
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

/// Collect project scripts from vx.toml and package.json
async fn collect_project_scripts(project_root: &std::path::Path) -> Result<Vec<ScriptInfo>> {
    let mut scripts = Vec::new();

    // Check for vx.toml scripts using the proper config parser
    let vx_toml = project_root.join("vx.toml");
    if vx_toml.exists()
        && let Ok(config) = vx_config::parse_config(&vx_toml)
    {
        for (name, script) in &config.scripts {
            let command = match script {
                vx_config::ScriptConfig::Simple(cmd) => cmd.clone(),
                vx_config::ScriptConfig::Detailed(d) => d.command.clone(),
            };
            scripts.push(ScriptInfo {
                name: name.clone(),
                command,
                description: None,
                tools: vec![],
            });
        }
    }

    // Check for package.json scripts
    let package_json = project_root.join("package.json");
    if package_json.exists()
        && let Ok(content) = std::fs::read_to_string(&package_json)
        && let Ok(json) = serde_json::from_str::<serde_json::Value>(&content)
        && let Some(scripts_obj) = json.get("scripts").and_then(|s| s.as_object())
    {
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

    Ok(scripts)
}

/// Collect tool constraints from vx.toml using the proper config parser
fn collect_tool_constraints(project_root: &std::path::Path) -> Result<Vec<ConstraintInfo>> {
    let vx_toml = project_root.join("vx.toml");
    if !vx_toml.exists() {
        return Ok(vec![]);
    }

    let config =
        vx_config::parse_config(&vx_toml).context("Failed to parse vx.toml for constraints")?;

    let constraints = config
        .tools_as_hashmap()
        .into_iter()
        .map(|(tool, version)| ConstraintInfo {
            tool,
            constraint: version,
            reason: None,
            satisfied: true, // TODO: Check if actually satisfied
        })
        .collect();

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
        if let Ok(content) = std::fs::read_to_string(project_root.join("package.json"))
            && let Ok(json) = serde_json::from_str::<serde_json::Value>(&content)
            && let Some(deps) = json.get("dependencies").and_then(|d| d.as_object())
        {
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
    let home = dirs::home_dir()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".vx").join("ai-session.json")
}

/// Handle session init
async fn handle_session_init(_ctx: &CommandContext) -> Result<()> {
    let session_path = session_file_path();
    let session_dir = session_path
        .parent()
        .expect("session file path should have a parent directory");

    std::fs::create_dir_all(session_dir).context("Failed to create vx session directory")?;

    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
    let now = chrono::Utc::now().to_rfc3339();

    let session = serde_json::json!({
        "session_id": uuid::Uuid::new_v4().to_string(),
        "project_root": current_dir.display().to_string(),
        "created_at": now,
        "active_tools": {},
        "last_check": now,
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
