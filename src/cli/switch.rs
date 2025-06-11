// Switch command implementation

use crate::ui::UI;
use anyhow::Result;

pub async fn handle(tool_version: String, global: bool) -> Result<()> {
    let parts: Vec<&str> = tool_version.split('@').collect();
    if parts.len() != 2 {
        UI::error("Invalid format. Use: tool@version (e.g., go@1.21.6)");
        std::process::exit(1);
    }

    let tool_name = parts[0];
    let version = parts[1];

    let spinner = UI::new_spinner(&format!("Switching {tool_name} to version {version}..."));

    let mut executor = crate::executor::Executor::new()?;
    match executor.switch_version(tool_name, version) {
        Ok(()) => {
            spinner.finish_and_clear();
            if global {
                UI::success(&format!(
                    "Globally switched {tool_name} to version {version}"
                ));
            } else {
                UI::success(&format!("Switched {tool_name} to version {version}"));
            }
        }
        Err(e) => {
            spinner.finish_and_clear();
            UI::error(&format!("Failed to switch version: {e}"));
            std::process::exit(1);
        }
    }

    Ok(())
}
