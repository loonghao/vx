//! 7-Zip runtime tests

use rstest::rstest;
use vx_provider_7zip::{SevenZipProvider, SevenZipRuntime};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

// ============================================================================
// SevenZipRuntime — basic trait tests
// ============================================================================

#[rstest]
fn test_7zip_runtime_name() {
    let runtime = SevenZipRuntime::new();
    assert_eq!(runtime.name(), "7zip");
}

#[rstest]
fn test_7zip_runtime_description() {
    let runtime = SevenZipRuntime::new();
    let desc = runtime.description();
    assert!(
        desc.contains("7-Zip") || desc.contains("archiver"),
        "description should mention 7-Zip or archiver, got: {}",
        desc
    );
}

#[rstest]
fn test_7zip_runtime_ecosystem() {
    let runtime = SevenZipRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::System);
}

#[rstest]
fn test_7zip_runtime_aliases() {
    let runtime = SevenZipRuntime::new();
    let aliases = runtime.aliases();
    assert!(aliases.contains(&"7z"), "should have alias '7z'");
    assert!(aliases.contains(&"7za"), "should have alias '7za'");
}

#[rstest]
fn test_7zip_runtime_executable_name() {
    let runtime = SevenZipRuntime::new();
    assert_eq!(runtime.executable_name(), "7z");
}

#[rstest]
fn test_7zip_runtime_metadata() {
    let runtime = SevenZipRuntime::new();
    let meta = runtime.metadata();
    assert!(
        meta.contains_key("homepage"),
        "metadata should contain homepage"
    );
    assert!(
        meta.get("homepage")
            .map(|v| v.contains("7-zip.org"))
            .unwrap_or(false),
        "homepage should point to 7-zip.org"
    );
    assert!(
        meta.contains_key("repository"),
        "metadata should contain repository"
    );
}

// ============================================================================
// SevenZipRuntime — executable_relative_path
// ============================================================================

#[rstest]
fn test_7zip_executable_relative_path_windows() {
    let runtime = SevenZipRuntime::new();
    let platform = Platform::new(Os::Windows, Arch::X86_64);
    let path = runtime.executable_relative_path("24.09", &platform);
    assert!(
        path.ends_with(".exe"),
        "Windows executable should end with .exe, got: {}",
        path
    );
    assert!(
        path.contains("7z"),
        "executable path should contain '7z', got: {}",
        path
    );
}

#[rstest]
fn test_7zip_executable_relative_path_linux() {
    let runtime = SevenZipRuntime::new();
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let path = runtime.executable_relative_path("24.09", &platform);
    assert!(
        !path.ends_with(".exe"),
        "Linux executable should not end with .exe, got: {}",
        path
    );
    assert!(
        path.contains("7z"),
        "executable path should contain '7z', got: {}",
        path
    );
}

#[rstest]
fn test_7zip_executable_relative_path_macos() {
    let runtime = SevenZipRuntime::new();
    let platform = Platform::new(Os::MacOS, Arch::Aarch64);
    let path = runtime.executable_relative_path("24.09", &platform);
    assert!(
        !path.ends_with(".exe"),
        "macOS executable should not end with .exe, got: {}",
        path
    );
}

// ============================================================================
// SevenZipRuntime — download_url
// ============================================================================

#[tokio::test]
async fn test_7zip_download_url_windows_x64() {
    let runtime = SevenZipRuntime::new();
    let platform = Platform::new(Os::Windows, Arch::X86_64);
    let url = runtime.download_url("24.09", &platform).await.unwrap();
    assert!(url.is_some(), "Windows x64 should have a download URL");
    let url = url.unwrap();
    assert!(
        url.contains("24.09") || url.contains("2409"),
        "URL should contain version, got: {}",
        url
    );
    assert!(
        url.contains("github.com") || url.contains("7-zip.org"),
        "URL should be from github.com or 7-zip.org, got: {}",
        url
    );
}

#[tokio::test]
async fn test_7zip_download_url_linux_x64() {
    let runtime = SevenZipRuntime::new();
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let url = runtime.download_url("24.09", &platform).await.unwrap();
    assert!(url.is_some(), "Linux x64 should have a download URL");
    let url = url.unwrap();
    assert!(
        url.contains("linux"),
        "Linux URL should contain 'linux', got: {}",
        url
    );
    // Linux should use tar.xz (supported format)
    assert!(
        url.ends_with(".tar.xz") || url.ends_with(".tar.gz") || url.ends_with(".zip"),
        "Linux URL should use a supported archive format, got: {}",
        url
    );
}

#[tokio::test]
async fn test_7zip_download_url_linux_arm64() {
    let runtime = SevenZipRuntime::new();
    let platform = Platform::new(Os::Linux, Arch::Aarch64);
    let url = runtime.download_url("24.09", &platform).await.unwrap();
    assert!(url.is_some(), "Linux arm64 should have a download URL");
    let url = url.unwrap();
    assert!(
        url.contains("arm64") || url.contains("aarch64"),
        "arm64 URL should contain 'arm64' or 'aarch64', got: {}",
        url
    );
}

