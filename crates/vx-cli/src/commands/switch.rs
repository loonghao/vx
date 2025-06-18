//! Switch command implementation

use crate::ui::UI;
use vx_core::{PluginRegistry, Result, VxEnvironment, VxError, VxShimexeManager};

pub async fn handle(_registry: &PluginRegistry, tool_version: &str, global: bool) -> Result<()> {
    // Parse tool@version format
    let (tool_name, version) = parse_tool_version(tool_version)?;

    UI::info(&format!("Switching {} to version {}", tool_name, version));

    // Create environment and shim manager
    let environment = VxEnvironment::new()?;
    let shim_manager = VxShimexeManager::new(environment.clone())?;

    // Check if the version is installed
    if !environment.is_version_installed(&tool_name, &version) {
        UI::error(&format!(
            "Version {} of {} is not installed. Install it first with: vx install {}@{}",
            version, tool_name, tool_name, version
        ));
        return Err(VxError::VersionNotInstalled {
            tool_name: tool_name.clone(),
            version: version.clone(),
        });
    }

    // Get the installation info to find the executable path
    let installation = environment
        .get_installation_info(&tool_name, &version)?
        .ok_or_else(|| VxError::VersionNotInstalled {
            tool_name: tool_name.clone(),
            version: version.clone(),
        })?;

    // Switch the tool version using shim
    let _shim_path =
        shim_manager.switch_tool_version(&tool_name, &version, &installation.executable_path)?;

    if global {
        // Set as global default version
        environment.set_active_version(&tool_name, &version)?;
        UI::success(&format!(
            "Switched {} to version {} globally",
            tool_name, version
        ));
        UI::hint(&format!(
            "All new terminal sessions will use {}@{}",
            tool_name, version
        ));
    } else {
        // Session-level switch (for now, just update the shim)
        UI::success(&format!(
            "Switched {} to version {} in current session",
            tool_name, version
        ));
        UI::hint(&format!(
            "Use 'vx switch {}@{} --global' to make this the default for all sessions",
            tool_name, version
        ));
    }

    // Verify the switch worked
    let shim_dir = environment.shim_dir()?;
    let shim_path = shim_dir.join(if cfg!(windows) {
        format!("{}.exe", tool_name)
    } else {
        tool_name.clone()
    });

    if shim_path.exists() {
        UI::info(&format!("Shim created at: {}", shim_path.display()));
        UI::hint(&format!(
            "Make sure {} is in your PATH to use the switched version",
            shim_dir.display()
        ));
    }

    Ok(())
}

/// Parse tool@version format
pub fn parse_tool_version(tool_version: &str) -> Result<(String, String)> {
    if let Some((tool, version)) = tool_version.split_once('@') {
        if tool.is_empty() || version.is_empty() {
            return Err(VxError::ParseError {
                message: format!("Invalid tool@version format: {}", tool_version),
            });
        }
        Ok((tool.to_string(), version.to_string()))
    } else {
        Err(VxError::ParseError {
            message: format!(
                "Invalid format: {}. Expected format: tool@version (e.g., node@20.10.0)",
                tool_version
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_version() {
        // Valid cases
        assert_eq!(
            parse_tool_version("node@20.10.0").unwrap(),
            ("node".to_string(), "20.10.0".to_string())
        );
        assert_eq!(
            parse_tool_version("python@3.11.0").unwrap(),
            ("python".to_string(), "3.11.0".to_string())
        );

        // Invalid cases
        assert!(parse_tool_version("node").is_err());
        assert!(parse_tool_version("@20.10.0").is_err());
        assert!(parse_tool_version("node@").is_err());
        assert!(parse_tool_version("").is_err());
    }
}
