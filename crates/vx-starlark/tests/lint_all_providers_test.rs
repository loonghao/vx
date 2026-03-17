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
use std::path::{Path, PathBuf};
use vx_starlark::provider_test_support::provider_lint_known_globals;

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

    let globals = provider_lint_known_globals();
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
