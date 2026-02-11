//! .NET/C# dependency parsing
//!
//! Parses dependencies from .csproj files (PackageReference elements)

use crate::dependency::{Dependency, DependencySource};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use regex::Regex;
use std::path::Path;

/// Parse dependencies from .csproj file content
pub fn parse_csproj_dependencies(content: &str, path: &Path) -> AnalyzerResult<Vec<Dependency>> {
    let mut deps = Vec::new();

    // Match <PackageReference Include="Name" Version="Version" />
    // Also handles multi-line format:
    // <PackageReference Include="Name">
    //   <Version>1.0.0</Version>
    // </PackageReference>
    let inline_re =
        Regex::new(r#"<PackageReference\s+Include="([^"]+)"\s+Version="([^"]+)""#).unwrap();

    for cap in inline_re.captures_iter(content) {
        let name = cap[1].to_string();
        let version = cap[2].to_string();

        let mut dep = Dependency::new(
            name,
            Ecosystem::DotNet,
            DependencySource::ConfigFile {
                path: path.to_path_buf(),
                section: "PackageReference".to_string(),
            },
        );
        dep = dep.with_version(version);
        deps.push(dep);
    }

    // Also match Include-only references (version might be managed centrally)
    let include_only_re = Regex::new(r#"<PackageReference\s+Include="([^"]+)"\s*/>"#).unwrap();

    for cap in include_only_re.captures_iter(content) {
        let name = cap[1].to_string();
        // Skip if already captured with version
        if !deps.iter().any(|d| d.name == name) {
            deps.push(Dependency::new(
                name,
                Ecosystem::DotNet,
                DependencySource::ConfigFile {
                    path: path.to_path_buf(),
                    section: "PackageReference".to_string(),
                },
            ));
        }
    }

    Ok(deps)
}

/// Parse dependencies from Directory.Packages.props (central package management)
pub fn parse_directory_packages_props(
    content: &str,
    path: &Path,
) -> AnalyzerResult<Vec<Dependency>> {
    let mut deps = Vec::new();

    let re = Regex::new(r#"<PackageVersion\s+Include="([^"]+)"\s+Version="([^"]+)""#).unwrap();

    for cap in re.captures_iter(content) {
        let name = cap[1].to_string();
        let version = cap[2].to_string();

        let mut dep = Dependency::new(
            name,
            Ecosystem::DotNet,
            DependencySource::ConfigFile {
                path: path.to_path_buf(),
                section: "PackageVersion".to_string(),
            },
        );
        dep = dep.with_version(version);
        deps.push(dep);
    }

    Ok(deps)
}
