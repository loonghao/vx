// Install command implementation

use crate::ui::UI;
use crate::with_progress_events;
use anyhow::Result;
use tracing::instrument;

#[instrument(name = "install_command", fields(tool = %tool, version = ?version, force = force))]
pub async fn handle(tool: String, version: Option<String>, force: bool) -> Result<()> {
    let version = version.unwrap_or_else(|| "latest".to_string());

    // Check if already installed (only for system tools when not forcing)
    if !force && which::which(&tool).is_ok() {
        // Check if it's a vx-managed package
        let package_manager = crate::package_manager::PackageManager::new()?;
        let vx_versions = package_manager.list_versions(&tool);
        if vx_versions.is_empty() {
            UI::success(&format!("{tool} is already installed (system)"));
            UI::hint("Use --force to install vx-managed version");
            return Ok(());
        }
    }

    // Clone values for use in async block
    let tool_clone = tool.clone();
    let version_clone = version.clone();

    // Install with progress indication using tracing spans
    let path = with_progress_events!(
        "install_tool_operation",
        "Installation completed successfully",
        "Installation failed",
        async move {
            let mut executor = crate::executor::Executor::new()?;
            executor.install_tool(&tool_clone, &version_clone).await
        }
    )
    .await?;

    UI::info(&format!("Installed to: {}", path.display()));

    // Add to PATH if needed
    if let Some(parent) = path.parent() {
        UI::hint(&format!("Make sure {} is in your PATH", parent.display()));
        UI::hint(&format!(
            "Or use 'vx {tool}' to run the vx-managed version"
        ));
    }

    Ok(())
}
