// Remove command implementation

use crate::ui::UI;
use anyhow::Result;

pub async fn handle(tool: String, version: Option<String>, force: bool) -> Result<()> {
    let mut executor = crate::executor::Executor::new()?;

    if !force {
        let confirmation_message = if let Some(version) = &version {
            format!("Remove {tool} version {version}?")
        } else {
            format!("Remove all versions of {tool}?")
        };

        if !UI::confirm(&confirmation_message, false)? {
            UI::info("Operation cancelled");
            return Ok(());
        }
    }

    let spinner = if let Some(version) = &version {
        UI::new_spinner(&format!("Removing {tool} version {version}..."))
    } else {
        UI::new_spinner(&format!("Removing all versions of {tool}..."))
    };

    match version {
        Some(version) => {
            executor.remove_version(&tool, &version)?;
            spinner.finish_and_clear();
            UI::success(&format!("Removed {tool} version {version}"));
        }
        None => {
            executor.remove_tool(&tool)?;
            spinner.finish_and_clear();
            UI::success(&format!("Removed all versions of {tool}"));
        }
    }

    Ok(())
}
