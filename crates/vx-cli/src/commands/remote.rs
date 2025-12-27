//! Remote development environment commands
//!
//! Provides remote development configuration:
//! - Codespaces configuration generation
//! - GitPod configuration generation
//! - devcontainer.json generation

use anyhow::Result;
use std::env;
use std::fs;
use std::path::Path;

use crate::ui::UI;

/// Handle remote configuration generation
pub async fn handle_generate(
    target: Option<String>,
    output: Option<String>,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    let current_dir = env::current_dir()?;
    let config_path = current_dir.join(".vx.toml");

    UI::header("Generate Remote Development Configuration");

    if !config_path.exists() {
        return Err(anyhow::anyhow!("No .vx.toml found. Run 'vx init' first."));
    }

    let config = vx_config::parse_config(&config_path)?;

    let remote_config = config.remote.clone().unwrap_or_default();

    // Determine which configurations to generate
    let targets: Vec<&str> = match target.as_deref() {
        Some("codespaces") | Some("devcontainer") => vec!["devcontainer"],
        Some("gitpod") => vec!["gitpod"],
        Some("all") | None => vec!["devcontainer", "gitpod"],
        Some(t) => {
            return Err(anyhow::anyhow!(
                "Unknown target: {}. Use: codespaces, gitpod, all",
                t
            ))
        }
    };

    for t in targets {
        match t {
            "devcontainer" => {
                generate_devcontainer(
                    &current_dir,
                    &config,
                    &remote_config,
                    output.clone(),
                    dry_run,
                    verbose,
                )
                .await?;
            }
            "gitpod" => {
                generate_gitpod(
                    &current_dir,
                    &config,
                    &remote_config,
                    output.clone(),
                    dry_run,
                    verbose,
                )
                .await?;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Handle remote status display
pub async fn handle_status(verbose: bool) -> Result<()> {
    let current_dir = env::current_dir()?;

    UI::header("Remote Development Status");

    // Check for existing configurations
    let devcontainer_path = current_dir.join(".devcontainer").join("devcontainer.json");
    let gitpod_path = current_dir.join(".gitpod.yml");

    println!("\n{}", UI::format_header("Configuration Files"));

    // Devcontainer
    if devcontainer_path.exists() {
        println!("  {} devcontainer.json", UI::format_success("✓"));
        if verbose {
            let content = fs::read_to_string(&devcontainer_path)?;
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(name) = json["name"].as_str() {
                    println!("    Name: {}", name);
                }
                if let Some(image) = json["image"].as_str() {
                    println!("    Image: {}", image);
                }
            }
        }
    } else {
        println!("  {} devcontainer.json", UI::format_warn("✗"));
    }

    // GitPod
    if gitpod_path.exists() {
        println!("  {} .gitpod.yml", UI::format_success("✓"));
        if verbose {
            let content = fs::read_to_string(&gitpod_path)?;
            if let Ok(yaml) = serde_yaml::from_str::<serde_json::Value>(&content) {
                if let Some(image) = yaml["image"].as_str() {
                    println!("    Image: {}", image);
                }
            }
        }
    } else {
        println!("  {} .gitpod.yml", UI::format_warn("✗"));
    }

    // Check .vx.toml remote config
    let config_path = current_dir.join(".vx.toml");
    if config_path.exists() {
        let config = vx_config::parse_config(&config_path)?;
        if let Some(remote) = &config.remote {
            println!("\n{}", UI::format_header("Configuration in .vx.toml"));

            if let Some(cs) = &remote.codespaces {
                println!("  Codespaces:");
                if let Some(machine) = &cs.machine {
                    println!("    Machine: {}", machine);
                }
            }

            if let Some(gp) = &remote.gitpod {
                println!("  GitPod:");
                if let Some(image) = &gp.image {
                    println!("    Image: {}", image);
                }
            }
        }
    }

    UI::hint("Run 'vx remote generate' to create/update configuration files");

    Ok(())
}

// Helper functions

async fn generate_devcontainer(
    dir: &Path,
    config: &vx_config::VxConfig,
    remote_config: &vx_config::RemoteConfig,
    output: Option<String>,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    UI::step("Generating devcontainer.json...");

    let content = vx_config::generate_devcontainer_json(config, remote_config);

    if dry_run {
        UI::info("Preview of devcontainer.json:");
        println!("\n{}", content);
        return Ok(());
    }

    let output_path = if let Some(path) = output {
        Path::new(&path).to_path_buf()
    } else {
        let devcontainer_dir = dir.join(".devcontainer");
        if !devcontainer_dir.exists() {
            fs::create_dir_all(&devcontainer_dir)?;
        }
        devcontainer_dir.join("devcontainer.json")
    };

    fs::write(&output_path, &content)?;

    if verbose {
        println!("{}", content);
    }

    UI::success(&format!("Generated {}", output_path.display()));

    // Also generate Dockerfile if container config exists
    if config.container.is_some() {
        let dockerfile_path = output_path.parent().unwrap().join("Dockerfile");
        if !dockerfile_path.exists() {
            let dockerfile_content = vx_config::generate_dockerfile(config);
            fs::write(&dockerfile_path, &dockerfile_content)?;
            UI::success(&format!("Generated {}", dockerfile_path.display()));
        }
    }

    Ok(())
}

async fn generate_gitpod(
    dir: &Path,
    config: &vx_config::VxConfig,
    remote_config: &vx_config::RemoteConfig,
    output: Option<String>,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    UI::step("Generating .gitpod.yml...");

    let content = vx_config::generate_gitpod_yml(config, remote_config);

    if dry_run {
        UI::info("Preview of .gitpod.yml:");
        println!("\n{}", content);
        return Ok(());
    }

    let output_path = if let Some(path) = output {
        Path::new(&path).to_path_buf()
    } else {
        dir.join(".gitpod.yml")
    };

    fs::write(&output_path, &content)?;

    if verbose {
        println!("{}", content);
    }

    UI::success(&format!("Generated {}", output_path.display()));

    Ok(())
}
