//! .NET/C# dependency parsing
//!
//! Parses dependencies from .csproj files (PackageReference elements)

use crate::dependency::{Dependency, DependencySource};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

/// Regex for `<PackageReference Include="Name" Version="Version" />`
static PACKAGE_REF_INLINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<PackageReference\s+Include="([^"]+)"\s+Version="([^"]+)""#).unwrap()
});

/// Regex for `<PackageReference Include="Name" />` (version managed centrally)
static PACKAGE_REF_INCLUDE_ONLY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<PackageReference\s+Include="([^"]+)"\s*/>"#).unwrap());

/// Regex for `<PackageVersion Include="Name" Version="Version" />`
static PACKAGE_VERSION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<PackageVersion\s+Include="([^"]+)"\s+Version="([^"]+)""#).unwrap()
});

/// Parse dependencies from .csproj file content
pub fn parse_csproj_dependencies(content: &str, path: &Path) -> AnalyzerResult<Vec<Dependency>> {
    let mut deps = Vec::new();

    // Match <PackageReference Include="Name" Version="Version" />
    // Also handles multi-line format:
    // <PackageReference Include="Name">
    //   <Version>1.0.0</Version>
    // </PackageReference>
    for cap in PACKAGE_REF_INLINE_RE.captures_iter(content) {
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
    for cap in PACKAGE_REF_INCLUDE_ONLY_RE.captures_iter(content) {
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

    for cap in PACKAGE_VERSION_RE.captures_iter(content) {
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
