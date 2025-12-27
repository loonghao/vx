//! Team collaboration commands
//!
//! Provides team-related functionality:
//! - CODEOWNERS generation
//! - Convention validation (commit messages, branch names)
//! - Review rules

use anyhow::Result;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::ui::UI;

/// Handle CODEOWNERS generation
pub async fn handle_codeowners(output: Option<String>, dry_run: bool, verbose: bool) -> Result<()> {
    let current_dir = env::current_dir()?;
    let config_path = current_dir.join(".vx.toml");

    UI::header("Generate CODEOWNERS");

    if !config_path.exists() {
        return Err(anyhow::anyhow!("No .vx.toml found. Run 'vx init' first."));
    }

    let config = vx_config::parse_config(&config_path)?;

    let team_config = config.team.as_ref().ok_or_else(|| {
        anyhow::anyhow!("No [team] section in .vx.toml. Add team configuration first.")
    })?;

    // Generate CODEOWNERS content
    let content = vx_config::generate_codeowners(team_config);

    if content.is_empty() {
        UI::warn("No code owners defined in configuration");
        UI::hint("Add [team.code_owners] section to .vx.toml");
        return Ok(());
    }

    if dry_run {
        UI::info("Preview of CODEOWNERS file:");
        println!("\n{}", content);
        return Ok(());
    }

    // Determine output path
    let output_path = if let Some(path) = output {
        Path::new(&path).to_path_buf()
    } else {
        // Check for .github or .gitlab directory
        let github_path = current_dir.join(".github").join("CODEOWNERS");
        let gitlab_path = current_dir.join(".gitlab").join("CODEOWNERS");
        let root_path = current_dir.join("CODEOWNERS");

        if current_dir.join(".github").exists() {
            github_path
        } else if current_dir.join(".gitlab").exists() {
            gitlab_path
        } else {
            root_path
        }
    };

    // Create parent directory if needed
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    fs::write(&output_path, &content)?;

    if verbose {
        println!("{}", content);
    }

    UI::success(&format!(
        "Generated CODEOWNERS at {}",
        output_path.display()
    ));

    Ok(())
}

/// Handle convention validation
pub async fn handle_validate(commit: bool, branch: bool, all: bool, verbose: bool) -> Result<()> {
    let current_dir = env::current_dir()?;
    let config_path = current_dir.join(".vx.toml");

    UI::header("Validate Conventions");

    if !config_path.exists() {
        return Err(anyhow::anyhow!("No .vx.toml found. Run 'vx init' first."));
    }

    let config = vx_config::parse_config(&config_path)?;

    let team_config = config.team.as_ref().ok_or_else(|| {
        anyhow::anyhow!("No [team] section in .vx.toml. Add team configuration first.")
    })?;

    let mut has_errors = false;

    // Validate commit message
    if commit || all {
        if let Err(e) = validate_commit_message(team_config, verbose).await {
            UI::error(&format!("Commit message validation failed: {}", e));
            has_errors = true;
        }
    }

    // Validate branch name
    if branch || all {
        if let Err(e) = validate_branch_name(team_config, verbose).await {
            UI::error(&format!("Branch name validation failed: {}", e));
            has_errors = true;
        }
    }

    if has_errors {
        return Err(anyhow::anyhow!("Validation failed"));
    }

    UI::success("All conventions validated");
    Ok(())
}

/// Handle review rules display
pub async fn handle_review_rules(verbose: bool) -> Result<()> {
    let current_dir = env::current_dir()?;
    let config_path = current_dir.join(".vx.toml");

    UI::header("Review Rules");

    if !config_path.exists() {
        return Err(anyhow::anyhow!("No .vx.toml found. Run 'vx init' first."));
    }

    let config = vx_config::parse_config(&config_path)?;

    let team_config = config.team.as_ref().ok_or_else(|| {
        anyhow::anyhow!("No [team] section in .vx.toml. Add team configuration first.")
    })?;

    // Display review rules
    if let Some(review) = &team_config.review {
        if let Some(approvals) = review.required_approvals {
            println!("Required approvals: {}", approvals);
        }

        if let Some(require_code_owner) = review.require_code_owner {
            println!(
                "Require CODEOWNER review: {}",
                if require_code_owner { "Yes" } else { "No" }
            );
        }

        if let Some(dismiss_stale) = review.dismiss_stale {
            println!(
                "Dismiss stale reviews: {}",
                if dismiss_stale { "Yes" } else { "No" }
            );
        }

        if !review.protected_branches.is_empty() {
            println!(
                "Protected branches: {}",
                review.protected_branches.join(", ")
            );
        }

        if verbose {
            if let Some(auto_assign) = &review.auto_assign {
                if !auto_assign.reviewers.is_empty() {
                    println!(
                        "Auto-assign reviewers: {}",
                        auto_assign.reviewers.join(", ")
                    );
                }
            }
        }
    } else {
        UI::warn("No review rules configured");
        UI::hint("Add [team.review] section to .vx.toml");
    }

    Ok(())
}

