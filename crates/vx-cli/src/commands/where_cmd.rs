//! Where command — find the executable path for a vx-managed tool.
//!
//! # Architecture (RFC-0037)
//!
//! `ProviderHandle` is the **single source of truth** for vx-managed paths.
//! All path queries delegate to `provider.star` via `ProviderHandle`.
//!
//! Lookup priority:
//! 1. `ProviderHandle` (provider.star convention-based path scanning)
//! 2. Global packages (`~/.vx/packages/`)
//! 3. System PATH
//! 4. `provider.star::runtimes[].system_paths` glob patterns

use crate::cli::OutputFormat;
use crate::output::{OutputRenderer, ToolPathEntry, ToolSource, WhichOutput};
use crate::suggestions;
use crate::ui::UI;
use anyhow::Result;
use colored::Colorize;
use vx_paths::{PackageRegistry, VxPaths};
use vx_runtime::ProviderRegistry;
use vx_starlark::handle::global_registry;

pub async fn handle(
    registry: &ProviderRegistry,
    tool: &str,
    version: Option<&str>,
    all: bool,
    use_system_path: bool,
    format: OutputFormat,
) -> Result<()> {
    UI::debug(&format!("Looking for tool: {}@{:?}", tool, version));

    // Parse provider::runtime syntax (e.g., msvc::signtool, msvc::cl)
    //
    // Semantics: `provider::runtime` means "the `runtime` runtime under the `provider` provider".
    // The right-hand side is first tried as a runtime name; if not found in the registry,
    // it falls back to being treated as an executable override (legacy behaviour).
    let (runtime_part, exe_override) = if let Some((provider_hint, rhs)) = tool.split_once("::") {
        // Check if `rhs` is itself a known runtime name in the global registry
        let rhs_is_runtime = {
            let reg = global_registry().await;
            reg.get(rhs).is_some()
        };
        if rhs_is_runtime {
            // e.g. msvc::signtool → look up "signtool" as the runtime
            (rhs, None)
        } else {
            // e.g. msvc::cl (legacy exe-override) → look up "msvc", exe = "cl"
            (provider_hint, Some(rhs))
        }
    } else {
        (tool, None)
    };

    // If --use-system-path is specified, only check system PATH
    if use_system_path {
        let search_name = exe_override.unwrap_or(runtime_part);
        match which::which(search_name) {
            Ok(path) => {
                let output = WhichOutput {
                    tool: tool.to_string(),
                    version: None,
                    path: Some(path.display().to_string()),
                    source: ToolSource::Vx,
                    all_paths: vec![],
                };
                OutputRenderer::new(format).render(&output)?;
                return Ok(());
            }
            Err(_) => {
                let output = WhichOutput {
                    tool: tool.to_string(),
                    version: None,
                    path: None,
                    source: ToolSource::NotFound,
                    all_paths: vec![],
                };
                OutputRenderer::new(format).render(&output)?;
                std::process::exit(1);
            }
        }
    }

    // Resolve canonical runtime name and executable via ProviderRegistry
    let (canonical_name, exe_name) = if let Some(exe) = exe_override {
        let rt_name = registry
            .get_runtime(runtime_part)
            .map(|r| r.name().to_string())
            .unwrap_or_else(|| runtime_part.to_string());
        (rt_name, exe.to_string())
    } else if let Some(runtime) = registry.get_runtime(runtime_part) {
        let canonical = runtime.name().to_string();
        let exe = runtime.executable_name().to_string();
        UI::debug(&format!(
            "Resolved '{}' → canonical='{}', exe='{}'",
            tool, canonical, exe
        ));
        (canonical, exe)
    } else {
        (runtime_part.to_string(), runtime_part.to_string())
    };

    // ── Step 1: ProviderHandle (provider.star) ────────────────────────────
    let locations: Vec<(std::path::PathBuf, ToolSource)> = {
        let reg = global_registry().await;
        if let Some(handle) = reg.get(&canonical_name) {
            if all {
                // Return all installed versions
                handle
                    .installed_versions()
                    .into_iter()
                    .filter_map(|ver| handle.get_execute_path(&ver).map(|p| (p, ToolSource::Vx)))
                    .collect()
            } else if let Some(ver) = version {
                // Specific version requested
                handle
                    .get_execute_path(ver)
                    .map(|p| vec![(p, ToolSource::Vx)])
                    .unwrap_or_default()
            } else {
                // Latest installed version
                handle
                    .get_latest_execute_path()
                    .map(|p| vec![(p, ToolSource::Vx)])
                    .unwrap_or_default()
            }
        } else {
            UI::debug(&format!(
                "No ProviderHandle for '{}', skipping vx store lookup",
                canonical_name
            ));
            vec![]
        }
    };

    // Filter to only paths that actually exist on disk
    let locations: Vec<(std::path::PathBuf, ToolSource)> =
        locations.into_iter().filter(|(p, _)| p.exists()).collect();

    // ── Step 2: Global packages / System PATH / system_paths ─────────────
    let (final_path, final_source, all_paths) = if !locations.is_empty() {
        if all {
            let all_paths = locations
                .iter()
                .map(|(path, source)| ToolPathEntry {
                    path: path.display().to_string(),
                    version: extract_version_from_path(path),
                    source: *source,
                })
                .collect();
            (None, ToolSource::NotFound, all_paths)
        } else {
            let (path, source) = &locations[0];
            (Some(path.display().to_string()), *source, vec![])
        }
    } else {
        // Fallback chain: global packages → system PATH → system_paths
        resolve_fallback(tool, &exe_name, &canonical_name, all).await?
    };

    let renderer = OutputRenderer::new(format);

    // Handle not found
    if final_path.is_none() && all_paths.is_empty() {
        let output = WhichOutput {
            tool: tool.to_string(),
            version: None,
            path: None,
            source: ToolSource::NotFound,
            all_paths: vec![],
        };

        if renderer.is_json() {
            renderer.render(&output)?;
        } else {
            let available_tools = registry.runtime_names();
            let suggestions = suggestions::get_tool_suggestions(tool, &available_tools);

            eprintln!(
                "{} {}",
                "✗".red(),
                format!(
                    "Tool '{}' not found in vx-managed installations or system PATH",
                    tool
                )
                .red()
            );

            if !suggestions.is_empty() {
                eprintln!();
                for s in &suggestions {
                    if s.is_alias {
                        eprintln!(
                            "{} Did you mean: {} ({})",
                            "💡".cyan(),
                            s.suggested_tool.cyan().bold(),
                            s.description.dimmed()
                        );
                    } else {
                        eprintln!(
                            "{} Did you mean: {}",
                            "💡".cyan(),
                            s.suggested_tool.cyan().bold()
                        );
                    }
                }
            }

            eprintln!();
            eprintln!(
                "{} {}",
                "💡".cyan(),
                "Use 'vx list' to see installed tools".dimmed()
            );
            eprintln!(
                "{} {}",
                "💡".cyan(),
                format!("Use 'vx install {}' to install this tool", tool).dimmed()
            );
        }
        std::process::exit(1);
    }

    let output = WhichOutput {
        tool: tool.to_string(),
        version: final_path
            .as_ref()
            .and_then(|p| extract_version_from_path_str(p)),
        path: final_path,
        source: final_source,
        all_paths,
    };

    renderer.render(&output)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Fallback resolution (global packages → system PATH → system_paths)
// ---------------------------------------------------------------------------

async fn resolve_fallback(
    tool: &str,
    exe_name: &str,
    canonical_name: &str,
    all: bool,
) -> Result<(Option<String>, ToolSource, Vec<ToolPathEntry>)> {
    // 1. Global packages
    if let Some(path) = find_in_global_packages(tool)? {
        return Ok(make_single_or_all(path, ToolSource::GlobalPackage, all));
    }
    if exe_name != tool
        && let Some(path) = find_in_global_packages(exe_name)?
    {
        return Ok(make_single_or_all(path, ToolSource::GlobalPackage, all));
    }

    // 2. System PATH
    if let Ok(path) = which::which(exe_name) {
        return Ok(make_single_or_all(path, ToolSource::System, all));
    }

    // 3. provider.star system_paths glob patterns
    if let Some(path) = find_via_system_paths(canonical_name).await? {
        return Ok(make_single_or_all(path, ToolSource::Detected, all));
    }

    Ok((None, ToolSource::NotFound, vec![]))
}

fn make_single_or_all(
    path: std::path::PathBuf,
    source: ToolSource,
    all: bool,
) -> (Option<String>, ToolSource, Vec<ToolPathEntry>) {
    if all {
        (
            None,
            ToolSource::NotFound,
            vec![ToolPathEntry {
                path: path.display().to_string(),
                version: None,
                source,
            }],
        )
    } else {
        (Some(path.display().to_string()), source, vec![])
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Find an executable in globally installed packages (`~/.vx/packages/`)
fn find_in_global_packages(exe_name: &str) -> Result<Option<std::path::PathBuf>> {
    let paths = VxPaths::new()?;
    let registry_path = paths.packages_registry_file();

    UI::debug(&format!(
        "Looking for '{}' in global packages: {}",
        exe_name,
        registry_path.display()
    ));

    let registry = match PackageRegistry::load(&registry_path) {
        Ok(r) => r,
        Err(e) => {
            UI::debug(&format!("Failed to load package registry: {}", e));
            return Ok(None);
        }
    };

    if let Some(package) = registry.find_by_executable(exe_name) {
        let bin_dir =
            paths.global_package_bin_dir(&package.ecosystem, &package.name, &package.version);
        let candidates = [
            #[cfg(windows)]
            package.install_dir.join(format!("{}.cmd", exe_name)),
            #[cfg(windows)]
            package.install_dir.join(format!("{}.ps1", exe_name)),
            #[cfg(windows)]
            package.install_dir.join(format!("{}.exe", exe_name)),
            package.install_dir.join(exe_name),
            package
                .install_dir
                .join("bin")
                .join(format!("{}.cmd", exe_name)),
            package
                .install_dir
                .join("bin")
                .join(format!("{}.ps1", exe_name)),
            package
                .install_dir
                .join("bin")
                .join(format!("{}.exe", exe_name)),
            package.install_dir.join("bin").join(exe_name),
            bin_dir.join(format!("{}.cmd", exe_name)),
            bin_dir.join(exe_name),
        ];

        for candidate in &candidates {
            if candidate.exists() {
                return Ok(Some(candidate.clone()));
            }
        }
    }

    Ok(None)
}

/// Find an executable via `provider.star::runtimes[].system_paths` glob patterns
async fn find_via_system_paths(runtime_name: &str) -> Result<Option<std::path::PathBuf>> {
    let reg = global_registry().await;
    if let Some(handle) = reg.get(runtime_name) {
        for runtime_meta in handle.runtime_metas() {
            for pattern in &runtime_meta.system_paths {
                if let Ok(paths) = glob::glob(pattern) {
                    let mut found: Vec<std::path::PathBuf> = paths
                        .filter_map(|p| p.ok())
                        .filter(|p| p.exists())
                        .collect();
                    // Sort descending so newest version wins (e.g. VS 2022 > 2019)
                    found.sort_by(|a, b| b.cmp(a));
                    if let Some(path) = found.into_iter().next() {
                        UI::debug(&format!(
                            "Found '{}' via system_paths: {}",
                            runtime_name,
                            path.display()
                        ));
                        return Ok(Some(path));
                    }
                }
            }
        }
    }
    Ok(None)
}

/// Extract version string from an executable path
///
/// Looks for a path component that looks like a version number (e.g. `20.0.0`).
fn extract_version_from_path(path: &std::path::Path) -> Option<String> {
    for ancestor in path.ancestors() {
        if let Some(name) = ancestor.file_name().and_then(|n| n.to_str())
            && name.chars().any(|c| c.is_ascii_digit())
            && (name.contains('.') || name.chars().all(|c| c.is_ascii_digit()))
            && !name.contains('-')
        {
            return Some(name.to_string());
        }
    }
    None
}

fn extract_version_from_path_str(path_str: &str) -> Option<String> {
    extract_version_from_path(std::path::Path::new(path_str))
}
