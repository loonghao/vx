//! Shell spawning for dev command

use crate::commands::setup::ConfigView;
use crate::ui::UI;
use anyhow::Result;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use vx_env::{SessionContext, SessionSource, ShellSpawner};

/// Spawn an interactive dev shell
pub fn spawn_dev_shell(
    shell: Option<String>,
    env_vars: &HashMap<String, String>,
    config: &ConfigView,
    config_path: Option<PathBuf>,
) -> Result<()> {
    // Create SessionContext from config
    let mut session = SessionContext::new(&config.project_name)
        .tools(&config.tools)
        .env_vars(env_vars)  // Use the complete environment from build_dev_environment
        .isolated(config.isolation)
        .passenv(config.passenv.clone());

    // Set source based on config_path
    if let Some(path) = config_path {
        session = session.source(SessionSource::VxToml {
            path,
            project_name: config.project_name.clone(),
        });
    } else {
        session = session.source(SessionSource::Inline);
    }

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

    let status = spawner.spawn_interactive(shell.as_deref())?;

    if !status.success() {
        std::process::exit(vx_resolver::exit_code_from_status(&status));
    }

    UI::info("Left vx development environment");
    Ok(())
}
