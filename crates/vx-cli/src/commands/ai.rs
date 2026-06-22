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

use crate::cli::{HeadroomCommand, HeadroomMcpCommand, HeadroomProxyCommand};
use crate::cli::{OutputFormat, SessionCommand};
use crate::commands::CommandContext;
use crate::output::{
    AiContextOutput, ConstraintInfo, OutputRenderer, ProjectInfo, ScriptInfo, ToolInfo,
};
use crate::ui::UI;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::Duration;
use toml_edit::{DocumentMut, Item, Table, value};

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
/// Skills are stored in the top-level `skills/` directory — the single source of truth
/// shared between `vx ai setup`, ClawHub publishing, and agent config directories.
///
/// Each tuple is `(skill_name, skill_content)`.
const VX_SKILLS: &[(&str, &str)] = &[
    (
        "vx-usage",
        include_str!("../../../../skills/vx-usage/SKILL.md"),
    ),
    (
        "vx-commands",
        include_str!("../../../../skills/vx-commands/SKILL.md"),
    ),
    (
        "vx-project",
        include_str!("../../../../skills/vx-project/SKILL.md"),
    ),
    (
        "vx-troubleshooting",
        include_str!("../../../../skills/vx-troubleshooting/SKILL.md"),
    ),
    (
        "vx-best-practices",
        include_str!("../../../../skills/vx-best-practices/SKILL.md"),
    ),
];

