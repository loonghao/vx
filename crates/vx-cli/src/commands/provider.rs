//! Provider command implementation
//!
//! Manages user-defined providers loaded from provider.star files.
//! Supports adding, removing, listing, and inspecting providers.

use crate::cli::ProviderCommand;
use crate::registry::load_star_overrides;
use crate::ui::UI;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use vx_paths::VxPaths;
use vx_runtime::ProviderRegistry;
use vx_starlark::StarMetadata;

pub async fn handle(registry: &ProviderRegistry, command: ProviderCommand) -> Result<()> {
    match command {
        ProviderCommand::List {
            enabled: _,
            category: _,
        } => {
            // Only show locally installed providers (from ~/.vx/providers/ and .vx/providers/)
            let local_providers = load_star_overrides();

            if local_providers.is_empty() {
                UI::header("Installed Providers");
                UI::info("No local providers installed.");
                UI::hint("Use `vx provider add <path>` to add a provider.");
                return Ok(());
            }

            UI::header(&format!("Installed Providers  ({})", local_providers.len()));

            for (name, content) in &local_providers {
                // Extract description from star content if available
                let description = extract_description_from_star(content)
                    .unwrap_or_else(|| "User provider".to_string());

                UI::item(&format!("📦 {} - {}", name, description));

                // Extract and display runtime names
                let runtimes = extract_runtime_names_from_star(content);
                let last = runtimes.len().saturating_sub(1);
                for (i, runtime_name) in runtimes.iter().enumerate() {
                    let prefix = if i == last {
                        "  └──"
                    } else {
                        "  ├──"
                    };
                    UI::detail(&format!("{} {}", prefix, runtime_name));
                }
            }
        }

        ProviderCommand::Info { name } => {
            UI::header(&format!("Runtime: {name}"));

            if let Some(runtime) = registry.get_runtime(&name) {
                println!("Name: {}", runtime.name());
                println!("Description: {}", runtime.description());
                println!("Ecosystem: {:?}", runtime.ecosystem());

                let aliases = runtime.aliases();
                if !aliases.is_empty() {
                    println!("Aliases: {}", aliases.join(", "));
                }

                let deps = runtime.dependencies();
                if !deps.is_empty() {
                    println!("Dependencies:");
                    for dep in deps {
                        println!("  - {}", dep.name);
                    }
                }
            } else {
                UI::error(&format!("Runtime '{}' not found", name));
            }
        }

        ProviderCommand::Enable { name: _ } => {
            UI::warning("Enable/disable commands not applicable to the provider system");
            UI::hint("All providers are automatically available");
        }

        ProviderCommand::Disable { name: _ } => {
            UI::warning("Enable/disable commands not applicable to the provider system");
            UI::hint("All providers are automatically available");
        }

        ProviderCommand::Search { query } => {
            UI::header(&format!("Runtimes matching '{query}'"));

            let query_lower = query.to_lowercase();
            let mut found = false;

            for name in registry.runtime_names() {
                if name.to_lowercase().contains(&query_lower)
                    && let Some(runtime) = registry.get_runtime(&name)
                {
                    UI::item(&format!("{} - {}", name, runtime.description()));
                    found = true;
                }
            }

            if !found {
                UI::info(&format!("No runtimes found matching '{query}'"));
            }
        }

        ProviderCommand::Stats => {
            UI::header("Provider Statistics");

            let providers = registry.providers();
            let total_providers = providers.len();
            let total_runtimes = registry.runtime_names().len();

            println!("  Total providers: {}", total_providers);
            println!("  Total runtimes: {}", total_runtimes);

            println!("\n  Providers:");
            for provider in providers {
                let runtime_count = provider.runtimes().len();
                println!("    {} ({} runtimes)", provider.name(), runtime_count);
            }
        }

        ProviderCommand::Add { path, name, force } => {
            handle_add(&path, name.as_deref(), force).await?;
        }

        ProviderCommand::Remove { name } => {
            handle_remove(&name)?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// provider add — dispatcher
// ---------------------------------------------------------------------------

/// Entry point: dispatch to HTTP, directory, or single-file handler.
async fn handle_add(path: &str, name_override: Option<&str>, force: bool) -> Result<()> {
    if is_http_url(path) {
        handle_add_url(path, name_override, force).await
    } else {
        let local = Path::new(path);
        if local.is_dir() {
            if name_override.is_some() {
                anyhow::bail!(
                    "--name is not supported when adding a directory (each provider.star \
                     uses its own embedded name)"
                );
            }
            handle_add_dir(local, force)
        } else {
            handle_add_file(local, name_override, force)
        }
    }
}

fn is_http_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

// ---------------------------------------------------------------------------
// HTTP URL
// ---------------------------------------------------------------------------

async fn handle_add_url(url: &str, name_override: Option<&str>, force: bool) -> Result<()> {
    UI::info(&format!("Downloading provider from {url} …"));

    let response = reqwest::get(url)
        .await
        .with_context(|| format!("Failed to fetch {url}"))?;

    if !response.status().is_success() {
        anyhow::bail!("HTTP {} when fetching {url}", response.status());
    }

    let content = response
        .text()
        .await
        .with_context(|| format!("Failed to read response body from {url}"))?;

    // Resolve provider name
    let provider_name = if let Some(n) = name_override {
        n.to_string()
    } else if let Some(n) = extract_name_from_star(&content) {
        n
    } else {
        // Fall back to the last path segment of the URL (without extension)
        url.rsplit('/')
            .next()
            .and_then(|s| s.split('.').next())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "custom".to_string())
    };

    install_star_content(&provider_name, &content, force)?;
    print_success(&provider_name, &content);
    Ok(())
}

// ---------------------------------------------------------------------------
// Local directory — recursive
// ---------------------------------------------------------------------------

fn handle_add_dir(dir: &Path, force: bool) -> Result<()> {
    if !dir.exists() {
        anyhow::bail!("Directory not found: {}", dir.display());
    }

    let star_files = find_star_files_recursive(dir);

    if star_files.is_empty() {
        anyhow::bail!("No provider.star files found under {}", dir.display());
    }

    let mut added = 0usize;
    let mut skipped = 0usize;

    for star_path in &star_files {
        let content = std::fs::read_to_string(star_path)
            .with_context(|| format!("Failed to read {}", star_path.display()))?;

        // Resolve name: embedded name → parent dir name → file stem
        let provider_name = extract_name_from_star(&content).unwrap_or_else(|| {
            star_path
                .parent()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "custom".to_string())
        });

        match install_star_content(&provider_name, &content, force) {
            Ok(()) => {
                print_success(&provider_name, &content);
                added += 1;
            }
            Err(e) if e.to_string().contains("already exists") => {
                UI::warning(&format!(
                    "Skipped '{}' (already exists — use --force to overwrite)",
                    provider_name
                ));
                skipped += 1;
            }
            Err(e) => return Err(e),
        }
    }

    if added > 0 || skipped > 0 {
        println!();
        UI::info(&format!(
            "Done: {} provider(s) added, {} skipped.",
            added, skipped
        ));
    }

    Ok(())
}

/// Recursively find all `provider.star` files under `root`.
fn find_star_files_recursive(root: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    collect_star_recursive(root, &mut results);
    results
}

fn collect_star_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_star_recursive(&path, out);
        } else if path
            .file_name()
            .map(|n| n == "provider.star")
            .unwrap_or(false)
        {
            out.push(path);
        }
    }
}

