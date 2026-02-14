//! Rust script parsing
//!
//! Parses scripts from Cargo.toml (cargo-make, etc.).
//!
//! Note: Justfile parsing has been moved to the common module
//! since `just` is a language-agnostic tool.

use crate::error::AnalyzerResult;
use crate::script_parser::ScriptParser;
use crate::types::{Script, ScriptSource};

/// Parse scripts from Cargo.toml [package.metadata.scripts] or cargo-make
pub fn parse_cargo_scripts(content: &str, parser: &ScriptParser) -> AnalyzerResult<Vec<Script>> {
    let mut scripts = Vec::new();
    let doc: toml::Value = toml::from_str(content)?;

    // Check for [package.metadata.scripts] (custom scripts section)
    if let Some(package) = doc.get("package")
        && let Some(metadata) = package.get("metadata")
        && let Some(scripts_table) = metadata.get("scripts")
        && let Some(table) = scripts_table.as_table()
    {
        for (name, value) in table {
            if let Some(cmd) = value.as_str() {
                let mut script = Script::new(name, cmd, ScriptSource::CargoToml);
                script.tools = parser.parse(cmd);
                scripts.push(script);
            }
        }
    }

    Ok(scripts)
}