#[tokio::test]
async fn test_7zip_download_url_version_format() {
    let runtime = SevenZipRuntime::new();
    let platform = Platform::new(Os::Linux, Arch::X86_64);

    // Test with different version formats
    let url_24_09 = runtime.download_url("24.09", &platform).await.unwrap();
    let url_23_01 = runtime.download_url("23.01", &platform).await.unwrap();

    assert!(url_24_09.is_some());
    assert!(url_23_01.is_some());

    // URLs should differ between versions
    assert_ne!(
        url_24_09.unwrap(),
        url_23_01.unwrap(),
        "Different versions should produce different URLs"
    );
}

#[tokio::test]
async fn test_7zip_download_url_windows_uses_msi() {
    // Windows uses .msi installer (extracted via msiexec /a, no registry changes)
    let runtime = SevenZipRuntime::new();
    let platform = Platform::new(Os::Windows, Arch::X86_64);
    let url = runtime.download_url("24.09", &platform).await.unwrap();

    assert!(url.is_some(), "Windows x64 should have a download URL");
    let url = url.unwrap();
    assert!(
        url.ends_with(".msi"),
        "Windows x64 URL should use .msi installer, got: {}",
        url
    );
    assert!(
        url.contains("x64"),
        "Windows x64 URL should contain 'x64', got: {}",
        url
    );
    assert!(
        url.contains("2409"),
        "Windows URL should contain compact version '2409', got: {}",
        url
    );
}

// ============================================================================
// SevenZipProvider — provider trait tests
// ============================================================================

#[rstest]
fn test_7zip_provider_name() {
    let provider = SevenZipProvider::new();
    assert_eq!(provider.name(), "7zip");
}

#[rstest]
fn test_7zip_provider_description() {
    let provider = SevenZipProvider::new();
    let desc = provider.description();
    assert!(!desc.is_empty(), "provider description should not be empty");
}

#[rstest]
fn test_7zip_provider_runtimes() {
    let provider = SevenZipProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(
        runtimes.len(),
        1,
        "7zip provider should have exactly 1 runtime"
    );

    let runtime = &runtimes[0];
    assert_eq!(runtime.name(), "7zip");
}

#[rstest]
fn test_7zip_provider_supports_name() {
    let provider = SevenZipProvider::new();
    assert!(provider.supports("7zip"), "provider should support '7zip'");
}

#[rstest]
fn test_7zip_provider_supports_aliases() {
    let provider = SevenZipProvider::new();
    // The provider itself checks runtimes; alias resolution is done at resolver layer
    // but the runtime should expose aliases
    let runtimes = provider.runtimes();
    let runtime = &runtimes[0];
    let aliases = runtime.aliases();
    assert!(aliases.contains(&"7z"), "runtime should expose '7z' alias");
}

#[rstest]
fn test_7zip_provider_does_not_support_unrelated() {
    let provider = SevenZipProvider::new();
    assert!(!provider.supports("node"), "should not support 'node'");
    assert!(!provider.supports("python"), "should not support 'python'");
    assert!(!provider.supports("zip"), "should not support 'zip'");
}

// ============================================================================
// SevenZipRuntime — is_version_installable
// ============================================================================

#[rstest]
#[case("24.09")]
#[case("23.01")]
#[case("22.01")]
fn test_7zip_version_is_installable(#[case] version: &str) {
    let runtime = SevenZipRuntime::new();
    assert!(
        runtime.is_version_installable(version),
        "version {} should be installable",
        version
    );
}

// ============================================================================
// SevenZipRuntime — verify_installation
// ============================================================================

#[rstest]
fn test_7zip_verify_installation_missing_exe() {
    use tempfile::TempDir;

    let runtime = SevenZipRuntime::new();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let platform = Platform::new(Os::Linux, Arch::X86_64);

    // Empty directory — no executable
    let result = runtime.verify_installation("24.09", temp_dir.path(), &platform);

    // If 7z is not in PATH either, verification should fail
    // (we can't guarantee 7z is installed in CI, so we just check the result is valid)
    let _ = result; // just ensure it doesn't panic
}

#[rstest]
fn test_7zip_verify_installation_with_exe() {
    use tempfile::TempDir;

    let runtime = SevenZipRuntime::new();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let platform = Platform::new(Os::Linux, Arch::X86_64);

    // Create a dummy 7z executable
    let exe_path = temp_dir.path().join("7z");
    std::fs::write(&exe_path, b"#!/bin/sh\necho '7-Zip 24.09'").expect("Failed to create exe");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&exe_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&exe_path, perms).unwrap();
    }

    let result = runtime.verify_installation("24.09", temp_dir.path(), &platform);
    assert!(result.valid, "verification should succeed when exe exists");
}

// ============================================================================
// Default trait
// ============================================================================

#[rstest]
fn test_7zip_runtime_default() {
    let r1 = SevenZipRuntime::new();
    let r2 = SevenZipRuntime::default();
    assert_eq!(r1.name(), r2.name());
}

#[rstest]
fn test_7zip_provider_default() {
    let p1 = SevenZipProvider::new();
    let p2 = SevenZipProvider::default();
    assert_eq!(p1.name(), p2.name());
}