// ---------------------------------------------------------------------------
// Single file
// ---------------------------------------------------------------------------

fn handle_add_file(src: &Path, name_override: Option<&str>, force: bool) -> Result<()> {
    if !src.exists() {
        anyhow::bail!("File not found: {}", src.display());
    }
    if !src.is_file() {
        anyhow::bail!("Expected a file, got a directory: {}", src.display());
    }

    let content = std::fs::read_to_string(src)
        .with_context(|| format!("Failed to read {}", src.display()))?;

    // Resolve provider name
    let provider_name = if let Some(n) = name_override {
        n.to_string()
    } else if let Some(n) = extract_name_from_star(&content) {
        n
    } else {
        // Fall back to the parent directory name or file stem
        src.parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| {
                src.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "custom".to_string())
            })
    };

    if provider_name.is_empty() {
        anyhow::bail!("Could not determine provider name. Use --name to specify one.");
    }

    install_star_content(&provider_name, &content, force)?;
    print_success(&provider_name, &content);
    Ok(())
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Write `content` to `~/.vx/providers/<name>/provider.star`.
fn install_star_content(provider_name: &str, content: &str, force: bool) -> Result<()> {
    let vx_paths = VxPaths::new().context("Failed to resolve VX home directory")?;
    let dest_dir = vx_paths.base_dir.join("providers").join(provider_name);
    let dest_file = dest_dir.join("provider.star");

    if dest_file.exists() && !force {
        anyhow::bail!(
            "Provider '{}' already exists at {}\nUse --force to overwrite.",
            provider_name,
            dest_file.display()
        );
    }

    std::fs::create_dir_all(&dest_dir)
        .with_context(|| format!("Failed to create directory {}", dest_dir.display()))?;

    std::fs::write(&dest_file, content)
        .with_context(|| format!("Failed to write {}", dest_file.display()))?;

    Ok(())
}

