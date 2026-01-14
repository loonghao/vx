//! Rust runtime tests

use rstest::rstest;
use vx_provider_rust::{CargoRuntime, RustProvider, RustUrlBuilder, RustcRuntime};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

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
fn test_rustc_runtime_aliases() {
    let runtime = RustcRuntime::new();
    assert!(runtime.aliases().contains(&"rust"));
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
    assert_eq!(runtimes.len(), 2);

    let names: Vec<&str> = runtimes.iter().map(|r| r.name()).collect();
    assert!(names.contains(&"cargo"));
    assert!(names.contains(&"rustc"));
}

#[rstest]
fn test_rust_provider_supports() {
    let provider = RustProvider::new();
    assert!(provider.supports("cargo"));
    assert!(provider.supports("rustc"));
    assert!(provider.supports("rust")); // alias for rustc
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

    // "rust" is an alias for "rustc"
    let rust = provider.get_runtime("rust");
    assert!(rust.is_some());
    assert_eq!(rust.unwrap().name(), "rustc");

    let unknown = provider.get_runtime("unknown");
    assert!(unknown.is_none());
}

// ============================================================================
// URL Builder Tests
// ============================================================================

#[rstest]
fn test_rust_url_builder_download_url_format() {
    let platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };
    // Test that download URL uses tar.gz format (not msi)
    let url = RustUrlBuilder::download_url("1.75.0", &platform);
    assert!(url.is_some());
    let url = url.unwrap();

    // Should use tar.gz format for all platforms
    assert!(
        url.ends_with(".tar.gz"),
        "URL should end with .tar.gz, got: {}",
        url
    );
    assert!(
        !url.ends_with(".msi"),
        "URL should not use .msi format, got: {}",
        url
    );
    assert!(url.contains("static.rust-lang.org/dist/rust-1.75.0-"));
}

#[rstest]
fn test_rust_url_builder_platform_string() {
    let platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };
    let platform_str = RustUrlBuilder::get_platform_string(&platform);

    // Platform string should be non-empty and contain expected patterns
    assert!(!platform_str.is_empty());
    assert_eq!(platform_str, "x86_64-unknown-linux-gnu");
}

#[rstest]
fn test_rust_url_builder_windows() {
    let platform = Platform {
        os: Os::Windows,
        arch: Arch::X86_64,
    };
    let url = RustUrlBuilder::download_url("1.75.0", &platform);
    assert!(url.is_some());
    let url = url.unwrap();

    // Windows should also use tar.gz (not msi)
    assert!(url.ends_with(".tar.gz"));
    assert!(url.contains("x86_64-pc-windows-msvc"));
}

#[rstest]
fn test_rust_url_builder_macos_arm64() {
    let platform = Platform {
        os: Os::MacOS,
        arch: Arch::Aarch64,
    };
    let url = RustUrlBuilder::download_url("1.75.0", &platform);
    assert!(url.is_some());
    let url = url.unwrap();

    assert!(url.ends_with(".tar.gz"));
    assert!(url.contains("aarch64-apple-darwin"));
}
