//! Node.js runtime tests

use rstest::rstest;
use vx_provider_node::{NodeProvider, NodeRuntime, NpmRuntime, NpxRuntime};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_node_runtime_creation() {
    let runtime = NodeRuntime::new();
    assert_eq!(runtime.name(), "node");
    assert!(!runtime.description().is_empty());
    assert!(runtime.aliases().contains(&"nodejs"));
    assert_eq!(runtime.ecosystem(), Ecosystem::NodeJs);
}

#[test]
fn test_npm_runtime_creation() {
    let runtime = NpmRuntime::new();
    assert_eq!(runtime.name(), "npm");
    assert!(!runtime.description().is_empty());
    assert_eq!(runtime.ecosystem(), Ecosystem::NodeJs);
}

#[test]
fn test_npx_runtime_creation() {
    let runtime = NpxRuntime::new();
    assert_eq!(runtime.name(), "npx");
    assert!(!runtime.description().is_empty());
    assert_eq!(runtime.ecosystem(), Ecosystem::NodeJs);
}

#[test]
fn test_node_runtime_metadata() {
    let runtime = NodeRuntime::new();
    let metadata = runtime.metadata();

    assert!(metadata.contains_key("homepage"));
    assert!(metadata.contains_key("ecosystem"));
    assert_eq!(metadata.get("ecosystem"), Some(&"javascript".to_string()));
}

#[test]
fn test_node_provider_creation() {
    let provider = NodeProvider::new();
    assert_eq!(provider.name(), "node");
    assert!(!provider.description().is_empty());
}

#[test]
fn test_node_provider_runtimes() {
    let provider = NodeProvider::new();
    let runtimes = provider.runtimes();

    assert_eq!(runtimes.len(), 3);

    let names: Vec<&str> = runtimes.iter().map(|r| r.name()).collect();
    assert!(names.contains(&"node"));
    assert!(names.contains(&"npm"));
    assert!(names.contains(&"npx"));
}

#[rstest]
#[case("node", true)]
#[case("nodejs", true)]
#[case("npm", true)]
#[case("npx", true)]
#[case("go", false)]
#[case("python", false)]
fn test_node_provider_supports(#[case] name: &str, #[case] expected: bool) {
    let provider = NodeProvider::new();
    assert_eq!(provider.supports(name), expected);
}

#[test]
fn test_node_provider_get_runtime() {
    let provider = NodeProvider::new();

    let node = provider.get_runtime("node");
    assert!(node.is_some());
    assert_eq!(node.unwrap().name(), "node");

    let nodejs = provider.get_runtime("nodejs");
    assert!(nodejs.is_some());
    assert_eq!(nodejs.unwrap().name(), "node");

    let npm = provider.get_runtime("npm");
    assert!(npm.is_some());
    assert_eq!(npm.unwrap().name(), "npm");

    let unknown = provider.get_runtime("unknown");
    assert!(unknown.is_none());
}

// ============================================================================
// Windows executable path tests
// ============================================================================

/// Test that Node.js executable path is correct for Windows
/// Windows Node.js archives have executables directly in the root (no bin subdirectory)
#[rstest]
#[case("20.10.0", Os::Windows, Arch::X86_64, "node-v20.10.0-win-x64/node.exe")]
#[case(
    "20.10.0",
    Os::Windows,
    Arch::Aarch64,
    "node-v20.10.0-win-arm64/node.exe"
)]
#[case("20.10.0", Os::Linux, Arch::X86_64, "node-v20.10.0-linux-x64/bin/node")]
#[case(
    "20.10.0",
    Os::MacOS,
    Arch::Aarch64,
    "node-v20.10.0-darwin-arm64/bin/node"
)]
fn test_node_executable_relative_path(
    #[case] version: &str,
    #[case] os: Os,
    #[case] arch: Arch,
    #[case] expected: &str,
) {
    let runtime = NodeRuntime::new();
    let platform = Platform::new(os, arch);
    let path = runtime.executable_relative_path(version, &platform);
    assert_eq!(path, expected);
}

/// Test that NPM executable path is correct for Windows
/// Windows uses npm.cmd directly in the root, Unix uses bin/npm
#[rstest]
#[case("20.10.0", Os::Windows, Arch::X86_64, "node-v20.10.0-win-x64/npm.cmd")]
#[case(
    "20.10.0",
    Os::Windows,
    Arch::Aarch64,
    "node-v20.10.0-win-arm64/npm.cmd"
)]
#[case("20.10.0", Os::Linux, Arch::X86_64, "node-v20.10.0-linux-x64/bin/npm")]
#[case(
    "20.10.0",
    Os::MacOS,
    Arch::Aarch64,
    "node-v20.10.0-darwin-arm64/bin/npm"
)]
fn test_npm_executable_relative_path(
    #[case] version: &str,
    #[case] os: Os,
    #[case] arch: Arch,
    #[case] expected: &str,
) {
    let runtime = NpmRuntime::new();
    let platform = Platform::new(os, arch);
    let path = runtime.executable_relative_path(version, &platform);
    assert_eq!(path, expected);
}

/// Test that NPX executable path is correct for Windows
/// Windows uses npx.cmd directly in the root, Unix uses bin/npx
#[rstest]
#[case("20.10.0", Os::Windows, Arch::X86_64, "node-v20.10.0-win-x64/npx.cmd")]
#[case(
    "20.10.0",
    Os::Windows,
    Arch::Aarch64,
    "node-v20.10.0-win-arm64/npx.cmd"
)]
#[case("20.10.0", Os::Linux, Arch::X86_64, "node-v20.10.0-linux-x64/bin/npx")]
#[case(
    "20.10.0",
    Os::MacOS,
    Arch::Aarch64,
    "node-v20.10.0-darwin-arm64/bin/npx"
)]
fn test_npx_executable_relative_path(
    #[case] version: &str,
    #[case] os: Os,
    #[case] arch: Arch,
    #[case] expected: &str,
) {
    let runtime = NpxRuntime::new();
    let platform = Platform::new(os, arch);
    let path = runtime.executable_relative_path(version, &platform);
    assert_eq!(path, expected);
}

/// Test archive directory name generation
#[rstest]
#[case("20.10.0", Os::Windows, Arch::X86_64, "node-v20.10.0-win-x64")]
#[case("18.19.0", Os::Linux, Arch::X86_64, "node-v18.19.0-linux-x64")]
#[case("21.5.0", Os::MacOS, Arch::Aarch64, "node-v21.5.0-darwin-arm64")]
fn test_node_archive_dir_name(
    #[case] version: &str,
    #[case] os: Os,
    #[case] arch: Arch,
    #[case] expected: &str,
) {
    let platform = Platform::new(os, arch);
    let dir_name = NodeRuntime::get_archive_dir_name(version, &platform);
    assert_eq!(dir_name, expected);
}
