//! Shared test support for provider.star unit and lint tests.
//!
//! This module centralizes the Starlark globals and assertion helpers used by
//! provider test suites so that all providers share a single source of truth.

use starlark::analysis::AstModuleLint;
use starlark::syntax::{AstModule, Dialect};
use std::collections::HashSet;

/// Shared globals available to provider.star tests and lints.
///
/// This intentionally uses a superset of builtins and vx stdlib symbols to keep
/// provider tests aligned with the batch lint check in `vx-starlark`.
pub fn provider_lint_known_globals() -> HashSet<String> {
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
        // vx stdlib symbols used by provider.star tests
        "fetch_json_versions",
        "fetch_github_versions",
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
        "package_alias",
        "package_prefixes",
        "vx_version",
        // Common function names in provider.star
        "fetch_versions",
        "download_url",
        "install_layout",
        "environment",
        "store_root",
        "get_execute_path",
        "post_install",
        "post_extract",
        "pre_run",
        "deps",
        "system_install",
        "script_install",
        "supported_platforms",
        "uninstall",
        "ctx",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// Assert that a provider.star source parses and lints cleanly.
pub fn assert_provider_star_lint_clean(provider_star: &str) {
    let ast = AstModule::parse(
        "provider.star",
        provider_star.to_string(),
        &Dialect::Standard,
    )
    .expect("provider.star should parse without errors");

    let globals = provider_lint_known_globals();
    let lints = ast.lint(Some(&globals));

    assert!(
        lints.is_empty(),
        "provider.star has lint issues:\n{}",
        lints
            .iter()
            .map(|l| format!("  [{}] {} at {}", l.short_name, l.problem, l.location))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