/// Handle `vx ai setup` command
///
/// Installs vx's built-in skills to the target AI agent configuration directories,
/// so that AI coding agents can better understand and use vx.
pub async fn handle_setup(
    agents: &[String],
    global: bool,
    project: bool,
    force: bool,
) -> Result<()> {
    let target_agents = resolve_agents(agents)?;
    let install_global = global || !project;
    let skills_hash = compute_skills_hash();

    UI::header("Setting up vx skills for AI agents");
    println!();

    let home_dir = ai_home_dir()?;
    let cwd = std::env::current_dir().context("Could not determine current directory")?;

    let mut installed_count = 0;
    let mut skipped_outdated_count = 0;

    for agent in &target_agents {
        let dirs_to_install: Vec<PathBuf> = if install_global {
            vec![home_dir.join(agent.global_skills_dir)]
        } else {
            vec![cwd.join(agent.project_skills_dir)]
        };

        for target_dir in dirs_to_install {
            let mut all_up_to_date = true;

            for (skill_name, skill_content) in VX_SKILLS {
                let skill_dir = target_dir.join(skill_name);
                let skill_file = skill_dir.join("SKILL.md");

                if skill_file.exists() && !force {
                    let existing = std::fs::read_to_string(&skill_file).unwrap_or_default();
                    if existing != *skill_content {
                        UI::warn(&format!(
                            "  {} - {} is outdated at {} (use --force to update)",
                            agent.name,
                            skill_name,
                            skill_file.display()
                        ));
                        skipped_outdated_count += 1;
                        continue;
                    }

                    UI::info(&format!(
                        "  {} - {} already up to date",
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

    if project {
        if skipped_outdated_count == 0 {
            record_project_skills_hash(&cwd, &skills_hash)?;
            UI::success(&format!(
                "Recorded vx skills hash in vx.toml: {}",
                short_hash(&skills_hash)
            ));
        } else {
            UI::warn("Skipped recording skills hash because some project skills are outdated.");
            UI::hint("Run `vx ai setup --project --force` to refresh skills and update vx.toml.");
        }
    }

    UI::hint("AI agents will now have access to all vx skills.");

    Ok(())
}

/// Handle `vx ai check` command.
///
/// Compares the built-in vx skills hash with the hash recorded in the current
/// project's `vx.toml` under `[ai].skills_hash`.
pub async fn handle_check() -> Result<()> {
    let cwd = std::env::current_dir().context("Could not determine current directory")?;
    let vx_toml = cwd.join("vx.toml");
    let current_hash = compute_skills_hash();

    UI::header("Checking vx skills");
    println!();

    if !vx_toml.exists() {
        UI::warn("No vx.toml found in the current directory.");
        UI::hint("Run `vx ai setup --project` to install project skills and record their hash.");
        return Ok(());
    }

    let config = vx_config::parse_config(&vx_toml).context("Failed to parse vx.toml")?;
    let recorded_hash = config.ai.and_then(|ai| ai.skills_hash);

    match recorded_hash {
        Some(hash) if hash == current_hash => {
            UI::success(&format!(
                "Project vx skills are up to date ({})",
                short_hash(&current_hash)
            ));
        }
        Some(hash) => {
            UI::warn(&format!(
                "Project vx skills are outdated (recorded {}, current {})",
                short_hash(&hash),
                short_hash(&current_hash)
            ));
            UI::hint("Run `vx ai setup --project --force` to refresh project skills.");
        }
        None => {
            UI::warn("No [ai].skills_hash recorded in vx.toml.");
            UI::hint(
                "Run `vx ai setup --project` to install project skills and record their hash.",
            );
        }
    }

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

/// Stable SHA-256 hash of all embedded vx skills.
pub fn compute_skills_hash() -> String {
    let mut hasher = Sha256::new();
    for (skill_name, skill_content) in VX_SKILLS {
        hasher.update(skill_name.as_bytes());
        hasher.update([0]);
        hasher.update(skill_content.as_bytes());
        hasher.update([0]);
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn short_hash(hash: &str) -> &str {
    hash.get(..12).unwrap_or(hash)
}

fn record_project_skills_hash(project_root: &std::path::Path, skills_hash: &str) -> Result<()> {
    let config_path = project_root.join("vx.toml");
    let original = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read {}", config_path.display()))?
    } else {
        String::new()
    };
    let mut doc: DocumentMut = original
        .parse()
        .context("Failed to parse vx.toml for skills hash update")?;

    if !doc.as_table().contains_key("ai") {
        doc["ai"] = Item::Table(Table::new());
    }
    doc["ai"]["skills_hash"] = value(skills_hash);
    doc["ai"]["skills_updated_at"] = value(chrono::Utc::now().to_rfc3339());

    std::fs::write(&config_path, doc.to_string())
        .with_context(|| format!("Failed to write {}", config_path.display()))?;

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

fn ai_home_dir() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("VX_AI_HOME") {
        return Ok(PathBuf::from(path));
    }
    dirs::home_dir().context("Could not determine home directory")
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

// ============================================================================
// vx ai headroom (PIP-584 Phase 1)
// ============================================================================

/// Default headroom version
const DEFAULT_HEADROOM_VERSION: &str = "0.22.3";
/// Default Python version for headroom runtime
#[allow(dead_code)]
const DEFAULT_PYTHON_VERSION: &str = "3.11";
/// Default mcpcall version
#[allow(dead_code)]
const DEFAULT_MCPCALL_VERSION: &str = "0.4.0";

/// Handle `vx ai headroom` command — dispatches to subcommands.
pub async fn handle_headroom(ctx: &CommandContext, command: &HeadroomCommand) -> Result<()> {
    match command {
        HeadroomCommand::Install {
            version,
            python,
            mcpcall_version,
            force,
        } => handle_headroom_install(version, python, mcpcall_version, *force).await,
        HeadroomCommand::Doctor {
            quick,
            json,
            port,
            mcp_port,
        } => handle_headroom_doctor(*quick, *json, *port, *mcp_port).await,
        HeadroomCommand::Setup {
            agent,
            dry_run,
            apply,
            port,
            mcp_port,
            headroom_version,
        } => {
            handle_headroom_setup(agent, *dry_run, *apply, *port, *mcp_port, headroom_version).await
        }
        HeadroomCommand::Proxy(proxy) => handle_headroom_proxy(proxy).await,
        HeadroomCommand::Mcp(mcp) => handle_headroom_mcp(ctx, mcp).await,
    }
}

/// Resolve the actual version string (replaces "latest" with the default)
fn resolve_version(version: &str) -> &str {
    if version == "latest" {
        DEFAULT_HEADROOM_VERSION
    } else {
        version
    }
}

/// Handle `vx ai headroom install` — installs headroom-ai[proxy] via uv tool install.
///
/// Bridge runner: delegates to `vx uv tool install --python <version>
/// --from 'headroom-ai[proxy]==<version>' headroom`.
/// Also ensures mcpcall is available.
async fn handle_headroom_install(
    version: &str,
    python: &str,
    mcpcall_version: &str,
    force: bool,
) -> Result<()> {
    let vx_exe = std::env::current_exe().context("Could not determine vx executable path")?;
    let resolved_version = resolve_version(version);

    UI::header("Installing headroom-ai");
    println!();

    // Step 1: Install headroom via uv tool install
    // Equivalent to: vx uv tool install --python <python> --from 'headroom-ai[proxy]==<version>' headroom
    let headroom_spec = format!("headroom-ai[proxy]=={}", resolved_version);

    let mut install_args = vec![
        "uv".to_string(),
        "tool".to_string(),
        "install".to_string(),
        "--python".to_string(),
        python.to_string(),
        "--from".to_string(),
        headroom_spec.clone(),
        "headroom".to_string(),
    ];

    if force {
        install_args.push("--force".to_string());
    }

    UI::info(&format!(
        "Installing {} with Python {}...",
        headroom_spec, python
    ));

    let install_status = std::process::Command::new(&vx_exe)
        .args(&install_args)
        .status()
        .context("Failed to execute 'vx uv tool install' for headroom-ai")?;

    if !install_status.success() {
        let code = install_status.code().unwrap_or(1);
        UI::error(&format!("headroom install failed with exit code {}", code));
        UI::hint(
            "Try running manually: vx uv tool install --python <version> --from 'headroom-ai[proxy]==<version>' headroom",
        );
        return Err(anyhow::anyhow!(
            "headroom install failed with exit code {}",
            code
        ));
    }

    UI::success(&format!(
        "headroom-ai[proxy] {} installed successfully",
        resolved_version
    ));

    // Step 2: Ensure mcpcall is available
    UI::info(&format!(
        "Ensuring mcpcall {} is available...",
        mcpcall_version
    ));

    let mcpcall_status = std::process::Command::new(&vx_exe)
        .args(["install", &format!("mcpcall@{}", mcpcall_version)])
        .status()
        .context("Failed to install mcpcall")?;

    if mcpcall_status.success() {
        UI::success(&format!("mcpcall {} ready", mcpcall_version));
    } else {
        UI::warn(&format!(
            "mcpcall {} may not be installed — MCP smoke tests may fail",
            mcpcall_version
        ));
    }

    // Step 3: Run quick doctor check
    UI::info("Running quick health check...");
    handle_headroom_doctor(true, false, 8787, 8765).await?;

    println!();
    UI::success("headroom-ai installation complete");
    UI::hint("Run 'vx ai headroom doctor' to verify the environment");
    UI::hint("Run 'vx ai headroom proxy start' to start the proxy");
    UI::hint("Run 'vx ai headroom mcp test' to validate MCP tools");

    Ok(())
}

/// Handle `vx ai headroom doctor` — environment diagnostic checks.
///
/// Three-layer check: environment, proxy, MCP.
/// Environment layer is fully implemented; proxy and MCP layers are stubs (PIP-602/PIP-603).
/// When `--json` is set, outputs pure machine-readable JSON on stdout.
async fn handle_headroom_doctor(quick: bool, json: bool, port: u16, mcp_port: u16) -> Result<()> {
    let vx_exe = std::env::current_exe().context("Could not determine vx executable path")?;

    if !json {
        UI::header("headroom doctor");
        println!();
        UI::info("--- Environment ---");
    }

    // Layer 1: Environment checks (always run — data collection)
    let uv_check = std::process::Command::new(&vx_exe)
        .args(["uv", "--version"])
        .output();
    let uv_version = match &uv_check {
        Ok(output) if output.status.success() => {
            let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !json {
                UI::success(&format!("uv: {}", ver));
            }
            Some(ver)
        }
        _ => {
            if !json {
                UI::error("uv: not available");
            }
            None
        }
    };

    let py_check = std::process::Command::new(&vx_exe)
        .args(["python", "--version"])
        .output();
    let py_version = match &py_check {
        Ok(output) if output.status.success() => {
            let ver = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let ver = if ver.is_empty() {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                ver
            };
            if !json {
                UI::success(&format!("python: {}", ver));
            }
            Some(ver)
        }
        _ => {
            if !json {
                UI::warn("python: check skipped (may not be installed via vx)");
            }
            None
        }
    };

    let hr_check = std::process::Command::new(&vx_exe)
        .args([
            "uv",
            "tool",
            "run",
            "--from",
            &format!("headroom-ai[proxy]=={}", DEFAULT_HEADROOM_VERSION),
            "headroom",
            "--version",
        ])
        .output();
    let hr_version = match &hr_check {
        Ok(output) if output.status.success() => {
            let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !json {
                UI::success(&format!("headroom-ai: {}", ver));
            }
            Some(ver)
        }
        _ => {
            if !json {
                UI::error(&format!(
                    "headroom-ai[proxy] {}: not installed",
                    DEFAULT_HEADROOM_VERSION
                ));
            }
            None
        }
    };

    let mcpcall_check = std::process::Command::new(&vx_exe)
        .args(["mcpcall", "--version"])
        .output();
    let mcpcall_version = match &mcpcall_check {
        Ok(output) if output.status.success() => {
            let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !json {
                UI::success(&format!("mcpcall: {}", ver));
            }
            Some(ver)
        }
        _ => {
            if !json {
                UI::warn("mcpcall: not installed (run 'vx ai headroom install' to set up)");
            }
            None
        }
    };

    // Layer 2 & 3: Proxy and MCP checks (run for both JSON and text output)
    let (proxy_status_msg, proxy_health, proxy_stats) = if !quick {
        check_proxy_layer(port).await
    } else {
        (None, None, None)
    };
    let (mcp_running, mcp_tools, mcp_cr_result) = if !quick {
        check_mcp_layer(mcp_port).await
    } else {
        (false, vec![], None)
    };

    // Build and emit JSON output when requested
    if json {
        let mut env = serde_json::Map::new();
        env.insert("uv".into(), serde_json::json!(uv_version));
        env.insert("python".into(), serde_json::json!(py_version));
        env.insert("headroom".into(), serde_json::json!(hr_version));
        env.insert("mcpcall".into(), serde_json::json!(mcpcall_version));

        let mut result = serde_json::Map::new();
        result.insert("environment".into(), serde_json::Value::Object(env));
        result.insert(
            "headroom_version".into(),
            serde_json::json!(DEFAULT_HEADROOM_VERSION),
        );

        if !quick {
            result.insert(
                "proxy".into(),
                serde_json::json!({
                    "status": proxy_health.as_deref().unwrap_or(
                        if proxy_status_msg.as_deref() == Some("running") {
                            "port_in_use"
                        } else {
                            "not_running"
                        }
                    ),
                    "port": port,
                    "health": proxy_health,
                    "stats": proxy_stats,
                }),
            );
            result.insert(
                "mcp".into(),
                serde_json::json!({
                    "running": mcp_running,
                    "port": mcp_port,
                    "tools": mcp_tools,
                    "compress_retrieve": mcp_cr_result,
                }),
            );
        }

        println!("{}", serde_json::Value::Object(result));
        return Ok(());
    }

    if quick {
        UI::hint("Quick check complete. Run without --quick for proxy and MCP checks.");
        return Ok(());
    }

    // Layer 2: Proxy checks (text output)
    println!();
    UI::info(&format!("--- Proxy (port {}) ---", port));

    match proxy_status_msg.as_deref() {
        Some("running") => {
            UI::success("Proxy is running");
            if let Some(health) = &proxy_health {
                UI::success(&format!("/health: {}", health));
            }
            if let Some(stats) = &proxy_stats {
                UI::success(&format!("/stats: {}", stats));
            }
        }
        Some("port_in_use") => {
            UI::warn(&format!(
                "Port {} is in use but /health did not respond",
                port
            ));
            if let Some(health) = &proxy_health {
                UI::success(&format!("/health: {}", health));
            }
        }
        None | Some("not_running") => {
            UI::warn(&format!("Port {} is free — proxy is not running.", port));
            UI::hint("Run 'vx ai headroom proxy start' to start the proxy service.");
        }
        Some(other) => {
            UI::warn(&format!("Proxy status: {}", other));
        }
    }

    // Layer 3: MCP checks (text output)
    println!();
    UI::info(&format!("--- MCP (port {}) ---", mcp_port));

    if mcp_running {
        UI::success(&format!("MCP server detected on port {}", mcp_port));
        if !mcp_tools.is_empty() {
            UI::success(&format!("Tools available: {}", mcp_tools.join(", ")));
        } else {
            UI::warn("No expected MCP tools found via mcpcall list");
        }
        match &mcp_cr_result {
            Some(r) if r == "ok" => {
                UI::success("compress \u{2192} retrieve round-trip: PASSED");
            }
            Some(r) => {
                UI::warn(&format!("compress \u{2192} retrieve: {}", r));
            }
            None => {
                UI::warn("compress \u{2192} retrieve: not tested");
            }
        }
    } else {
        UI::warn(&format!(
            "Port {} is free — MCP server is not running.",
            mcp_port
        ));
        UI::hint("Run 'vx ai headroom mcp test' to validate MCP tools.");
    }

    Ok(())
}

// ============================================================================
// Helper functions for doctor checks
// ============================================================================

/// Check if a TCP port is free (nothing listening). Returns `true` if free.
fn is_port_free(port: u16) -> bool {
    TcpStream::connect_timeout(
        &format!("127.0.0.1:{}", port).parse().unwrap(),
        Duration::from_millis(500),
    )
    .is_err()
}

/// Make a simple HTTP GET request to a local endpoint. Returns the status line and body.
async fn check_http_endpoint(port: u16, path: &str) -> Result<String> {
    use std::io::Read;

    let mut stream = TcpStream::connect_timeout(
        &format!("127.0.0.1:{}", port).parse()?,
        Duration::from_secs(2),
    )?;

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
        path, port
    );
    stream.write_all(request.as_bytes())?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    let status_line = response.lines().next().unwrap_or("unknown");
    let body = response
        .split("\r\n\r\n")
        .nth(1)
        .unwrap_or("")
        .trim()
        .to_string();

    Ok(if body.is_empty() {
        status_line.to_string()
    } else {
        format!("{} | body: {}", status_line, body)
    })
}

/// Check proxy layer: port status, /health, /stats endpoints.
async fn check_proxy_layer(port: u16) -> (Option<String>, Option<String>, Option<String>) {
    if is_port_free(port) {
        return (None, None, None); // not_running
    }

    let health = check_http_endpoint(port, "/health")
        .await
        .ok()
        .filter(|s| !s.starts_with("unknown") && !s.contains("Connection refused"));

    let stats = check_http_endpoint(port, "/stats")
        .await
        .ok()
        .filter(|s| !s.starts_with("unknown") && !s.contains("Connection refused"));

    let status = if health.is_some() {
        Some("running".to_string())
    } else {
        Some("port_in_use".to_string())
    };

    (status, health, stats)
}

/// Check MCP layer: port status, tool listing, compress→retrieve round-trip.
async fn check_mcp_layer(mcp_port: u16) -> (bool, Vec<String>, Option<String>) {
    let vx_exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(_) => return (false, vec![], None),
    };

    if is_port_free(mcp_port) {
        return (false, vec![], None);
    }

    let mcp_url = format!("http://127.0.0.1:{}/mcp", mcp_port);

    // Try mcpcall list to discover tools
    let tools = match std::process::Command::new(&vx_exe)
        .args(["mcpcall", "--url", &mcp_url, "list"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{}{}", stdout, stderr);
            extract_mcp_tool_names(&combined)
        }
        _ => vec![],
    };

    if tools.is_empty() {
        // Port is occupied but mcpcall didn't return tools — server might not be MCP
        return (true, vec![], Some("mcpcall list returned no tools".into()));
    }

    // Try compress → retrieve round-trip
    let cr_result = match test_mcp_compress_retrieve(&vx_exe, &mcp_url).await {
        Ok(true) => Some("ok".into()),
        Ok(false) => Some("retrieved content does not match original".into()),
        Err(e) => Some(format!("error: {}", e)),
    };

    (true, tools, cr_result)
}

/// Extract MCP tool names from mcpcall list output.
fn extract_mcp_tool_names(output: &str) -> Vec<String> {
    let expected = ["compress", "retrieve", "stats"];
    let mut found: Vec<String> = Vec::new();

    for kw in &expected {
        let lower_output = output.to_lowercase();
        if !lower_output.contains(kw) {
            continue;
        }
        // Try to find a line containing both "headroom" and the keyword
        let mut matched = false;
        for line in output.lines() {
            let trimmed = line.trim();
            let lower = trimmed.to_lowercase();
            if lower.contains(*kw) && (lower.contains("headroom") || lower.contains("mcp")) {
                found.push(trimmed.to_string());
                matched = true;
                break;
            }
        }
        if !matched {
            found.push(format!("headroom_{}", kw));
        }
    }

    found
}

/// Test MCP compress → retrieve round-trip via mcpcall.
async fn test_mcp_compress_retrieve(vx_exe: &std::path::Path, mcp_url: &str) -> Result<bool> {
    let sample = "Hello, headroom MCP round-trip test!";

    // Compress
    let compress_output = std::process::Command::new(vx_exe)
        .args([
            "mcpcall",
            "--url",
            mcp_url,
            "call",
            "headroom_compress",
            "--args",
            &format!(r#"{{"content": "{}"}}"#, sample),
        ])
        .output()
        .context("Failed to run mcpcall compress")?;

    if !compress_output.status.success() {
        let stderr = String::from_utf8_lossy(&compress_output.stderr);
        return Err(anyhow::anyhow!("mcpcall compress failed: {}", stderr));
    }

    let compress_text = String::from_utf8_lossy(&compress_output.stdout)
        .trim()
        .to_string();
    let hash = compress_text
        .lines()
        .last()
        .unwrap_or(&compress_text)
        .trim();
    if hash.is_empty() {
        return Err(anyhow::anyhow!("empty response from compress"));
    }

    // Retrieve
    let retrieve_output = std::process::Command::new(vx_exe)
        .args([
            "mcpcall",
            "--url",
            mcp_url,
            "call",
            "headroom_retrieve",
            "--args",
            &format!(r#"{{"hash": "{}"}}"#, hash),
        ])
        .output()
        .context("Failed to run mcpcall retrieve")?;

    if !retrieve_output.status.success() {
        let stderr = String::from_utf8_lossy(&retrieve_output.stderr);
        return Err(anyhow::anyhow!("mcpcall retrieve failed: {}", stderr));
    }

    let retrieve_text = String::from_utf8_lossy(&retrieve_output.stdout)
        .trim()
        .to_string();

    Ok(retrieve_text.contains(sample))
}

/// Handle `vx ai headroom setup` — generate MCP config templates.
///
/// Stub in PIP-601; full implementation in PIP-604.
pub async fn handle_headroom_setup(
    agents: &[String],
    _dry_run: bool,
    apply: bool,
    port: u16,
    mcp_port: u16,
    headroom_version: &str,
) -> Result<()> {
    UI::header("headroom setup");
    println!();
    UI::info("setup: not yet implemented (PIP-604)");
    UI::info(&format!(
        "Would configure MCP for agents: {}",
        if agents.is_empty() {
            "codex, claude-code, cursor (default)"
        } else {
            ""
        }
    ));
    if !agents.is_empty() {
        UI::info(&format!("  requested agents: {}", agents.join(", ")));
    }
    UI::info(&format!("  proxy port: {}", port));
    UI::info(&format!("  MCP port: {}", mcp_port));
    UI::info(&format!("  headroom version: {}", headroom_version));
    UI::info(&format!(
        "  mode: {}",
        if apply {
            "--apply"
        } else {
            "--dry-run (default)"
        }
    ));
    UI::hint("This command will be fully implemented in PIP-604.");
    Ok(())
}

/// Handle `vx ai headroom proxy` — dispatch to proxy subcommands.
///
/// Stubs in PIP-601; full implementation in PIP-602.
async fn handle_headroom_proxy(command: &HeadroomProxyCommand) -> Result<()> {
    match command {
        HeadroomProxyCommand::Start {
            host,
            port,
            foreground,
            log_file,
            no_optimize,
        } => {
            UI::header("headroom proxy start");
            println!();
            UI::info("proxy start: not yet implemented (PIP-602)");
            UI::info(&format!("  host: {}", host));
            UI::info(&format!("  port: {}", port));
            UI::info(&format!("  foreground: {}", foreground));
            if let Some(log) = log_file {
                UI::info(&format!("  log file: {}", log));
            }
            UI::info(&format!("  optimize: {}", !no_optimize));
            UI::hint("This command will be fully implemented in PIP-602.");
        }
        HeadroomProxyCommand::Status { port, json } => {
            if *json {
                println!(
                    "{}",
                    serde_json::json!({"status": "not_implemented", "port": port})
                );
            } else {
                UI::header("headroom proxy status");
                println!();
                UI::info("proxy status: not yet implemented (PIP-602)");
                UI::info(&format!("  checking port: {}", port));
                UI::hint("This command will be fully implemented in PIP-602.");
            }
        }
        HeadroomProxyCommand::Stop { port } => {
            UI::header("headroom proxy stop");
            println!();
            UI::info("proxy stop: not yet implemented (PIP-602)");
            UI::info(&format!("  stopping on port: {}", port));
            UI::hint("This command will be fully implemented in PIP-602.");
        }
    }
    Ok(())
}

/// Handle `vx ai headroom mcp` — dispatch to MCP subcommands.
async fn handle_headroom_mcp(_ctx: &CommandContext, command: &HeadroomMcpCommand) -> Result<()> {
    match command {
        HeadroomMcpCommand::Stdio => {
            // Internal bridge entry point for agent MCP configurations.
            // Launches headroom MCP server in stdio mode and relays stdin/stdout.
            let vx_exe =
                std::env::current_exe().context("Could not determine vx executable path")?;

            let mut child = std::process::Command::new(&vx_exe)
                .args([
                    "uv",
                    "tool",
                    "run",
                    "--from",
                    &format!("headroom-ai[proxy]=={}", DEFAULT_HEADROOM_VERSION),
                    "headroom",
                    "mcp",
                ])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::inherit())
                .spawn()
                .context("Failed to start headroom MCP server")?;

            let mut child_stdin = child.stdin.take().unwrap();
            let mut child_stdout = child.stdout.take().unwrap();

            // Pipe parent stdin → headroom stdin
            let parent_stdin = std::io::stdin();
            std::thread::spawn(move || {
                let _ = std::io::copy(&mut parent_stdin.lock(), &mut child_stdin);
            });

            // Pipe headroom stdout → parent stdout
            let parent_stdout = std::io::stdout();
            std::thread::spawn(move || {
                let _ = std::io::copy(&mut child_stdout, &mut parent_stdout.lock());
            });

            let status = child.wait().context("Headroom MCP server stalled")?;
            std::process::exit(status.code().unwrap_or(1));
        }
        HeadroomMcpCommand::Test {
            url,
            json,
            sample_file,
        } => {
            if !json {
                UI::header("headroom mcp test");
                println!();
                UI::info(&format!("MCP URL: {}", url));
            }

            let vx_exe =
                std::env::current_exe().context("Could not determine vx executable path")?;

            // Step 1: List tools via mcpcall
            if !json {
                UI::info("--- Listing MCP tools ---");
            }

            let list_output = std::process::Command::new(&vx_exe)
                .args(["mcpcall", "--url", url, "list"])
                .output()
                .context("Failed to run mcpcall list")?;

            let tools_found = if list_output.status.success() {
                let stdout = String::from_utf8_lossy(&list_output.stdout);
                let stderr = String::from_utf8_lossy(&list_output.stderr);
                let combined = format!("{}{}", stdout, stderr);
                let tool_names = extract_mcp_tool_names(&combined);

                if !json {
                    if tool_names.is_empty() {
                        UI::warn(
                            "mcpcall list ran but no expected tools (headroom_compress, headroom_retrieve, headroom_stats) found",
                        );
                        println!("  raw output: {}", combined.trim());
                    } else {
                        UI::success(&format!("Found MCP tools: {}", tool_names.join(", ")));
                    }
                }
                tool_names
            } else {
                let stderr = String::from_utf8_lossy(&list_output.stderr);
                if !json {
                    UI::error(&format!("mcpcall list failed: {}", stderr.trim()));
                }
                vec![]
            };

            // Step 2: Read sample content
            let sample_content: String = if let Some(sample_path) = sample_file {
                std::fs::read_to_string(sample_path).context("Failed to read sample file")?
            } else {
                "Hello, headroom MCP test! This is sample content for testing the compress and retrieve round-trip.".to_string()
            };

            // Step 3: Test compress
            if !json {
                println!();
                UI::info("--- Testing headroom_compress ---");
            }

            let compress_output = std::process::Command::new(&vx_exe)
                .args([
                    "mcpcall",
                    "--url",
                    url,
                    "call",
                    "headroom_compress",
                    "--args",
                    &format!(r#"{{"content": "{}"}}"#, sample_content),
                ])
                .output()
                .context("Failed to run mcpcall compress")?;

            let compress_ok = compress_output.status.success();
            let compress_text = if compress_ok {
                let t = String::from_utf8_lossy(&compress_output.stdout)
                    .trim()
                    .to_string();
                if t.is_empty() {
                    String::from_utf8_lossy(&compress_output.stderr)
                        .trim()
                        .to_string()
                } else {
                    t
                }
            } else {
                String::from_utf8_lossy(&compress_output.stderr)
                    .trim()
                    .to_string()
            };

            if !json {
                if compress_ok {
                    UI::success(&format!("compress result: {}", compress_text));
                } else {
                    UI::error(&format!("compress failed: {}", compress_text));
                }
            }

            // Step 4: Test retrieve
            if !json {
                println!();
                UI::info("--- Testing headroom_retrieve ---");
            }

            let hash = compress_text
                .lines()
                .last()
                .unwrap_or(&compress_text)
                .trim();
            let retrieve_ok = if compress_ok && !hash.is_empty() {
                let retrieve_output = std::process::Command::new(&vx_exe)
                    .args([
                        "mcpcall",
                        "--url",
                        url,
                        "call",
                        "headroom_retrieve",
                        "--args",
                        &format!(r#"{{"hash": "{}"}}"#, hash),
                    ])
                    .output()
                    .context("Failed to run mcpcall retrieve")?;

                let retrieve_text = String::from_utf8_lossy(&retrieve_output.stdout)
                    .trim()
                    .to_string();
                let retrieve_text = if retrieve_text.is_empty() {
                    String::from_utf8_lossy(&retrieve_output.stderr)
                        .trim()
                        .to_string()
                } else {
                    retrieve_text
                };

                if retrieve_output.status.success() {
                    let roundtrip_ok = retrieve_text.contains(&sample_content);
                    if !json {
                        if roundtrip_ok {
                            UI::success("retrieve: content matches original");
                        } else {
                            UI::warn(&format!("retrieve result: {}", retrieve_text));
                        }
                    }
                    roundtrip_ok
                } else {
                    if !json {
                        UI::error(&format!("retrieve failed: {}", retrieve_text));
                    }
                    false
                }
            } else {
                if !json {
                    UI::error("Cannot test retrieve: compress step failed or hash was empty");
                }
                false
            };

            // Step 5: Test stats
            if !json {
                println!();
                UI::info("--- Testing headroom_stats ---");
            }

            let stats_output = std::process::Command::new(&vx_exe)
                .args([
                    "mcpcall",
                    "--url",
                    url,
                    "call",
                    "headroom_stats",
                    "--args",
                    "{}",
                ])
                .output()
                .context("Failed to run mcpcall stats")?;

            if !json {
                if stats_output.status.success() {
                    let stats_text = String::from_utf8_lossy(&stats_output.stdout);
                    let stats_text = if stats_text.trim().is_empty() {
                        String::from_utf8_lossy(&stats_output.stderr)
                    } else {
                        stats_text
                    };
                    UI::success(&format!("stats: {}", stats_text.trim()));
                } else {
                    let err = String::from_utf8_lossy(&stats_output.stderr);
                    UI::error(&format!("stats failed: {}", err.trim()));
                }
            }

            // JSON output
            if *json {
                let result = serde_json::json!({
                    "url": url,
                    "tools_found": tools_found,
                    "compress_ok": compress_ok,
                    "retrieve_ok": retrieve_ok,
                    "stats_ok": stats_output.status.success(),
                    "roundtrip_ok": compress_ok && retrieve_ok,
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
            }

            // Final summary
            if !json {
                println!();
                UI::header("Summary");
                println!();
                UI::info(&format!("  Tools found:     {}", tools_found.join(", ")));
                UI::info(&format!(
                    "  compress:       {}",
                    if compress_ok { "PASS" } else { "FAIL" }
                ));
                UI::info(&format!(
                    "  retrieve:       {}",
                    if retrieve_ok { "PASS" } else { "FAIL" }
                ));
                UI::info(&format!(
                    "  stats:          {}",
                    if stats_output.status.success() {
                        "PASS"
                    } else {
                        "FAIL"
                    }
                ));
                UI::info(&format!(
                    "  round-trip:     {}",
                    if compress_ok && retrieve_ok {
                        "PASS"
                    } else {
                        "FAIL"
                    }
                ));
            }

            Ok(())
        }
    }
}
