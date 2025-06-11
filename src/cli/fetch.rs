// Fetch command implementation
// Shows available versions for tools with interactive selection

use crate::ui::UI;
use crate::{with_progress_events, with_progress_span};
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Select};
use tracing::instrument;

#[instrument(name = "fetch_command", fields(tool = %tool, latest = ?latest, prerelease = prerelease))]
pub async fn handle(
    tool: String,
    latest: Option<usize>,
    prerelease: bool,
    detailed: bool,
    interactive: bool,
) -> Result<()> {
    UI::header(&format!("Available versions for {}", tool));

    // Clone tool for use in async blocks
    let tool_clone = tool.clone();
    let tool_clone2 = tool.clone();

    // Get available versions using tracing spans (community best practice)
    let versions = with_progress_events!(
        "fetch_tool_versions",
        "Found versions successfully",
        "Failed to fetch versions",
        fetch_tool_versions(&tool_clone, prerelease)
    )
    .await?;

    if versions.is_empty() {
        UI::warning(&format!("No versions found for tool '{}'", tool));
        UI::hint("Check if the tool name is correct");
        UI::hint("Run 'vx list --all' to see supported tools");
        return Ok(());
    }

    // Limit results if requested
    let display_versions = if let Some(limit) = latest {
        versions.into_iter().take(limit).collect()
    } else {
        versions
    };

    // Get installed versions using tracing span
    let (installed_versions, active_version) =
        with_progress_span!("check_installed_versions", async move {
            let package_manager = crate::package_manager::PackageManager::new()?;
            let installed_versions: Vec<String> = package_manager
                .list_versions(&tool_clone2)
                .iter()
                .map(|p| p.version.clone())
                .collect();

            // Get current active version
            let active_version = get_active_version(&tool_clone2).await;

            Ok::<(Vec<String>, Option<String>), anyhow::Error>((installed_versions, active_version))
        })
        .await?;

    if interactive {
        handle_interactive_selection(
            &tool,
            &display_versions,
            &installed_versions,
            &active_version,
        )
        .await
    } else {
        display_versions_list(
            &tool,
            &display_versions,
            &installed_versions,
            &active_version,
            detailed,
        )
        .await
    }
}

async fn display_versions_list(
    tool: &str,
    versions: &[VersionInfo],
    installed_versions: &[String],
    active_version: &Option<String>,
    detailed: bool,
) -> Result<()> {
    println!();

    for (index, version) in versions.iter().enumerate() {
        let mut status_indicators = Vec::new();

        // Check if this version is installed
        let is_installed = installed_versions.contains(&version.version);
        if is_installed {
            status_indicators.push("installed".to_string());
        }

        // Check if this is the active version
        let is_active = active_version.as_ref() == Some(&version.version);
        if is_active {
            status_indicators.push("active".to_string());
        }

        // Check if this is the latest version
        if index == 0 {
            status_indicators.push("latest".to_string());
        }

        // Check if this is a prerelease
        if version.is_prerelease {
            status_indicators.push("prerelease".to_string());
        }

        // Format status
        let status_str = if status_indicators.is_empty() {
            String::new()
        } else {
            format!(" <{}>", status_indicators.join(", "))
        };

        // Display version
        let icon = if is_active {
            "→"
        } else if is_installed {
            "✓"
        } else {
            " "
        };

        if detailed {
            println!(
                "  {} {} {}{}",
                icon,
                version.version,
                status_str,
                if let Some(date) = &version.release_date {
                    format!(" ({})", date)
                } else {
                    String::new()
                }
            );

            if let Some(notes) = &version.release_notes {
                let truncated_notes = if notes.len() > 100 {
                    format!("{}...", &notes[..97])
                } else {
                    notes.clone()
                };
                println!("    {}", truncated_notes);
            }
        } else {
            println!("  {} {}{}", icon, version.version, status_str);
        }
    }

    println!();

    // Show summary
    let installed_count = versions
        .iter()
        .filter(|v| installed_versions.contains(&v.version))
        .count();

    UI::info(&format!(
        "Found {} versions ({} installed)",
        versions.len(),
        installed_count
    ));

    // Show usage hints
    if let Some(latest) = versions.first() {
        if !installed_versions.contains(&latest.version) {
            UI::hint(&format!(
                "Install latest: vx install {}@{}",
                tool, latest.version
            ));
        }
    }

    UI::hint(&format!(
        "Install specific version: vx install {}@<version>",
        tool
    ));
    UI::hint(&format!("Switch version: vx switch {}@<version>", tool));

    Ok(())
}

