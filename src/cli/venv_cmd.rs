// Virtual environment CLI commands

use crate::ui::UI;
use crate::venv::VenvManager;
use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct VenvArgs {
    #[command(subcommand)]
    pub command: VenvCommand,
}

#[derive(Subcommand)]
pub enum VenvCommand {
    /// Create a new virtual environment
    Create {
        /// Name of the virtual environment
        name: String,
        /// Tools to install (format: tool@version)
        #[arg(short, long, value_delimiter = ',')]
        tools: Vec<String>,
    },
    /// List all virtual environments
    List,
    /// Activate a virtual environment
    Activate {
        /// Name of the virtual environment
        name: String,
    },
    /// Deactivate current virtual environment
    Deactivate,
    /// Remove a virtual environment
    Remove {
        /// Name of the virtual environment
        name: String,
        /// Force removal without confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Show current virtual environment
    Current,
}

pub async fn handle_venv_command(args: VenvArgs) -> Result<()> {
    let manager = VenvManager::new()?;

    match args.command {
        VenvCommand::Create { name, tools } => create_venv(&manager, &name, &tools).await,
        VenvCommand::List => list_venvs(&manager).await,
        VenvCommand::Activate { name } => activate_venv(&manager, &name).await,
        VenvCommand::Deactivate => deactivate_venv().await,
        VenvCommand::Remove { name, force } => remove_venv(&manager, &name, force).await,
        VenvCommand::Current => show_current_venv().await,
    }
}

async fn create_venv(manager: &VenvManager, name: &str, tools: &[String]) -> Result<()> {
    UI::info(&format!("Creating virtual environment '{}'", name));

    // Parse tool specifications
    let mut tool_specs = Vec::new();
    for tool_spec in tools {
        if let Some((tool, version)) = tool_spec.split_once('@') {
            tool_specs.push((tool.to_string(), version.to_string()));
        } else {
            tool_specs.push((tool_spec.clone(), "latest".to_string()));
        }
    }

    if tool_specs.is_empty() {
        UI::warning("No tools specified. Creating empty virtual environment.");
    } else {
        UI::info("Tools to install:");
        for (tool, version) in &tool_specs {
            println!("  {} @ {}", tool, version);
        }
    }

    manager.create(name, &tool_specs)?;

    UI::success(&format!(
        "Virtual environment '{}' created successfully!",
        name
    ));
    UI::info(&format!("Activate with: vx venv activate {}", name));

    Ok(())
}

async fn list_venvs(manager: &VenvManager) -> Result<()> {
    let venvs = manager.list()?;

    if venvs.is_empty() {
        UI::info("No virtual environments found.");
        UI::hint("Create one with: vx venv create <name>");
        return Ok(());
    }

    UI::header("Virtual Environments");
    let current = VenvManager::current();

    for venv in venvs {
        if Some(&venv) == current.as_ref() {
            println!("* {} (active)", venv);
        } else {
            println!("  {}", venv);
        }
    }

    if let Some(current) = current {
        UI::info(&format!("Currently active: {}", current));
    }

    Ok(())
}

async fn activate_venv(manager: &VenvManager, name: &str) -> Result<()> {
    if VenvManager::is_active() {
        if let Some(current) = VenvManager::current() {
            if current == name {
                UI::warning(&format!("Virtual environment '{}' is already active", name));
                return Ok(());
            } else {
                UI::warning(&format!("Deactivating current environment '{}'", current));
            }
        }
    }

    let activation_script = manager.activate(name)?;

    UI::success(&format!("Activating virtual environment '{}'", name));
    UI::info("Run the following commands in your shell:");
    println!();
    println!("{}", activation_script);
    println!();
    UI::hint(&format!(
        "Copy and paste the above commands, or use: eval \"$(vx venv activate {})\"",
        name
    ));

    Ok(())
}

async fn deactivate_venv() -> Result<()> {
    if !VenvManager::is_active() {
        UI::warning("No virtual environment is currently active");
        return Ok(());
    }

    let current = VenvManager::current().unwrap();
    let deactivation_script = VenvManager::deactivate();

    UI::info(&format!("Deactivating virtual environment '{}'", current));
    println!();
    println!("{}", deactivation_script);
    println!();
    UI::hint("Copy and paste the above commands, or use: eval \"$(vx venv deactivate)\"");

    Ok(())
}

async fn remove_venv(manager: &VenvManager, name: &str, force: bool) -> Result<()> {
    if !force {
        use dialoguer::Confirm;

        let confirm = Confirm::new()
            .with_prompt(&format!(
                "Are you sure you want to remove virtual environment '{}'?",
                name
            ))
            .default(false)
            .interact()?;

        if !confirm {
            UI::info("Operation cancelled");
            return Ok(());
        }
    }

    // Check if trying to remove active environment
    if let Some(current) = VenvManager::current() {
        if current == name {
            UI::warning("Cannot remove active virtual environment. Deactivate first.");
            return Ok(());
        }
    }

    manager.remove(name)?;
    UI::success(&format!("Virtual environment '{}' removed", name));

    Ok(())
}

async fn show_current_venv() -> Result<()> {
    if let Some(current) = VenvManager::current() {
        UI::info(&format!("Current virtual environment: {}", current));
    } else {
        UI::info("No virtual environment is currently active");
        UI::hint("Activate one with: vx venv activate <name>");
    }

    Ok(())
}
