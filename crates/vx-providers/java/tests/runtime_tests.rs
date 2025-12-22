//! Java (Temurin JDK) runtime tests

use rstest::rstest;
use vx_provider_java::{JavaProvider, JavaRuntime};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_java_runtime_creation() {
    let runtime = JavaRuntime::new();
    assert_eq!(runtime.name(), "java");
    assert!(!runtime.description().is_empty());
    assert_eq!(runtime.ecosystem(), Ecosystem::Unknown);
}

#[test]
fn test_java_runtime_description() {
    let runtime = JavaRuntime::new();
    assert!(
        runtime.description().contains("Java")
            || runtime.description().contains("JDK")
            || runtime.description().contains("Temurin")
    );
}

#[test]
fn test_java_runtime_metadata() {
    let runtime = JavaRuntime::new();
    let metadata = runtime.metadata();

    assert!(metadata.contains_key("homepage"));
    assert!(metadata.contains_key("ecosystem"));
    assert!(metadata.contains_key("repository"));
    assert!(metadata.contains_key("license"));
    assert_eq!(
        metadata.get("homepage"),
        Some(&"https://adoptium.net/".to_string())
    );
    assert_eq!(metadata.get("ecosystem"), Some(&"java".to_string()));
}

#[test]
fn test_java_runtime_aliases() {
    let runtime = JavaRuntime::new();
    let aliases = runtime.aliases();
    assert!(aliases.contains(&"jdk"));
    assert!(aliases.contains(&"temurin"));
    assert!(aliases.contains(&"openjdk"));
}

#[test]
fn test_java_provider_creation() {
    let provider = JavaProvider::new();
    assert_eq!(provider.name(), "java");
    assert!(!provider.description().is_empty());
}

#[test]
fn test_java_provider_runtimes() {
    let provider = JavaProvider::new();
    let runtimes = provider.runtimes();

    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "java");
}

#[rstest]
#[case("java", true)]
#[case("jdk", true)]
#[case("temurin", true)]
#[case("openjdk", true)]
#[case("python", false)]
#[case("node", false)]
fn test_java_provider_supports(#[case] name: &str, #[case] expected: bool) {
    let provider = JavaProvider::new();
    assert_eq!(provider.supports(name), expected);
}

#[test]
fn test_java_provider_get_runtime() {
    let provider = JavaProvider::new();

    let java = provider.get_runtime("java");
    assert!(java.is_some());
    assert_eq!(java.unwrap().name(), "java");

    let jdk = provider.get_runtime("jdk");
    assert!(jdk.is_some());
    assert_eq!(jdk.unwrap().name(), "java");

    let temurin = provider.get_runtime("temurin");
    assert!(temurin.is_some());

    let unknown = provider.get_runtime("unknown");
    assert!(unknown.is_none());
}

/// Test that executable_relative_path returns correct path for Java archives
/// Java archives extract to a versioned directory like jdk-21.0.1+12/
#[rstest]
#[case(Os::Linux, Arch::X86_64, "*/bin/java")]
#[case(Os::Linux, Arch::Aarch64, "*/bin/java")]
#[case(Os::Windows, Arch::X86_64, "*/bin/java.exe")]
fn test_java_executable_relative_path_linux_windows(
    #[case] os: Os,
    #[case] arch: Arch,
    #[case] expected: &str,
) {
    let runtime = JavaRuntime::new();
    let platform = Platform { os, arch };
    let path = runtime.executable_relative_path("21", &platform);
    assert_eq!(path, expected);
}

/// macOS has a different structure (Contents/Home/bin/java)
#[rstest]
#[case(Os::MacOS, Arch::X86_64, "*/Contents/Home/bin/java")]
#[case(Os::MacOS, Arch::Aarch64, "*/Contents/Home/bin/java")]
fn test_java_executable_relative_path_macos(
    #[case] os: Os,
    #[case] arch: Arch,
    #[case] expected: &str,
) {
    let runtime = JavaRuntime::new();
    let platform = Platform { os, arch };
    let path = runtime.executable_relative_path("21", &platform);
    assert_eq!(path, expected);
}

#[tokio::test]
async fn test_java_download_url_format() {
    let runtime = JavaRuntime::new();
    let platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };

    let url = runtime.download_url("21", &platform).await.unwrap();
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("api.adoptium.net"));
    assert!(url.contains("/21/"));
    assert!(url.contains("linux"));
    assert!(url.contains("x64"));
}

#[tokio::test]
async fn test_java_download_url_macos() {
    let runtime = JavaRuntime::new();
    let platform = Platform {
        os: Os::MacOS,
        arch: Arch::Aarch64,
    };

    let url = runtime.download_url("17", &platform).await.unwrap();
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("mac"));
    assert!(url.contains("aarch64"));
}