async fn handle_interactive_selection(
    tool: &str,
    versions: &[VersionInfo],
    installed_versions: &[String],
    active_version: &Option<String>,
) -> Result<()> {
    if versions.is_empty() {
        return Ok(());
    }

    // Create selection items
    let items: Vec<String> = versions
        .iter()
        .map(|v| {
            let mut status_parts = Vec::new();

            if installed_versions.contains(&v.version) {
                status_parts.push("installed");
            }

            if active_version.as_ref() == Some(&v.version) {
                status_parts.push("active");
            }

            if versions.first().map(|first| &first.version) == Some(&v.version) {
                status_parts.push("latest");
            }

            if v.is_prerelease {
                status_parts.push("prerelease");
            }

            let status_str = if status_parts.is_empty() {
                String::new()
            } else {
                format!(" ({})", status_parts.join(", "))
            };

            format!("{}{}", v.version, status_str)
        })
        .collect();

    // Add action options
    let mut all_items = items;
    all_items.push("──────────────".to_string());
    all_items.push("📋 Show detailed info".to_string());
    all_items.push("🔄 Refresh versions".to_string());
    all_items.push("❌ Cancel".to_string());

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Select a version of {} to install/switch to:",
            tool
        ))
        .items(&all_items)
        .default(0)
        .interact_opt()?;

    match selection {
        Some(index) => {
            if index < versions.len() {
                // User selected a version
                let selected_version = &versions[index];

                if installed_versions.contains(&selected_version.version) {
                    // Version is already installed, offer to switch
                    UI::info(&format!(
                        "Version {} is already installed",
                        selected_version.version
                    ));

                    if UI::confirm(
                        &format!("Switch to version {}?", selected_version.version),
                        true,
                    )? {
                        let tool_version = format!("{}@{}", tool, selected_version.version);
                        crate::cli::switch::handle(tool_version, false).await?;
                    }
                } else {
                    // Version not installed, offer to install
                    if UI::confirm(
                        &format!("Install version {}?", selected_version.version),
                        true,
                    )? {
                        crate::cli::install::handle(
                            tool.to_string(),
                            Some(selected_version.version.clone()),
                            false,
                        )
                        .await?;
                    }
                }
            } else {
                // Handle special actions
                let action_index = index - versions.len() - 1;
                match action_index {
                    0 => {
                        // Show detailed info
                        display_versions_list(
                            tool,
                            versions,
                            installed_versions,
                            active_version,
                            true,
                        )
                        .await?;
                    }
                    1 => {
                        // Refresh versions
                        UI::info("Refreshing version information...");
                        UI::hint("Refresh functionality would reload version data");
                    }
                    2 => {
                        // Cancel
                        UI::info("Operation cancelled");
                    }
                    _ => {}
                }
            }
        }
        None => {
            UI::info("Operation cancelled");
        }
    }

    Ok(())
}

async fn fetch_tool_versions(tool: &str, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
    let spinner = UI::new_spinner(&format!("Fetching versions for {}...", tool));

    let versions = match tool {
        "uv" => fetch_uv_versions(include_prerelease).await?,
        "node" => fetch_node_versions(include_prerelease).await?,
        "go" => fetch_go_versions(include_prerelease).await?,
        "rust" => fetch_rust_versions(include_prerelease).await?,
        "python" => fetch_python_versions(include_prerelease).await?,
        _ => {
            spinner.finish_and_clear();
            return Err(anyhow::anyhow!(
                "Version fetching not implemented for {}",
                tool
            ));
        }
    };

    spinner.finish_and_clear();
    Ok(versions)
}

async fn get_active_version(tool: &str) -> Option<String> {
    // Try to get version from system PATH
    if let Ok(output) = std::process::Command::new(tool).arg("--version").output() {
        if output.status.success() {
            let version_output = String::from_utf8_lossy(&output.stdout);
            if let Ok(version) =
                crate::version::VersionManager::extract_version_from_output(&version_output)
            {
                return Some(version);
            }
        }
    }

    None
}

