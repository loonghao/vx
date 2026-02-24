//! Batch lint test for all provider.star files in the workspace.
//!
//! Run with:
//!   cargo test -p vx-starlark --test lint_all_providers_test -- --nocapture
//!
//! This test:
//! 1. Discovers all provider.star files under crates/vx-providers/
//! 2. Parses and lints each one with AstModuleLint
//! 3. Prints a summary of issues found
//! 4. FAILS if any provider.star has lint issues

use starlark::analysis::AstModuleLint;
use starlark::syntax::{AstModule, Dialect};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Globals that are always available in provider.star scripts.
/// These come from the vx runtime or are standard Starlark builtins.
fn known_globals() -> HashSet<String> {
    [
        // Standard Starlark builtins
        "True",
        "False",
        "None",
        "len",
        "range",
        "print",
        "str",
        "int",
        "float",
        "bool",
        "list",
        "dict",
        "tuple",
        "set",
        "type",
        "repr",
        "hash",
        "dir",
        "hasattr",
        "getattr",
        "setattr",
        "enumerate",
        "zip",
        "map",
        "filter",
        "sorted",
        "reversed",
        "min",
        "max",
        "sum",
        "any",
        "all",
        "abs",
        "round",
        "divmod",
        "pow",
        "hex",
        "oct",
        "chr",
        "ord",
        "bytes",
        "bytearray",
        "struct",
        "fail",
        "assert_",
        // vx stdlib symbols (from @vx//stdlib:http.star)
        "fetch_json_versions",
        "fetch_github_versions",
        // vx stdlib symbols (from @vx//stdlib:install.star)
        "set_permissions",
        "ensure_dependencies",
        // Common provider-level exports
        "name",
        "description",
        "homepage",
        "repository",
        "license",
        "ecosystem",
        "runtimes",
        "permissions",
        "requires",
        // Common function names in provider.star
        "fetch_versions",
        "download_url",
        "install_layout",
        "environment",
        "post_install",
        "pre_run",
        "uninstall",
        "ctx",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// Find all provider.star files under the given root directory.
fn find_provider_stars(root: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                results.extend(find_provider_stars(&path));
            } else if path
                .file_name()
                .map(|n| n == "provider.star")
                .unwrap_or(false)
            {
                results.push(path);
            }
        }
    }
    results.sort();
    results
}

#[test]
fn lint_all_provider_stars() {
    // Locate the workspace root relative to this test file's manifest dir
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let providers_dir = manifest_dir
        .parent() // crates/
        .unwrap()
        .join("vx-providers");

    assert!(
        providers_dir.exists(),
        "vx-providers directory not found at: {}",
        providers_dir.display()
    );

    let stars = find_provider_stars(&providers_dir);
    assert!(!stars.is_empty(), "No provider.star files found");

    println!("\n=== Linting {} provider.star files ===\n", stars.len());

    let globals = known_globals();
    let mut total_issues = 0usize;
    let mut files_with_issues = Vec::new();

    for star_path in &stars {
        let content = match std::fs::read_to_string(star_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("  ERROR reading {}: {}", star_path.display(), e);
                continue;
            }
        };

        let provider_name = star_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let ast = match AstModule::parse("provider.star", content, &Dialect::Standard) {
            Ok(a) => a,
            Err(e) => {
                println!("  [PARSE ERROR] {}: {}", provider_name, e);
                total_issues += 1;
                files_with_issues.push(format!("{} (parse error)", provider_name));
                continue;
            }
        };

        let lints = ast.lint(Some(&globals));

        if lints.is_empty() {
            println!("  ✓  {}", provider_name);
        } else {
            println!("  ✗  {} ({} issue(s)):", provider_name, lints.len());
            for lint in &lints {
                println!(
                    "       [{:20}] {} at {}",
                    lint.short_name, lint.problem, lint.location
                );
            }
            total_issues += lints.len();
            files_with_issues.push(provider_name.to_string());
        }
    }

    println!(
        "\n=== Summary: {}/{} files clean, {} total issue(s) ===\n",
        stars.len() - files_with_issues.len(),
        stars.len(),
        total_issues
    );

    assert!(
        total_issues == 0,
        "{} provider.star file(s) have lint issues: {}",
        files_with_issues.len(),
        files_with_issues.join(", ")
    );
}
