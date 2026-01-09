//! Bun runtime tests

use rstest::rstest;
use vx_provider_bun::{BunProvider, BunRuntime};
use vx_runtime::{Ecosystem, Provider, Runtime};

#[rstest]
fn test_bun_runtime_name() {
    let runtime = BunRuntime::new();
    assert_eq!(runtime.name(), "bun");
}

#[rstest]
fn test_bun_runtime_ecosystem() {
    let runtime = BunRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::NodeJs);
}

#[rstest]
fn test_bun_runtime_description() {
    let runtime = BunRuntime::new();
    assert!(runtime.description().contains("JavaScript runtime"));
}

#[rstest]
fn test_bun_runtime_aliases() {
    let runtime = BunRuntime::new();
    // bunx is now handled as a separate RuntimeSpec, not an alias
    assert_eq!(runtime.aliases().len(), 0);
}

#[rstest]
fn test_bun_provider_name() {
    let provider = BunProvider::new();
    assert_eq!(provider.name(), "bun");
}

#[rstest]
fn test_bun_provider_runtimes() {
    let provider = BunProvider::new();
    let runtimes = provider.runtimes();
    // bun and bunx are both defined in provider.toml
    assert_eq!(runtimes.len(), 2);

    let names: Vec<&str> = runtimes.iter().map(|r| r.name()).collect();
    assert!(names.contains(&"bun"));
    assert!(names.contains(&"bunx"));
}

#[rstest]
fn test_bun_provider_supports() {
    let provider = BunProvider::new();
    assert!(provider.supports("bun"));
    // bunx should be supported through alias resolution in the resolver layer
    assert!(!provider.supports("npm"));
}
