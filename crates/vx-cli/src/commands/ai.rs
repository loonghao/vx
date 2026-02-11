//! AI tools command - manage AI agent skills and integrations
//!
//! This module implements the `vx ai` command for managing AI agent skills.
//!
//! ## Commands
//!
//! - `vx ai setup` - Auto-install vx skills to AI agent configuration directories
//! - `vx ai skills <args...>` - Proxy to Vercel Skills CLI (npx skills)
//! - `vx ai agents` - List supported AI agents and their config paths

use crate::commands::CommandContext;
use crate::ui::UI;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Supported AI agent targets: (name, project_skills_dir, global_skills_dir)
const SUPPORTED_AGENTS: &[(&str, &str, &str)] = &[
    ("codebuddy", ".codebuddy/skills", ".codebuddy/skills"),
    ("claude-code", ".claude/skills", ".claude/skills"),
    ("cursor", ".cursor/skills", ".cursor/skills"),
    ("codex", ".agents/skills", ".codex/skills"),
    (
        "windsurf",
        ".windsurf/skills",
        ".codeium/windsurf/skills",
    ),
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

    let home_dir =
        dirs::home_dir().context("Could not determine home directory")?;
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
            std::fs::create_dir_all(&skill_dir).with_context(|| {
                format!("Failed to create directory: {}", skill_dir.display())
            })?;

            std::fs::write(&skill_file, VX_USAGE_SKILL).with_context(|| {
                format!("Failed to write SKILL.md to {}", skill_file.display())
            })?;

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
        println!(
            "  {:<16} {:<24} ~/{}",
            name, project_path, global_path
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
                let available: Vec<&str> =
                    SUPPORTED_AGENTS.iter().map(|(n, _, _)| *n).collect();
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
