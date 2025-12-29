//! Analyze command implementation
//!
//! Analyzes project dependencies, scripts, and tools to provide
//! insights and suggestions for vx.toml configuration.

use anyhow::{Context, Result};
use colored::Colorize;
use std::env;
use std::path::Path;
use vx_project_analyzer::{AnalyzerConfig, ProjectAnalyzer, SyncAction};

use crate::ui::UI;

/// Handle the analyze command
pub async fn handle(json: bool, verbose: bool) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    analyze_project(&current_dir, json, verbose).await
}

/// Analyze a project and display results
pub async fn analyze_project(root: &Path, json: bool, verbose: bool) -> Result<()> {
    let config = AnalyzerConfig {
        check_installed: true,
        check_tools: true,
        generate_sync_actions: true,
        max_depth: 3,
    };

    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(root).await?;

    if json {
        // Output as JSON
        let json_output = serde_json::to_string_pretty(&analysis)?;
        println!("{}", json_output);
        return Ok(());
    }

    // Pretty print results
    println!("{}", "📊 Project Analysis".bold());
    println!();

    // Ecosystems
    if !analysis.ecosystems.is_empty() {
        let ecosystems: Vec<_> = analysis
            .ecosystems
            .iter()
            .map(|e| e.display_name())
            .collect();
        println!("{}: {}", "Ecosystems".bold(), ecosystems.join(", "));
        println!();
    }

    // Dependencies
    if !analysis.dependencies.is_empty() {
        println!("{}", "📦 Dependencies:".bold());
        let mut by_ecosystem: std::collections::HashMap<_, Vec<_>> =
            std::collections::HashMap::new();
        for dep in &analysis.dependencies {
            by_ecosystem.entry(dep.ecosystem).or_default().push(dep);
        }

        for (ecosystem, deps) in by_ecosystem {
            println!("  {} ({}):", ecosystem.display_name(), deps.len());
            for dep in deps {
                let status = if dep.is_installed {
                    "✅".green()
                } else {
                    "⚠️ ".yellow()
                };
                let version = dep
                    .version
                    .as_ref()
                    .map(|v| format!(" = \"{}\"", v))
                    .unwrap_or_default();
                let dev_marker = if dep.is_dev { " (dev)" } else { "" };
                let installed_marker = if !dep.is_installed {
                    " - not installed"
                } else {
                    ""
                };
                println!(
                    "    {} {}{}{}{}",
                    status, dep.name, version, dev_marker, installed_marker
                );
            }
        }
        println!();
    }

    // Scripts
    if !analysis.scripts.is_empty() {
        println!("{}", "📜 Scripts:".bold());
        for script in &analysis.scripts {
            println!("  {}: {}", script.name.cyan(), script.command);
            if verbose && !script.tools.is_empty() {
                for tool in &script.tools {
                    let status = if tool.is_available {
                        "✅".green()
                    } else {
                        "⚠️ ".yellow()
                    };
                    println!(
                        "    └─ requires: {} ({}) {}",
                        tool.name, tool.invocation, status
                    );
                }
            }
        }
        println!();
    }

    // Required tools
    if !analysis.required_tools.is_empty() {
        println!("{}", "🔧 Required Tools:".bold());
        for tool in &analysis.required_tools {
            let status = if tool.is_available {
                "✅".green()
            } else {
                "⚠️ ".yellow()
            };
            let version = tool
                .version
                .as_ref()
                .map(|v| format!(" = \"{}\"", v))
                .unwrap_or_else(|| " = \"latest\"".to_string());
            let missing_marker = if !tool.is_available {
                format!(" - missing ({})", tool.install_method)
            } else {
                String::new()
            };
            println!("  {} {}{}{}", status, tool.name, version, missing_marker);
        }
        println!();
    }

    // Sync suggestions
    if !analysis.sync_actions.is_empty() {
        println!("{}", "💡 Suggestions:".bold());
        for (i, action) in analysis.sync_actions.iter().enumerate() {
            let suggestion = match action {
                SyncAction::AddTool { name, version } => {
                    format!("Add [tools] {} = \"{}\" to vx.toml", name, version)
                }
                SyncAction::UpdateTool {
                    name,
                    old_version,
                    new_version,
                } => {
                    format!(
                        "Update [tools] {} \"{}\" → \"{}\"",
                        name, old_version, new_version
                    )
                }
                SyncAction::AddScript { name, command } => {
                    format!("Add [scripts] {} = \"{}\" to vx.toml", name, command)
                }
                SyncAction::UpdateScript {
                    name,
                    old_command: _,
                    new_command,
                } => {
                    format!("Update [scripts] {} = \"{}\"", name, new_command)
                }
                SyncAction::InstallDependency {
                    command,
                    description,
                } => {
                    format!("Run: {} ({})", command, description)
                }
                SyncAction::AddProjectDependency {
                    file,
                    section,
                    content,
                } => {
                    format!("Add to {} [{}]: {}", file.display(), section, content)
                }
            };
            println!("  {}. {}", i + 1, suggestion);
        }
        println!();
    }

    // Summary
    let missing_deps = analysis.missing_dependencies().len();
    let missing_tools = analysis.missing_tools().len();

    if missing_deps > 0 || missing_tools > 0 {
        UI::warn(&format!(
            "{} missing dependencies, {} missing tools",
            missing_deps, missing_tools
        ));
        UI::hint("Run 'vx sync' to synchronize configuration");
    } else {
        UI::success("All dependencies and tools are available");
    }

    Ok(())
}

/// Analyze and apply sync actions
pub async fn handle_sync_from_analysis(
    root: &Path,
    dry_run: bool,
    _force: bool,
    verbose: bool,
) -> Result<()> {
    let config = AnalyzerConfig {
        check_installed: true,
        check_tools: true,
        generate_sync_actions: true,
        max_depth: 3,
    };

    let analyzer = ProjectAnalyzer::new(config);
    let analysis = analyzer.analyze(root).await?;

    if analysis.sync_actions.is_empty() {
        UI::success("Project is already in sync");
        return Ok(());
    }

    UI::info(&format!(
        "Found {} sync action(s)",
        analysis.sync_actions.len()
    ));

    if verbose {
        for action in &analysis.sync_actions {
            println!("  - {}", action);
        }
    }

    let sync_manager = analyzer.sync_manager();
    let result = sync_manager
        .apply_actions(root, &analysis.sync_actions, dry_run)
        .await?;

    if dry_run {
        UI::info("Dry run completed - no changes made");
        return Ok(());
    }

    if !result.applied.is_empty() {
        UI::success(&format!("Applied {} action(s)", result.applied.len()));
    }

    if !result.install_commands.is_empty() {
        UI::info("Run the following commands to install dependencies:");
        for cmd in &result.install_commands {
            println!("  {}", cmd.cyan());
        }
    }

    Ok(())
}
