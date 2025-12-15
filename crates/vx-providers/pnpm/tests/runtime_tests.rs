//! PNPM runtime tests

use rstest::rstest;
use vx_provider_pnpm::{PnpmProvider, PnpmRuntime};
use vx_runtime::{Ecosystem, Provider, Runtime};

#[rstest]
fn test_pnpm_runtime_name() {
    let runtime = PnpmRuntime::new();
    assert_eq!(runtime.name(), "pnpm");
}

#[rstest]
fn test_pnpm_runtime_ecosystem() {
    let runtime = PnpmRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::NodeJs);
}

#[rstest]
fn test_pnpm_runtime_description() {
    let runtime = PnpmRuntime::new();
    assert_eq!(
        runtime.description(),
        "Fast, disk space efficient package manager"
    );
}

#[rstest]
fn test_pnpm_provider_name() {
    let provider = PnpmProvider::new();
    assert_eq!(provider.name(), "pnpm");
}

#[rstest]
fn test_pnpm_provider_runtimes() {
    let provider = PnpmProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "pnpm");
}

#[rstest]
fn test_pnpm_provider_supports() {
    let provider = PnpmProvider::new();
    assert!(provider.supports("pnpm"));
    assert!(!provider.supports("npm"));
}

#[rstest]
fn test_pnpm_provider_get_runtime() {
    let provider = PnpmProvider::new();

    let pnpm = provider.get_runtime("pnpm");
    assert!(pnpm.is_some());
    assert_eq!(pnpm.unwrap().name(), "pnpm");

    let unknown = provider.get_runtime("unknown");
    assert!(unknown.is_none());
}
