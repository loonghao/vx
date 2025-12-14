//! Switch command implementation

use crate::ui::UI;
use anyhow::Result;
use vx_plugin::BundleRegistry;

pub async fn handle(_registry: &BundleRegistry, tool_version: &str, global: bool) -> Result<()> {
    // Parse tool@version format
    let (tool_name, version) = parse_tool_version(tool_version)?;

    UI::info(&format!("Switching {} to version {}", tool_name, version));

    // Simplified implementation - switch command not fully implemented yet
    UI::warning("Switch command not yet fully implemented in new architecture");
    UI::hint(&format!(
        "Would switch {} to version {} (global: {})",
        tool_name, version, global
    ));

    Ok(())
}

/// Parse tool@version format
pub fn parse_tool_version(tool_version: &str) -> Result<(String, String)> {
    if let Some((tool, version)) = tool_version.split_once('@') {
        if tool.is_empty() || version.is_empty() {
            return Err(anyhow::anyhow!(
                "Invalid tool@version format: {}",
                tool_version
            ));
        }
        Ok((tool.to_string(), version.to_string()))
    } else {
        Err(anyhow::anyhow!(
            "Invalid format: {}. Expected format: tool@version (e.g., node@20.10.0)",
            tool_version
        ))
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
