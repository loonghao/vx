//! Rust runtime tests

use rstest::rstest;
use vx_provider_rust::{CargoRuntime, RustProvider, RustcRuntime, RustupRuntime};
use vx_runtime::{Ecosystem, Provider, Runtime};

#[rstest]
fn test_cargo_runtime_name() {
    let runtime = CargoRuntime::new();
    assert_eq!(runtime.name(), "cargo");
}

#[rstest]
fn test_cargo_runtime_ecosystem() {
    let runtime = CargoRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::Rust);
}

#[rstest]
fn test_cargo_runtime_description() {
    let runtime = CargoRuntime::new();
    assert!(runtime.description().contains("package manager"));
}

#[rstest]
fn test_rustc_runtime_name() {
    let runtime = RustcRuntime::new();
    assert_eq!(runtime.name(), "rustc");
}

#[rstest]
fn test_rustup_runtime_name() {
    let runtime = RustupRuntime::new();
    assert_eq!(runtime.name(), "rustup");
}

#[rstest]
fn test_rust_provider_name() {
    let provider = RustProvider::new();
    assert_eq!(provider.name(), "rust");
}

#[rstest]
fn test_rust_provider_runtimes() {
    let provider = RustProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 3);

    let names: Vec<&str> = runtimes.iter().map(|r| r.name()).collect();
    assert!(names.contains(&"cargo"));
    assert!(names.contains(&"rustc"));
    assert!(names.contains(&"rustup"));
}

#[rstest]
fn test_rust_provider_supports() {
    let provider = RustProvider::new();
    assert!(provider.supports("cargo"));
    assert!(provider.supports("rustc"));
    assert!(provider.supports("rustup"));
    assert!(!provider.supports("go"));
}

#[rstest]
fn test_rust_provider_get_runtime() {
    let provider = RustProvider::new();

    let cargo = provider.get_runtime("cargo");
    assert!(cargo.is_some());
    assert_eq!(cargo.unwrap().name(), "cargo");

    let rustc = provider.get_runtime("rustc");
    assert!(rustc.is_some());
    assert_eq!(rustc.unwrap().name(), "rustc");

    let rustup = provider.get_runtime("rustup");
    assert!(rustup.is_some());
    assert_eq!(rustup.unwrap().name(), "rustup");

    let unknown = provider.get_runtime("unknown");
    assert!(unknown.is_none());
}
