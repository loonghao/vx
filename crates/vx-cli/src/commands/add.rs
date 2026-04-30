//! `vx add` — add one or more tools to the project (vx.toml + vx.lock).
//!
//! This command is inspired by mainstream toolchains (`cargo add`, `uv add`,
//! `pnpm add`, `bun add`). Unlike the legacy single-tool add, it:
//!
//! 1. Accepts **multiple specs** at once (`vx add node@22 python@3.12 ripgrep`)
//! 2. Edits `vx.toml` **preserving comments & formatting** (via `toml_edit`)
//! 3. Resolves versions and updates `vx.lock` incrementally
//! 4. Installs the tools by default (opt out with `--no-install`)
//! 5. Validates each tool name against the `ProviderRegistry`
//!
//! ## Spec grammar
//!
//! Each spec is `name[@version]`. The version may be `latest`, a semver
//! (`22.14.0`), a partial (`22`, `3.11`), or a range (`^1.2`, `~3.11`).
//!
//! ## Flags
//!
//! | Flag | Meaning |
//! |------|---------|
//! | `--no-install`      | Only edit `vx.toml` + `vx.lock`, don't install |
//! | `--no-lock`         | Don't update `vx.lock` |
//! | `--frozen`          | Don't modify `vx.lock`; fail if resolution would change it |
//! | `--dev`             | (reserved) mark as dev-only dependency |
//! | `--dry-run`         | Print planned changes without writing |
//! | `--os <list>`       | Restrict tool to platforms (comma-separated `windows,linux,macos`) |
//! | `--force`           | Overwrite existing entry in `vx.toml` |

use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use toml_edit::{Array, DocumentMut, Item, Table, Value};
use vx_paths::project::{LOCK_FILE_NAME, find_vx_config};
use vx_resolver::{LockFile, RuntimeRequest};
use vx_runtime::ProviderRegistry;

use crate::ui::UI;

/// Parsed tool specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddRuntimeSpec {
    pub name: String,
    pub version: String,
}

impl AddRuntimeSpec {
    /// Parse a `name[@version]` spec. Defaults version to `latest`.
    pub fn parse(raw: &str) -> Result<Self> {
        let raw = raw.trim();
        if raw.is_empty() {
            bail!("empty tool spec");
        }

        // Reuse the canonical runtime request parser for consistency.
        let request = RuntimeRequest::parse(raw);

        if request.name.is_empty() {
            bail!("invalid tool spec '{}': missing name", raw);
        }

        Ok(Self {
            name: request.name,
            version: request.version.unwrap_or_else(|| "latest".to_string()),
        })
    }
}

/// Options for the `add` command.
#[derive(Debug, Clone, Default)]
pub struct AddOptions {
    pub no_install: bool,
    pub no_lock: bool,
    pub frozen: bool,
    pub dry_run: bool,
    pub force: bool,
    /// Platform restriction (writes a detailed `[tools.<name>]` entry).
    pub os: Vec<String>,
    pub verbose: bool,
}