/// Handle team status display
pub async fn handle_status(verbose: bool) -> Result<()> {
    let current_dir = env::current_dir()?;
    let config_path = current_dir.join(".vx.toml");

    UI::header("Team Configuration Status");

    if !config_path.exists() {
        UI::warn("No .vx.toml found");
        return Ok(());
    }

    let config = vx_config::parse_config(&config_path)?;

    let team_config = match &config.team {
        Some(tc) => tc,
        None => {
            UI::warn("No [team] section in .vx.toml");
            return Ok(());
        }
    };

    // Code owners
    println!("\n{}", UI::format_header("Code Owners"));
    if let Some(code_owners) = &team_config.code_owners {
        if code_owners.paths.is_empty() && code_owners.default_owners.is_empty() {
            println!("  No code owners defined");
        } else {
            if !code_owners.default_owners.is_empty() {
                println!("  * -> {}", code_owners.default_owners.join(", "));
            }
            for (path, owners) in &code_owners.paths {
                println!("  {} -> {}", path, owners.join(", "));
            }
        }
    } else {
        println!("  No code owners defined");
    }

    // Conventions
    println!("\n{}", UI::format_header("Conventions"));
    if let Some(conv) = &team_config.conventions {
        if let Some(format) = &conv.commit_format {
            println!("  Commit format: {}", format);
        }
        if let Some(pattern) = &conv.branch_pattern {
            println!("  Branch pattern: {}", pattern);
        }
        if verbose && !conv.merge_strategies.is_empty() {
            println!("  Merge strategies: {}", conv.merge_strategies.join(", "));
        }
    } else {
        println!("  No conventions defined");
    }

    // Review rules
    println!("\n{}", UI::format_header("Review Rules"));
    if let Some(review) = &team_config.review {
        if let Some(approvals) = review.required_approvals {
            println!("  Required approvals: {}", approvals);
        }
        println!(
            "  Require CODEOWNER review: {}",
            if review.require_code_owner.unwrap_or(false) {
                "Yes"
            } else {
                "No"
            }
        );
    } else {
        println!("  No review rules defined");
    }

    // Check for existing CODEOWNERS file
    println!("\n{}", UI::format_header("Files"));
    let codeowners_paths = [
        current_dir.join("CODEOWNERS"),
        current_dir.join(".github").join("CODEOWNERS"),
        current_dir.join(".gitlab").join("CODEOWNERS"),
        current_dir.join("docs").join("CODEOWNERS"),
    ];

    let mut found_codeowners = false;
    for path in &codeowners_paths {
        if path.exists() {
            println!("  CODEOWNERS: {}", path.display());
            found_codeowners = true;
            break;
        }
    }
    if !found_codeowners {
        println!("  CODEOWNERS: Not found");
        UI::hint("Run 'vx team codeowners' to generate");
    }

    Ok(())
}

// Helper functions

async fn validate_commit_message(team_config: &vx_config::TeamConfig, verbose: bool) -> Result<()> {
    let conventions = team_config
        .conventions
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No conventions defined in team configuration"))?;

    // Get the latest commit message
    let output = Command::new("git")
        .args(["log", "-1", "--format=%s"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to get commit message"));
    }

    let commit_msg = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if verbose {
        UI::info(&format!("Checking commit: {}", commit_msg));
    }

    // Check commit format
    if let Some(format) = &conventions.commit_format {
        match format.as_str() {
            "conventional" => {
                // Conventional commits: type(scope): description
                let re = regex::Regex::new(
                    r"^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(.+\))?: .+",
                )?;

                if !re.is_match(&commit_msg) {
                    return Err(anyhow::anyhow!(
                        "Commit message does not follow conventional commits format.\n\
                         Expected: type(scope): description\n\
                         Got: {}",
                        commit_msg
                    ));
                }
            }
            "angular" => {
                // Angular format
                let re = regex::Regex::new(
                    r"^(build|ci|docs|feat|fix|perf|refactor|style|test)(\(.+\))?: .+",
                )?;

                if !re.is_match(&commit_msg) {
                    return Err(anyhow::anyhow!(
                        "Commit message does not follow Angular format"
                    ));
                }
            }
            _ => {
                // Custom format - treat as regex
                let re = regex::Regex::new(format)?;
                if !re.is_match(&commit_msg) {
                    return Err(anyhow::anyhow!(
                        "Commit message does not match pattern: {}",
                        format
                    ));
                }
            }
        }
    }

    // Check custom commit pattern if specified
    if let Some(pattern) = &conventions.commit_pattern {
        let re = regex::Regex::new(pattern)?;
        if !re.is_match(&commit_msg) {
            return Err(anyhow::anyhow!(
                "Commit message does not match pattern: {}",
                pattern
            ));
        }
    }

    UI::success("Commit message is valid");
    Ok(())
}

async fn validate_branch_name(team_config: &vx_config::TeamConfig, verbose: bool) -> Result<()> {
    let conventions = team_config
        .conventions
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No conventions defined in team configuration"))?;

    // Get current branch name
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to get branch name"));
    }

    let branch_name = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if verbose {
        UI::info(&format!("Checking branch: {}", branch_name));
    }

    // Check branch pattern
    if let Some(pattern) = &conventions.branch_pattern {
        let re = regex::Regex::new(pattern)?;

        if !re.is_match(&branch_name) {
            return Err(anyhow::anyhow!(
                "Branch name '{}' does not match pattern: {}",
                branch_name,
                pattern
            ));
        }
    }

    // Check protected branches from review config
    if let Some(review) = &team_config.review {
        if !review.protected_branches.is_empty()
            && review.protected_branches.contains(&branch_name)
        {
            UI::warn(&format!(
                "Branch '{}' is protected. Consider creating a feature branch.",
                branch_name
            ));
        }
    }

    UI::success("Branch name is valid");
    Ok(())
}
