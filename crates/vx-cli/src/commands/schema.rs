//! `vx schema` command — runtime introspection for AI agents
//!
//! Allows AI agents to discover what a provider accepts at runtime,
//! without consuming context-window tokens on static docs.
//!
//! Inspired by Google Cloud `gws schema <method>` pattern from
//! Justin Poehnelt's "Rewrite Your CLI for AI Agents" article.
//!
//! ## Usage
//!
//! ```text
//! vx schema node          # Dump schema for the node runtime
//! vx schema --all         # Dump schemas for all registered runtimes (NDJSON)
//! vx schema --commands    # List all available vx sub-commands as JSON
//! ```

use anyhow::Result;
use serde::Serialize;
use vx_runtime::{Platform, ProviderRegistry};

// ============================================================================
// Output types
// ============================================================================

/// Schema descriptor for a single runtime/provider.
#[derive(Serialize)]
pub struct RuntimeSchema {
    /// Runtime name (primary)
    pub name: String,
    /// Aliases (e.g. "nodejs" for "node")
    pub aliases: Vec<String>,
    /// Human-readable description
    pub description: String,
    /// Ecosystem (nodejs, python, rust, go, system, custom)
    pub ecosystem: String,
    /// Whether this runtime is supported on the current platform
    pub platform_supported: bool,
    /// Version constraint syntax examples
    pub version_examples: Vec<String>,
    /// How to install this runtime
    pub install_command: String,
    /// How to execute this runtime
    pub execute_example: String,
    /// All runtimes bundled with this provider (e.g. npm, npx bundled with node)
    pub bundled_runtimes: Vec<String>,
}

/// Schema for a vx CLI sub-command (for `--commands` mode).
#[derive(Serialize)]
pub struct CommandSchema {
    /// Command name
    pub name: String,
    /// Aliases
    pub aliases: Vec<&'static str>,
    /// Short description
    pub description: String,
    /// Key flags
    pub flags: Vec<FlagSchema>,
}

/// Description of a single CLI flag.
#[derive(Serialize)]
pub struct FlagSchema {
    /// Flag name (without --)
    pub name: String,
    /// Short flag if any
    pub short: Option<char>,
    /// Description
    pub description: String,
    /// Whether this flag takes a value
    pub takes_value: bool,
    /// Whether this is a boolean flag
    pub is_bool: bool,
    /// Default value if any
    pub default: Option<String>,
}

// ============================================================================
// Command handler
// ============================================================================

/// Handle `vx schema [runtime]` or `vx schema --all` or `vx schema --commands`
pub async fn handle(
    registry: &ProviderRegistry,
    runtime: Option<&str>,
    all: bool,
    commands: bool,
) -> Result<()> {
    if commands {
        return handle_commands_schema();
    }

    if all {
        // Emit NDJSON: one JSON object per line
        for name in registry.runtime_names() {
            if let Some(schema) = build_runtime_schema(registry, &name) {
                println!("{}", serde_json::to_string(&schema)?);
            }
        }
        return Ok(());
    }

    let runtime_name = runtime.ok_or_else(|| {
        anyhow::anyhow!(
            "Usage: vx schema <runtime>\n       vx schema --all\n       vx schema --commands\n\nRun 'vx list' to see available runtimes."
        )
    })?;

    match build_runtime_schema(registry, runtime_name) {
        Some(schema) => {
            println!("{}", serde_json::to_string_pretty(&schema)?);
        }
        None => {
            anyhow::bail!(
                "Unknown runtime '{}'. Run 'vx list' to see available runtimes.",
                runtime_name
            );
        }
    }

    Ok(())
}

// ============================================================================
// Internal helpers
// ============================================================================

fn build_runtime_schema(registry: &ProviderRegistry, name: &str) -> Option<RuntimeSchema> {
    let runtime = registry.get_runtime(name)?;
    let ecosystem = runtime.ecosystem().to_string();

    // Build list of bundled runtimes via the global starlark registry
    let bundled: Vec<String> = vx_starlark::handle::GLOBAL_REGISTRY
        .try_read()
        .ok()
        .and_then(|reg| {
            let handle = reg.get(name)?;
            let bundled: Vec<String> = handle
                .runtime_metas()
                .iter()
                .filter(|r| r.name != name && r.bundled_with.is_some())
                .map(|r| r.name.clone())
                .collect();
            Some(bundled)
        })
        .unwrap_or_default();

    // Check platform support using current platform
    let current_platform = Platform::current();
    let platform_supported = runtime.is_platform_supported(&current_platform);

    Some(RuntimeSchema {
        name: runtime.name().to_string(),
        aliases: runtime.aliases().iter().map(|s| s.to_string()).collect(),
        description: runtime.description().to_string(),
        ecosystem,
        platform_supported,
        version_examples: vec![
            format!("vx {} --version", name),
            format!("vx install {}@latest", name),
            format!("vx install {}@<semver>", name),
        ],
        install_command: format!("vx install {}", name),
        execute_example: format!("vx {} --version", name),
        bundled_runtimes: bundled,
    })
}

