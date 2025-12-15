//! Bun runtime tests

use rstest::rstest;
use vx_provider_bun::{BunProvider, BunRuntime, BunxRuntime};
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
fn test_bunx_runtime_name() {
    let runtime = BunxRuntime::new();
    assert_eq!(runtime.name(), "bunx");
}

#[rstest]
fn test_bunx_runtime_description() {
    let runtime = BunxRuntime::new();
    assert!(runtime.description().contains("package runner"));
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
    assert_eq!(runtimes.len(), 2);

    let names: Vec<&str> = runtimes.iter().map(|r| r.name()).collect();
    assert!(names.contains(&"bun"));
    assert!(names.contains(&"bunx"));
}

#[rstest]
fn test_bun_provider_supports() {
    let provider = BunProvider::new();
    assert!(provider.supports("bun"));
    assert!(provider.supports("bunx"));
    assert!(!provider.supports("npm"));
}

#[rstest]
fn test_bun_provider_get_runtime() {
    let provider = BunProvider::new();

    let bun = provider.get_runtime("bun");
    assert!(bun.is_some());
    assert_eq!(bun.unwrap().name(), "bun");

    let bunx = provider.get_runtime("bunx");
    assert!(bunx.is_some());
    assert_eq!(bunx.unwrap().name(), "bunx");

    let unknown = provider.get_runtime("unknown");
    assert!(unknown.is_none());
}