// Version information structure
#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub version: String,
    pub is_prerelease: bool,
    pub release_date: Option<String>,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
}

// Tool-specific version fetchers
async fn fetch_uv_versions(include_prerelease: bool) -> Result<Vec<VersionInfo>> {
    // Fetch from GitHub releases API
    let url = "https://api.github.com/repos/astral-sh/uv/releases";
    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .header("User-Agent", "vx-tool-manager")
        .send()
        .await?;

    let releases: serde_json::Value = response.json().await?;

    let mut versions = Vec::new();

    if let Some(releases_array) = releases.as_array() {
        for release in releases_array {
            let version = release["tag_name"].as_str().unwrap_or("").to_string();
            let is_prerelease = release["prerelease"].as_bool().unwrap_or(false);

            if !include_prerelease && is_prerelease {
                continue;
            }

            let release_date = release["published_at"]
                .as_str()
                .map(|s| s.split('T').next().unwrap_or(s).to_string());

            let release_notes = release["body"].as_str().map(|s| {
                // Truncate long release notes
                if s.len() > 200 {
                    format!("{}...", &s[..197])
                } else {
                    s.to_string()
                }
            });

            versions.push(VersionInfo {
                version,
                is_prerelease,
                release_date,
                release_notes,
                download_url: None,
            });
        }
    }

    Ok(versions)
}

async fn fetch_node_versions(_include_prerelease: bool) -> Result<Vec<VersionInfo>> {
    // For now, return some mock data
    // TODO: Implement actual Node.js version fetching
    Ok(vec![
        VersionInfo {
            version: "20.10.0".to_string(),
            is_prerelease: false,
            release_date: Some("2023-11-22".to_string()),
            release_notes: Some("LTS release with performance improvements".to_string()),
            download_url: None,
        },
        VersionInfo {
            version: "18.19.0".to_string(),
            is_prerelease: false,
            release_date: Some("2023-11-29".to_string()),
            release_notes: Some("LTS maintenance release".to_string()),
            download_url: None,
        },
    ])
}

async fn fetch_go_versions(_include_prerelease: bool) -> Result<Vec<VersionInfo>> {
    // For now, return some mock data
    // TODO: Implement actual Go version fetching
    Ok(vec![
        VersionInfo {
            version: "1.21.6".to_string(),
            is_prerelease: false,
            release_date: Some("2024-01-09".to_string()),
            release_notes: Some("Security and bug fixes".to_string()),
            download_url: None,
        },
        VersionInfo {
            version: "1.21.5".to_string(),
            is_prerelease: false,
            release_date: Some("2023-12-05".to_string()),
            release_notes: Some("Bug fixes and improvements".to_string()),
            download_url: None,
        },
    ])
}

async fn fetch_rust_versions(_include_prerelease: bool) -> Result<Vec<VersionInfo>> {
    // For now, return some mock data
    // TODO: Implement actual Rust version fetching
    Ok(vec![
        VersionInfo {
            version: "1.75.0".to_string(),
            is_prerelease: false,
            release_date: Some("2023-12-28".to_string()),
            release_notes: Some("Stable release with new features".to_string()),
            download_url: None,
        },
        VersionInfo {
            version: "1.74.1".to_string(),
            is_prerelease: false,
            release_date: Some("2023-12-07".to_string()),
            release_notes: Some("Bug fix release".to_string()),
            download_url: None,
        },
    ])
}

async fn fetch_python_versions(_include_prerelease: bool) -> Result<Vec<VersionInfo>> {
    // For now, return some mock data
    // TODO: Implement actual Python version fetching
    Ok(vec![
        VersionInfo {
            version: "3.12.1".to_string(),
            is_prerelease: false,
            release_date: Some("2023-12-07".to_string()),
            release_notes: Some("Latest stable release".to_string()),
            download_url: None,
        },
        VersionInfo {
            version: "3.11.7".to_string(),
            is_prerelease: false,
            release_date: Some("2023-12-04".to_string()),
            release_notes: Some("Security and bug fixes".to_string()),
            download_url: None,
        },
    ])
}
