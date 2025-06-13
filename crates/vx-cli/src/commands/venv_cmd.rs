// Virtual environment CLI commands

use crate::ui::UI;
use clap::{Args, Subcommand};
use vx_core::{Result, VenvManager};

#[derive(Args)]
pub struct VenvArgs {
    #[command(subcommand)]
    pub command: VenvCommand,
}

#[derive(Subcommand, Clone)]
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

pub async fn handle(command: VenvCommand) -> Result<()> {
    let manager = VenvManager::new()?;

    match command {
        VenvCommand::Create { name, tools } => create_venv(&manager, &name, &tools).await,
        VenvCommand::List => list_venvs(&manager).await,
        VenvCommand::Activate { name } => activate_venv(&manager, &name).await,
        VenvCommand::Deactivate => deactivate_venv().await,
        VenvCommand::Remove { name, force } => remove_venv(&manager, &name, force).await,
        VenvCommand::Current => show_current_venv().await,
    }
}

async fn create_venv(manager: &VenvManager, name: &str, tools: &[String]) -> Result<()> {
    UI::info(&format!("Creating virtual environment '{name}'"));

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
            UI::detail(&format!("  {} @ {}", tool, version));
        }
    }

    // Create the virtual environment using VenvManager
    match manager.create(name, &tool_specs) {
        Ok(()) => {
            UI::success(&format!(
                "Virtual environment '{}' created successfully!",
                name
            ));
            UI::hint(&format!("Activate with: vx venv activate {}", name));
        }
        Err(e) => {
            UI::error(&format!(
                "Failed to create virtual environment '{}': {}",
                name, e
            ));
            return Err(e);
        }
    }

    Ok(())
}

async fn list_venvs(manager: &VenvManager) -> Result<()> {
    UI::header("Virtual Environments");

    let venvs = match manager.list() {
        Ok(venvs) => venvs,
        Err(e) => {
            UI::error(&format!("Failed to list virtual environments: {}", e));
            return Err(e);
        }
    };

    if venvs.is_empty() {
        UI::info("No virtual environments found.");
        UI::hint("Create one with: vx venv create <name>");
        return Ok(());
    }

    let current = VenvManager::current();

    for venv in venvs {
        if Some(&venv) == current.as_ref() {
            UI::success(&format!("* {} (active)", venv));
        } else {
            UI::info(&format!("  {}", venv));
        }
    }

    if let Some(current) = current {
        UI::detail(&format!("Currently active: {}", current));
    } else {
        UI::hint("Activate an environment with: vx venv activate <name>");
    }

    Ok(())
}

async fn activate_venv(manager: &VenvManager, name: &str) -> Result<()> {
    UI::info(&format!("Activating virtual environment '{}'", name));

    // Check if already active
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

    // Generate activation script
    let activation_script = match manager.activate(name) {
        Ok(script) => script,
        Err(e) => {
            UI::error(&format!(
                "Failed to activate virtual environment '{}': {}",
                name, e
            ));
            return Err(e);
        }
    };

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
    UI::info("Deactivating virtual environment");

    if !VenvManager::is_active() {
        UI::warning("No virtual environment is currently active");
        return Ok(());
    }

    let current = VenvManager::current().unwrap();
    let deactivation_script = VenvManager::deactivate();

    UI::success(&format!("Deactivating virtual environment '{}'", current));
    UI::info("Run the following commands in your shell:");
    println!();
    println!("{}", deactivation_script);
    println!();
    UI::hint("Copy and paste the above commands, or use: eval \"$(vx venv deactivate)\"");

    Ok(())
}

async fn remove_venv(manager: &VenvManager, name: &str, force: bool) -> Result<()> {
    UI::info(&format!("Removing virtual environment '{}'", name));

    // Check if trying to remove active environment
    if let Some(current) = VenvManager::current() {
        if current == name {
            UI::error("Cannot remove active virtual environment. Deactivate first.");
            UI::hint("Run: vx venv deactivate");
            return Ok(());
        }
    }

    if !force {
        UI::warning(&format!(
            "This will permanently delete virtual environment '{}'",
            name
        ));
        UI::info("Use --force to confirm removal");
        return Ok(());
    }

    // Remove the virtual environment
    match manager.remove(name) {
        Ok(()) => {
            UI::success(&format!(
                "Virtual environment '{}' removed successfully",
                name
            ));
        }
        Err(e) => {
            UI::error(&format!(
                "Failed to remove virtual environment '{}': {}",
                name, e
            ));
            return Err(e);
        }
    }

    Ok(())
}

async fn show_current_venv() -> Result<()> {
    UI::header("Current Virtual Environment");

    if let Some(current) = VenvManager::current() {
        UI::success(&format!("Current virtual environment: {}", current));
        UI::detail("Environment is active");
    } else {
        UI::info("No virtual environment is currently active");
        UI::hint("Activate one with: vx venv activate <name>");
    }

    Ok(())
}
