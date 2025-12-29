//! C++ dependency parsing
//!
//! Parses dependencies from CMakeLists.txt

use crate::dependency::{Dependency, DependencySource};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use regex::Regex;
use std::path::Path;

/// Parse dependencies from CMakeLists.txt
pub fn parse_cmake_dependencies(content: &str, path: &Path) -> AnalyzerResult<Vec<Dependency>> {
    let mut deps = Vec::new();

    // Match find_package(PackageName ...)
    let find_package_re = Regex::new(r"find_package\s*\(\s*(\w+)").unwrap();
    for cap in find_package_re.captures_iter(content) {
        if let Some(name) = cap.get(1) {
            let name = name.as_str();
            // Skip common CMake modules that aren't real dependencies
            if !is_cmake_builtin(name) {
                let mut dep = Dependency::new(
                    name.to_string(),
                    Ecosystem::Cpp,
                    DependencySource::ConfigFile {
                        path: path.to_path_buf(),
                        section: "find_package".to_string(),
                    },
                );
                dep.is_installed = true; // Assume found packages are installed
                deps.push(dep);
            }
        }
    }

    // Match FetchContent_Declare(name ...)
    let fetch_content_re = Regex::new(r"FetchContent_Declare\s*\(\s*(\w+)").unwrap();
    for cap in fetch_content_re.captures_iter(content) {
        if let Some(name) = cap.get(1) {
            let dep = Dependency::new(
                name.as_str().to_string(),
                Ecosystem::Cpp,
                DependencySource::ConfigFile {
                    path: path.to_path_buf(),
                    section: "FetchContent".to_string(),
                },
            );
            deps.push(dep);
        }
    }

    // Match target_link_libraries(target ... lib1 lib2)
    // This is less reliable but can catch some dependencies
    let link_libs_re = Regex::new(r"target_link_libraries\s*\([^)]+\)").unwrap();
    for mat in link_libs_re.find_iter(content) {
        let text = mat.as_str();
        // Extract library names (skip first token which is target name)
        let tokens: Vec<&str> = text
            .trim_start_matches("target_link_libraries")
            .trim()
            .trim_start_matches('(')
            .trim_end_matches(')')
            .split_whitespace()
            .skip(1) // Skip target name
            .filter(|s| {
                !s.starts_with('$')
                    && !s.starts_with("PRIVATE")
                    && !s.starts_with("PUBLIC")
                    && !s.starts_with("INTERFACE")
            })
            .collect();

        for lib in tokens {
            // Skip internal targets (usually start with project name or common prefixes)
            if !lib.contains("::") && !is_internal_target(lib) {
                let dep = Dependency::new(
                    lib.to_string(),
                    Ecosystem::Cpp,
                    DependencySource::ConfigFile {
                        path: path.to_path_buf(),
                        section: "target_link_libraries".to_string(),
                    },
                );
                // Don't add duplicates
                if !deps.iter().any(|d| d.name == lib) {
                    deps.push(dep);
                }
            }
        }
    }

    Ok(deps)
}

/// Check if a package name is a CMake builtin module
fn is_cmake_builtin(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "threads"
            | "openmp"
            | "mpi"
            | "cuda"
            | "cudatoolkit"
            | "python"
            | "python3"
            | "perl"
            | "java"
            | "jni"
            | "git"
            | "subversion"
            | "cvs"
            | "doxygen"
            | "latex"
            | "gnuplot"
            | "hdf5"
            | "blas"
            | "lapack"
            | "pkgconfig"
            | "pkg-config"
    )
}

/// Check if a target name looks like an internal target
fn is_internal_target(name: &str) -> bool {
    // Common internal target patterns
    name.starts_with("${")
        || name.starts_with("mrv")
        || name.starts_with("tl")
        || name.starts_with("lib")
        || name.ends_with("_lib")
        || name.ends_with("_static")
        || name.ends_with("_shared")
}
