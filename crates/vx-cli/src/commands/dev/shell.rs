//! Shell spawning for dev command

use crate::commands::setup::ConfigView;
use crate::ui::UI;
use anyhow::Result;
use std::collections::HashMap;
use std::env;
use vx_env::{SessionContext, ShellSpawner};

/// Spawn an interactive dev shell
pub fn spawn_dev_shell(
    shell: Option<String>,
    _env_vars: &HashMap<String, String>,
    config: &ConfigView,
) -> Result<()> {
    // Create SessionContext from config
    let mut session = SessionContext::new(&config.project_name)
        .tools(&config.tools)
        .env_vars(&config.env)
        .env_vars(&config.setenv)
        .isolated(config.isolation)
        .passenv(config.passenv.clone());

    if let Ok(current_dir) = env::current_dir() {
        session = session.project_root(current_dir);
    }

    // Use ShellSpawner for unified shell management
    let spawner = ShellSpawner::new(session)?;

    UI::success("Entering vx development environment");
    UI::info(&format!(
        "Tools: {}",
        config.tools.keys().cloned().collect::<Vec<_>>().join(", ")
    ));
    UI::hint("Type 'exit' to leave the dev environment");
    println!();

    let status = spawner.spawn_interactive(shell.as_deref())?;

    if !status.success() {
        std::process::exit(vx_resolver::exit_code_from_status(&status));
    }

    UI::info("Left vx development environment");
    Ok(())
}