/// Handle `vx add <specs...>`.
pub async fn handle(
    registry: &ProviderRegistry,
    specs: &[String],
    options: AddOptions,
) -> Result<()> {
    if specs.is_empty() {
        bail!("no tools specified; usage: vx add <tool>[@version]...");
    }

    // 1. Parse specs
    let parsed = parse_specs(specs)?;

    // 2. Validate names against registry
    validate_tool_names(registry, &parsed)?;

    // 3. Locate vx.toml
    let current_dir = std::env::current_dir().context("failed to get current dir")?;
    let config_path = find_vx_config(&current_dir)
        .map_err(|e| anyhow::anyhow!("{}\nTip: run `vx init` to create vx.toml", e))?;
    let project_root = config_path.parent().unwrap_or(&current_dir).to_path_buf();

    // 4. Read vx.toml into a format-preserving document
    let original = std::fs::read_to_string(&config_path)
        .with_context(|| format!("failed to read {}", config_path.display()))?;
    let mut doc: DocumentMut = original
        .parse()
        .with_context(|| format!("failed to parse {}", config_path.display()))?;

    // 5. Apply edits to the document
    let edits = apply_edits(&mut doc, &parsed, &options)?;

    if edits.is_empty() {
        UI::info("No changes to vx.toml (all tools already configured with matching versions).");
        if options.dry_run {
            return Ok(());
        }
    }

    // 6. Preview or write
    if options.dry_run {
        UI::section("Planned changes to vx.toml");
        for edit in &edits {
            println!("  {}", edit.describe());
        }
        println!("\n--- vx.toml (proposed) ---");
        println!("{}", doc);

        if !options.no_lock {
            UI::hint("With `vx lock` the following entries would be (re)resolved:");
            for edit in &edits {
                println!("  - {}@{}", edit.name, edit.new_version);
            }
        }

        return Ok(());
    }

    std::fs::write(&config_path, doc.to_string())
        .with_context(|| format!("failed to write {}", config_path.display()))?;

    if edits.is_empty() {
        UI::success("vx.toml is already up to date");
    } else {
        UI::success(&format!(
            "Updated {} ({} tool{})",
            config_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            edits.len(),
            if edits.len() == 1 { "" } else { "s" }
        ));
        for edit in &edits {
            UI::detail(&format!("  {}", edit.describe()));
        }
    }

    // 7. Update vx.lock (call `vx lock --tool <name>` for each added tool)
    if !options.no_lock && !edits.is_empty() {
        update_lockfile(&project_root, &edits, options.frozen, options.verbose)?;
    } else if options.frozen {
        // With --frozen, verify the lock file already has matching entries.
        verify_frozen_lock(&project_root, &edits)?;
    }

    // 8. Install the tools (default behavior)
    if !options.no_install && !edits.is_empty() {
        install_tools(&edits, options.verbose)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Spec parsing & validation
// ---------------------------------------------------------------------------

fn parse_specs(specs: &[String]) -> Result<Vec<AddRuntimeSpec>> {
    let mut parsed = Vec::with_capacity(specs.len());
    let mut seen: HashSet<String> = HashSet::new();

    for raw in specs {
        let spec = AddRuntimeSpec::parse(raw)?;
        if !seen.insert(spec.name.clone()) {
            bail!("duplicate tool in command: '{}'", spec.name);
        }
        parsed.push(spec);
    }

    Ok(parsed)
}

fn validate_tool_names(registry: &ProviderRegistry, specs: &[AddRuntimeSpec]) -> Result<()> {
    let mut unknown = Vec::new();
    for spec in specs {
        if registry.get_provider(&spec.name).is_none() {
            unknown.push(spec.name.as_str());
        }
    }

    if unknown.is_empty() {
        return Ok(());
    }

    let available = registry.runtime_names();
    let suggestions = suggest_similar(&unknown, &available);

    let mut msg = format!(
        "unknown tool{}: {}",
        if unknown.len() == 1 { "" } else { "s" },
        unknown.join(", ")
    );
    if !suggestions.is_empty() {
        msg.push_str("\n\nDid you mean:");
        for (bad, good) in &suggestions {
            msg.push_str(&format!("\n  {} → {}", bad, good));
        }
    }
    msg.push_str("\n\nRun `vx list --available` to see all providers.");
    bail!(msg);
}

fn suggest_similar(unknowns: &[&str], available: &[String]) -> Vec<(String, String)> {
    let mut suggestions = Vec::new();
    for &bad in unknowns {
        // Prefer Levenshtein-style distance via `strsim` (already a dep).
        let best = available
            .iter()
            .map(|a| (a, strsim::jaro_winkler(bad, a)))
            .filter(|(_, score)| *score >= 0.8)
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((candidate, _)) = best {
            suggestions.push((bad.to_string(), candidate.clone()));
        }
    }
    suggestions
}

// ---------------------------------------------------------------------------
// TOML edits
// ---------------------------------------------------------------------------

/// Description of a single edit applied to vx.toml.
#[derive(Debug, Clone)]
pub struct Edit {
    pub name: String,
    pub new_version: String,
    pub kind: EditKind,
}

#[derive(Debug, Clone)]
pub enum EditKind {
    Added,
    Updated { old_version: String },
}

impl Edit {
    fn describe(&self) -> String {
        match &self.kind {
            EditKind::Added => format!("+ {}@{}", self.name, self.new_version),
            EditKind::Updated { old_version } => {
                format!("~ {}@{} (was {})", self.name, self.new_version, old_version)
            }
        }
    }
}

/// Apply edits to a parsed vx.toml document (exposed for testing).
pub fn apply_edits(
    doc: &mut DocumentMut,
    specs: &[AddRuntimeSpec],
    options: &AddOptions,
) -> Result<Vec<Edit>> {
    // Ensure [tools] table exists.
    if doc.get("tools").and_then(|i| i.as_table()).is_none() {
        let mut table = Table::new();
        table.set_implicit(false);
        doc["tools"] = Item::Table(table);
    }

    let tools = doc["tools"]
        .as_table_mut()
        .context("vx.toml `tools` section is not a table")?;

    let mut edits = Vec::new();

    for spec in specs {
        let existing = read_existing_version(tools, &spec.name);

        let has_os = !options.os.is_empty();

        if let Some(old) = existing.clone() {
            if !options.force && old == spec.version && !has_os {
                // No change, skip.
                continue;
            }

            write_tool_entry(tools, spec, &options.os)?;
            edits.push(Edit {
                name: spec.name.clone(),
                new_version: spec.version.clone(),
                kind: EditKind::Updated { old_version: old },
            });
        } else {
            write_tool_entry(tools, spec, &options.os)?;
            edits.push(Edit {
                name: spec.name.clone(),
                new_version: spec.version.clone(),
                kind: EditKind::Added,
            });
        }
    }

    // Keep `[tools]` entries sorted for deterministic output, but only touch
    // the simple-string entries so we don't disturb detailed subtables.
    sort_simple_tools(tools);

    Ok(edits)
}

fn read_existing_version(tools: &Table, name: &str) -> Option<String> {
    match tools.get(name)? {
        Item::Value(Value::String(s)) => Some(s.value().to_string()),
        Item::Table(t) => t
            .get("version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        Item::Value(Value::InlineTable(t)) => t
            .get("version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        _ => None,
    }
}

fn write_tool_entry(tools: &mut Table, spec: &AddRuntimeSpec, os: &[String]) -> Result<()> {
    if os.is_empty() {
        // If an existing detailed entry is present, preserve its extra fields,
        // only updating `version`.
        if let Some(Item::Table(existing)) = tools.get_mut(&spec.name) {
            existing.insert("version", Item::Value(Value::from(spec.version.as_str())));
            return Ok(());
        }

        // Simple string form: `name = "version"`.
        tools.insert(&spec.name, Item::Value(Value::from(spec.version.as_str())));
    } else {
        // Detailed form: `[tools.<name>] version = "..."  os = [...]`.
        let mut table = match tools.remove(&spec.name) {
            Some(Item::Table(t)) => t,
            _ => Table::new(),
        };

        table.insert("version", Item::Value(Value::from(spec.version.as_str())));

        let mut arr = Array::new();
        for o in os {
            arr.push(o.as_str());
        }
        table.insert("os", Item::Value(Value::Array(arr)));

        tools.insert(&spec.name, Item::Table(table));
    }

    Ok(())
}

/// Sort the simple `name = "version"` entries alphabetically, leaving
/// detailed sub-tables in their original position.
fn sort_simple_tools(tools: &mut Table) {
    // toml_edit doesn't expose a stable sort_values across all types, but we
    // can use `sort_values_by` which is stable for values inside the table.
    tools.sort_values_by(|a_key, _a_val, b_key, _b_val| a_key.get().cmp(b_key.get()));
}

// ---------------------------------------------------------------------------
// Lockfile / install integration
// ---------------------------------------------------------------------------

fn update_lockfile(project_root: &Path, edits: &[Edit], frozen: bool, verbose: bool) -> Result<()> {
    let lock_path = project_root.join(LOCK_FILE_NAME);

    if frozen {
        verify_frozen_lock(project_root, edits)?;
        return Ok(());
    }

    // Shell out to `vx lock --tool <name>` per edit. This reuses all the
    // resolver logic in `commands::lock` without duplicating it here.
    for edit in edits {
        let exe = std::env::current_exe().context("failed to get current exe")?;
        let mut cmd = Command::new(exe);
        cmd.arg("lock").arg("--tool").arg(&edit.name);
        if verbose {
            cmd.arg("--verbose");
        }
        cmd.current_dir(project_root);

        let status = cmd
            .status()
            .with_context(|| format!("failed to run `vx lock --tool {}`", edit.name))?;

        if !status.success() {
            bail!(
                "`vx lock --tool {}` failed with exit code {:?}",
                edit.name,
                status.code()
            );
        }
    }

    if lock_path.exists() {
        UI::success(&format!("Updated {}", LOCK_FILE_NAME));
    }

    Ok(())
}

fn verify_frozen_lock(project_root: &Path, edits: &[Edit]) -> Result<()> {
    let lock_path = project_root.join(LOCK_FILE_NAME);
    if !lock_path.exists() {
        bail!(
            "--frozen was requested but {} does not exist",
            LOCK_FILE_NAME
        );
    }

    let lock = LockFile::load(&lock_path)
        .with_context(|| format!("failed to load {}", lock_path.display()))?;

    let mut missing = Vec::new();
    for edit in edits {
        if !lock.is_locked(&edit.name) {
            missing.push(edit.name.as_str());
        }
    }

    if !missing.is_empty() {
        bail!(
            "--frozen: the following tool{} are not in {}: {}\nHint: rerun without --frozen, or run `vx lock` first.",
            if missing.len() == 1 { "" } else { "s" },
            LOCK_FILE_NAME,
            missing.join(", ")
        );
    }

    Ok(())
}

fn install_tools(edits: &[Edit], verbose: bool) -> Result<()> {
    let exe = std::env::current_exe().context("failed to get current exe")?;

    let mut args: Vec<String> = vec!["install".into()];
    for edit in edits {
        args.push(format!("{}@{}", edit.name, edit.new_version));
    }

    let mut cmd = Command::new(&exe);
    cmd.args(&args);
    if verbose {
        // Nothing to pass through here; `install` inherits stderr/stdout.
    }

    let status = cmd.status().context("failed to run `vx install`")?;

    if !status.success() {
        bail!("`vx install` failed with exit code {:?}", status.code());
    }

    Ok(())
}
