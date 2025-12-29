//! Node.js script parsing
//!
//! Parses scripts from package.json files.

use super::dependencies::PackageJson;
use super::package_manager::PackageManager;
use crate::error::AnalyzerResult;
use crate::script_parser::ScriptParser;
use crate::types::{Script, ScriptSource};

/// Parse scripts from package.json content
pub fn parse_package_json_scripts(
    content: &str,
    pm: PackageManager,
    parser: &ScriptParser,
) -> AnalyzerResult<Vec<Script>> {
    let mut scripts = Vec::new();
    let pkg: PackageJson = serde_json::from_str(content)?;

    let pkg_scripts = pkg.scripts.unwrap_or_default();

    // Collect all script names first for context-aware parsing
    let script_names: Vec<&str> = pkg_scripts.keys().map(|k| k.as_str()).collect();

    for (name, cmd) in &pkg_scripts {
        // Convert npm script to full command
        let full_cmd = format!("{} {}", pm.run_prefix(), name);
        let mut script = Script::new(name, full_cmd, ScriptSource::PackageJson);
        // Use context-aware parsing to filter out internal script references
        script.tools = parser.parse_with_context(cmd, &script_names);
        scripts.push(script);
    }

    Ok(scripts)
}