/// Print success message with the runtime names exposed by this provider.
fn print_success(provider_name: &str, content: &str) {
    let vx_paths = VxPaths::new().ok();
    let dest_file = vx_paths
        .map(|p| {
            p.base_dir
                .join("providers")
                .join(provider_name)
                .join("provider.star")
        })
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| format!("~/.vx/providers/{}/provider.star", provider_name));

    UI::success(&format!(
        "Provider '{}' added → {}",
        provider_name, dest_file
    ));

    // Extract runtime names from the star content for a helpful hint
    let runtimes = extract_runtime_names_from_star(content);
    if runtimes.is_empty() {
        UI::hint("Run `vx <runtime>` to use it (restart vx if already running)");
    } else if runtimes.len() == 1 {
        UI::hint(&format!(
            "Run `vx {}` to use it (restart vx if already running)",
            runtimes[0]
        ));
    } else {
        UI::hint(&format!(
            "Available runtimes: {}  (restart vx if already running)",
            runtimes
                .iter()
                .map(|r| format!("`vx {r}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
}

/// Extract the `description = "..."` field from a provider.star source.
fn extract_description_from_star(source: &str) -> Option<String> {
    let meta = StarMetadata::parse(source);
    meta.description
}

/// Extract the `name = "..."` field from a provider.star source.
fn extract_name_from_star(source: &str) -> Option<String> {
    let meta = StarMetadata::parse(source);
    meta.name
}

/// Extract runtime names from `runtime_def("name", ...)` calls in a provider.star.
fn extract_runtime_names_from_star(source: &str) -> Vec<String> {
    let meta = StarMetadata::parse(source);
    meta.runtimes
        .iter()
        .filter_map(|rt| rt.name.clone())
        .collect()
}

// ---------------------------------------------------------------------------
// provider remove
// ---------------------------------------------------------------------------

/// Remove a user provider from `~/.vx/providers/<name>/`.
fn handle_remove(name: &str) -> Result<()> {
    let vx_paths = VxPaths::new().context("Failed to resolve VX home directory")?;
    let dest_dir = vx_paths.base_dir.join("providers").join(name);

    if !dest_dir.exists() {
        anyhow::bail!("Provider '{}' not found in user providers directory.", name);
    }

    std::fs::remove_dir_all(&dest_dir)
        .with_context(|| format!("Failed to remove {}", dest_dir.display()))?;

    UI::success(&format!("Provider '{}' removed.", name));
    Ok(())
}
