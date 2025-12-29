//! Python script parsing
//!
//! Parses explicit scripts from pyproject.toml

use crate::error::AnalyzerResult;
use crate::script_parser::ScriptParser;
use crate::types::{Script, ScriptSource};

/// Parse scripts from pyproject.toml
pub fn parse_pyproject_scripts(
    content: &str,
    parser: &ScriptParser,
) -> AnalyzerResult<Vec<Script>> {
    let mut scripts = Vec::new();

    let doc: toml::Value = toml::from_str(content)?;

    // Parse [tool.uv.scripts] (uv-specific scripts)
    if let Some(tool) = doc.get("tool") {
        if let Some(uv) = tool.get("uv") {
            if let Some(uv_scripts) = uv.get("scripts") {
                if let Some(scripts_table) = uv_scripts.as_table() {
                    for (name, cmd) in scripts_table {
                        if let Some(cmd_str) = cmd.as_str() {
                            let mut script = Script::new(
                                name.clone(),
                                cmd_str.to_string(),
                                ScriptSource::PyprojectToml {
                                    section: "tool.uv.scripts".to_string(),
                                },
                            );
                            script.tools = parser.parse(cmd_str);
                            scripts.push(script);
                        }
                    }
                }
            }
        }
    }

    // Parse [project.scripts] (entry points)
    if let Some(project) = doc.get("project") {
        if let Some(project_scripts) = project.get("scripts") {
            if let Some(scripts_table) = project_scripts.as_table() {
                for (name, entry_point) in scripts_table {
                    if let Some(ep_str) = entry_point.as_str() {
                        // Entry points are module:function format
                        let cmd =
                            format!("python -m {}", ep_str.split(':').next().unwrap_or(ep_str));
                        let mut script = Script::new(
                            name.clone(),
                            cmd.clone(),
                            ScriptSource::PyprojectToml {
                                section: "project.scripts".to_string(),
                            },
                        );
                        script.tools = parser.parse(&cmd);
                        scripts.push(script);
                    }
                }
            }
        }
    }

    Ok(scripts)
}
