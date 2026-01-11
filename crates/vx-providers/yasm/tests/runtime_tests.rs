//! Tests for YASM runtime

use vx_provider_yasm::{YasmRuntime, YasmUrlBuilder};
use vx_runtime::{Arch, Os, Platform, Runtime};

#[test]
fn test_runtime_name() {
    let runtime = YasmRuntime::new();
    assert_eq!(runtime.name(), "yasm");
}

#[test]
fn test_runtime_aliases() {
    let runtime = YasmRuntime::new();
    assert!(runtime.aliases().is_empty());
}

#[test]
fn test_download_url_windows_x64() {
    let platform = Platform {
        os: Os::Windows,
        arch: Arch::X86_64,
    };
    let url = YasmUrlBuilder::download_url("1.3.0", &platform);
    assert!(url.is_some());
    assert!(url.unwrap().contains("win64"));
}

#[test]
fn test_executable_relative_path_windows() {
    let runtime = YasmRuntime::new();
    let platform = Platform {
        os: Os::Windows,
        arch: Arch::X86_64,
    };
    let path = runtime.executable_relative_path("1.3.0", &platform);
    assert_eq!(path, "yasm.exe");
}

#[test]
fn test_supported_platforms() {
    let runtime = YasmRuntime::new();
    let platforms = runtime.supported_platforms();

    // Should support Windows x64 and Windows x86
    assert_eq!(platforms.len(), 2);

    // Should include Windows x64
    assert!(platforms
        .iter()
        .any(|p| { matches!(p.os, Os::Windows) && matches!(p.arch, Arch::X86_64) }));

    // Should include Windows x86
    assert!(platforms
        .iter()
        .any(|p| { matches!(p.os, Os::Windows) && matches!(p.arch, Arch::X86) }));
}
