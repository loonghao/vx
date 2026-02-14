//! Go dependency parsing from go.mod

use crate::dependency::{Dependency, DependencySource};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use std::path::Path;

/// Parse dependencies from go.mod content
pub fn parse_go_mod_dependencies(content: &str, _path: &Path) -> AnalyzerResult<Vec<Dependency>> {
    let mut deps = Vec::new();
    let mut in_require_block = false;

    for line in content.lines() {
        let line = line.trim();

        // Skip comments
        if line.starts_with("//") {
            continue;
        }

        // Check for require block start
        if line.starts_with("require (") || line == "require (" {
            in_require_block = true;
            continue;
        }

        // Check for block end
        if line == ")" {
            in_require_block = false;
            continue;
        }

        // Parse single-line require
        if line.starts_with("require ") && !line.contains("(") {
            if let Some(dep) = parse_require_line(&line[8..]) {
                deps.push(dep);
            }
            continue;
        }

        // Parse require block entries
        if in_require_block && let Some(dep) = parse_require_line(line) {
            deps.push(dep);
        }
    }

    Ok(deps)
}

fn parse_require_line(line: &str) -> Option<Dependency> {
    let line = line.trim();

    // Skip empty lines and indirect dependencies
    if line.is_empty() || line.contains("// indirect") {
        return None;
    }

    // Format: module/path version [// comment]
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        let name = parts[0].to_string();
        let version = parts[1].to_string();

        return Some(Dependency {
            name,
            version: Some(version),
            ecosystem: Ecosystem::Go,
            source: DependencySource::GoMod,
            is_dev: false,
            is_installed: false,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_go_mod() {
        let content = r#"
module example.com/myapp

go 1.21

require (
    github.com/spf13/cobra v1.8.0
    github.com/stretchr/testify v1.8.4
    golang.org/x/sys v0.15.0 // indirect
)

require github.com/single/dep v1.0.0
"#;

        let deps = parse_go_mod_dependencies(content, Path::new("go.mod")).unwrap();

        assert_eq!(deps.len(), 3);
        assert_eq!(deps[0].name, "github.com/spf13/cobra");
        assert_eq!(deps[0].version, Some("v1.8.0".to_string()));
        assert_eq!(deps[1].name, "github.com/stretchr/testify");
        assert_eq!(deps[2].name, "github.com/single/dep");
    }
}