fn handle_commands_schema() -> Result<()> {
    let cmd_list: Vec<CommandSchema> = vec![
        CommandSchema {
            name: "install".to_string(),
            aliases: vec!["i"],
            description: "Install tool(s) by name and optional version".to_string(),
            flags: vec![
                FlagSchema {
                    name: "force".to_string(),
                    short: Some('f'),
                    description: "Force reinstallation even if already installed".to_string(),
                    takes_value: false,
                    is_bool: true,
                    default: None,
                },
                FlagSchema {
                    name: "dry-run".to_string(),
                    short: None,
                    description: "Preview install without executing".to_string(),
                    takes_value: false,
                    is_bool: true,
                    default: None,
                },
            ],
        },
        CommandSchema {
            name: "list".to_string(),
            aliases: vec!["ls"],
            description: "List available and installed runtimes".to_string(),
            flags: vec![
                FlagSchema {
                    name: "installed".to_string(),
                    short: None,
                    description: "Show only installed tools".to_string(),
                    takes_value: false,
                    is_bool: true,
                    default: None,
                },
                FlagSchema {
                    name: "fields".to_string(),
                    short: None,
                    description: "Comma-separated field mask (e.g. name,version,ecosystem)"
                        .to_string(),
                    takes_value: true,
                    is_bool: false,
                    default: None,
                },
            ],
        },
        CommandSchema {
            name: "versions".to_string(),
            aliases: vec![],
            description: "Show available versions for a tool".to_string(),
            flags: vec![
                FlagSchema {
                    name: "latest".to_string(),
                    short: None,
                    description: "Limit to N latest versions".to_string(),
                    takes_value: true,
                    is_bool: false,
                    default: Some("10".to_string()),
                },
                FlagSchema {
                    name: "fields".to_string(),
                    short: None,
                    description: "Comma-separated field mask (e.g. version,lts,installed)"
                        .to_string(),
                    takes_value: true,
                    is_bool: false,
                    default: None,
                },
            ],
        },
        CommandSchema {
            name: "which".to_string(),
            aliases: vec!["where"],
            description: "Show which version and path is being used".to_string(),
            flags: vec![FlagSchema {
                name: "all".to_string(),
                short: Some('a'),
                description: "Show all installed versions".to_string(),
                takes_value: false,
                is_bool: true,
                default: None,
            }],
        },
        CommandSchema {
            name: "schema".to_string(),
            aliases: vec![],
            description: "Introspect CLI/runtime schema (for AI agents)".to_string(),
            flags: vec![
                FlagSchema {
                    name: "all".to_string(),
                    short: Some('a'),
                    description: "Dump schemas for all runtimes as NDJSON".to_string(),
                    takes_value: false,
                    is_bool: true,
                    default: None,
                },
                FlagSchema {
                    name: "commands".to_string(),
                    short: Some('c'),
                    description: "List all vx sub-commands as JSON".to_string(),
                    takes_value: false,
                    is_bool: true,
                    default: None,
                },
            ],
        },
        CommandSchema {
            name: "sync".to_string(),
            aliases: vec![],
            description: "Sync project tools from vx.toml".to_string(),
            flags: vec![
                FlagSchema {
                    name: "check".to_string(),
                    short: None,
                    description: "Only check, don't install".to_string(),
                    takes_value: false,
                    is_bool: true,
                    default: None,
                },
                FlagSchema {
                    name: "dry-run".to_string(),
                    short: None,
                    description: "Preview operations without executing".to_string(),
                    takes_value: false,
                    is_bool: true,
                    default: None,
                },
            ],
        },
        CommandSchema {
            name: "check".to_string(),
            aliases: vec![],
            description: "Check version constraints and tool availability".to_string(),
            flags: vec![FlagSchema {
                name: "detailed".to_string(),
                short: Some('d'),
                description: "Show detailed info for each tool".to_string(),
                takes_value: false,
                is_bool: true,
                default: None,
            }],
        },
    ];

    println!("{}", serde_json::to_string_pretty(&cmd_list)?);
    Ok(())
}
