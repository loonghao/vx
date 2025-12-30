//! Python runtime tests

use rstest::rstest;
use vx_provider_python::{PythonProvider, PythonRuntime, PythonUrlBuilder};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_provider_name() {
    let provider = PythonProvider::new();
    assert_eq!(provider.name(), "python");
}

#[test]
fn test_provider_description() {
    let provider = PythonProvider::new();
    assert!(provider
        .description()
        .contains("Python programming language"));
}

#[test]
fn test_provider_runtimes() {
    let provider = PythonProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "python");
}

#[test]
fn test_runtime_name() {
    let runtime = PythonRuntime::new();
    assert_eq!(runtime.name(), "python");
}

#[test]
fn test_runtime_description() {
    let runtime = PythonRuntime::new();
    let desc = runtime.description();
    assert!(desc.contains("Python"));
    assert!(desc.contains("3.7") || desc.contains("3.12"));
}

#[test]
fn test_runtime_aliases() {
    let runtime = PythonRuntime::new();
    let aliases = runtime.aliases();
    assert!(aliases.contains(&"python3"));
    assert!(aliases.contains(&"py"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = PythonRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::Python);
}

#[test]
fn test_runtime_metadata() {
    let runtime = PythonRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("repository"));
    assert!(meta.contains_key("license"));
    assert!(meta.contains_key("supported_versions"));
    assert!(meta.get("note").unwrap().contains("uv"));
    // Check supported versions includes 3.7 to 3.12+
    let supported = meta.get("supported_versions").unwrap();
    assert!(supported.contains("3.7"));
    assert!(supported.contains("3.12"));
}

#[rstest]
#[case(Os::Windows, Arch::X86_64, "python/python.exe")]
#[case(Os::Linux, Arch::X86_64, "python/bin/python3")]
#[case(Os::MacOS, Arch::Aarch64, "python/bin/python3")]
fn test_executable_relative_path(#[case] os: Os, #[case] arch: Arch, #[case] expected_path: &str) {
    let runtime = PythonRuntime::new();
    let platform = Platform { os, arch };
    let path = runtime.executable_relative_path("3.12.8", &platform);
    assert_eq!(path, expected_path);
}

#[rstest]
#[case(Os::Windows, Arch::X86_64, "x86_64-pc-windows-msvc")]
#[case(Os::Windows, Arch::X86, "i686-pc-windows-msvc")]
#[case(Os::Windows, Arch::Aarch64, "aarch64-pc-windows-msvc")]
#[case(Os::MacOS, Arch::X86_64, "x86_64-apple-darwin")]
#[case(Os::MacOS, Arch::Aarch64, "aarch64-apple-darwin")]
#[case(Os::Linux, Arch::X86_64, "x86_64-unknown-linux-gnu")]
#[case(Os::Linux, Arch::Aarch64, "aarch64-unknown-linux-gnu")]
fn test_platform_string(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let platform = Platform { os, arch };
    assert_eq!(PythonUrlBuilder::get_platform_string(&platform), expected);
}

#[test]
fn test_download_url_format_python312() {
    let platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };
    let url = PythonUrlBuilder::download_url_with_date("3.12.12", "20251217", &platform).unwrap();

    assert_eq!(
        url,
        "https://github.com/astral-sh/python-build-standalone/releases/download/20251217/cpython-3.12.12+20251217-x86_64-unknown-linux-gnu-install_only.tar.gz"
    );
}

#[test]
fn test_download_url_format_python311() {
    let platform = Platform {
        os: Os::Windows,
        arch: Arch::X86_64,
    };
    let url = PythonUrlBuilder::download_url_with_date("3.11.14", "20251217", &platform).unwrap();

    assert_eq!(
        url,
        "https://github.com/astral-sh/python-build-standalone/releases/download/20251217/cpython-3.11.14+20251217-x86_64-pc-windows-msvc-install_only.tar.gz"
    );
}

#[test]
fn test_download_url_macos_arm() {
    let platform = Platform {
        os: Os::MacOS,
        arch: Arch::Aarch64,
    };
    let url = PythonUrlBuilder::download_url_with_date("3.12.12", "20251217", &platform).unwrap();

    assert_eq!(
        url,
        "https://github.com/astral-sh/python-build-standalone/releases/download/20251217/cpython-3.12.12+20251217-aarch64-apple-darwin-install_only.tar.gz"
    );
}

#[test]
fn test_filename_includes_release_date() {
    let platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };

    // Test different release dates
    let filename1 =
        PythonUrlBuilder::get_filename_with_date("3.12.12", "20251217", &platform).unwrap();
    assert_eq!(
        filename1,
        "cpython-3.12.12+20251217-x86_64-unknown-linux-gnu-install_only.tar.gz"
    );

    let filename2 =
        PythonUrlBuilder::get_filename_with_date("3.10.16", "20241206", &platform).unwrap();
    assert_eq!(
        filename2,
        "cpython-3.10.16+20241206-x86_64-unknown-linux-gnu-install_only.tar.gz"
    );
}

#[rstest]
#[case("3.10.19", "20251217", Os::Linux)]
#[case("3.11.14", "20251217", Os::Linux)]
#[case("3.12.12", "20251217", Os::Linux)]
#[case("3.13.11", "20251217", Os::Linux)]
#[case("3.11.14", "20251217", Os::Windows)]
#[case("3.12.12", "20251217", Os::MacOS)]
fn test_download_url_various_versions(
    #[case] version: &str,
    #[case] release_date: &str,
    #[case] os: Os,
) {
    let platform = Platform {
        os,
        arch: Arch::X86_64,
    };
    let url = PythonUrlBuilder::download_url_with_date(version, release_date, &platform).unwrap();

    assert!(url.contains(&format!("cpython-{}", version)));
    assert!(url.contains(release_date));
    assert!(url.contains("install_only.tar.gz"));
}

#[test]
fn test_filename_format_no_variant() {
    // Verify the new format without variant (shared/pgo+lto)
    let windows_platform = Platform {
        os: Os::Windows,
        arch: Arch::X86_64,
    };
    let linux_platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };

    let windows_filename =
        PythonUrlBuilder::get_filename_with_date("3.11.14", "20251217", &windows_platform).unwrap();
    let linux_filename =
        PythonUrlBuilder::get_filename_with_date("3.11.14", "20251217", &linux_platform).unwrap();

    // Should NOT contain variant suffixes
    assert!(!windows_filename.contains("shared"));
    assert!(!linux_filename.contains("pgo+lto"));

    // Should end with platform-install_only.tar.gz
    assert!(windows_filename.ends_with("-x86_64-pc-windows-msvc-install_only.tar.gz"));
    assert!(linux_filename.ends_with("-x86_64-unknown-linux-gnu-install_only.tar.gz"));
}
