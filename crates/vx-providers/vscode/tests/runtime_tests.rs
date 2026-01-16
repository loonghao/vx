//! Tests for VSCode runtime

use rstest::rstest;
use vx_provider_vscode::{VscodeProvider, VscodeRuntime, VscodeUrlBuilder};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_vscode_runtime_name() {
    let runtime = VscodeRuntime::new();
    assert_eq!(runtime.name(), "code");
}

#[test]
fn test_vscode_runtime_aliases() {
    let runtime = VscodeRuntime::new();
    let aliases = runtime.aliases();
    assert!(aliases.contains(&"vscode"));
    assert!(aliases.contains(&"vs-code"));
    assert!(aliases.contains(&"visual-studio-code"));
}

#[test]
fn test_vscode_runtime_ecosystem() {
    let runtime = VscodeRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::System);
}

#[test]
fn test_vscode_runtime_metadata() {
    let runtime = VscodeRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("repository"));
    assert!(meta.get("homepage").unwrap().contains("visualstudio"));
}

#[rstest]
#[case(Os::Windows, Arch::X86_64, "win32-x64")]
#[case(Os::Windows, Arch::X86, "win32")]
#[case(Os::Windows, Arch::Aarch64, "win32-arm64")]
#[case(Os::MacOS, Arch::X86_64, "darwin-x64")]
#[case(Os::MacOS, Arch::Aarch64, "darwin-arm64")]
#[case(Os::Linux, Arch::X86_64, "linux-x64")]
#[case(Os::Linux, Arch::Aarch64, "linux-arm64")]
fn test_platform_string(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let platform = Platform { os, arch };
    assert_eq!(VscodeUrlBuilder::get_platform_string(&platform), expected);
}

#[rstest]
#[case(
    Os::Windows,
    Arch::X86_64,
    "1.85.0",
    "https://update.code.visualstudio.com/1.85.0/win32-x64-archive/stable#.zip"
)]
#[case(
    Os::MacOS,
    Arch::Aarch64,
    "1.85.0",
    "https://update.code.visualstudio.com/1.85.0/darwin-arm64/stable#.zip"
)]
#[case(
    Os::Linux,
    Arch::X86_64,
    "1.85.0",
    "https://update.code.visualstudio.com/1.85.0/linux-x64/stable#.tar.gz"
)]
fn test_download_url(
    #[case] os: Os,
    #[case] arch: Arch,
    #[case] version: &str,
    #[case] expected: &str,
) {
    let platform = Platform { os, arch };
    let url = VscodeUrlBuilder::download_url(version, &platform);
    assert_eq!(url, Some(expected.to_string()));
}

#[rstest]
#[case(Os::Windows, Arch::X86_64, "bin/code.cmd")]
#[case(
    Os::MacOS,
    Arch::Aarch64,
    "Visual Studio Code.app/Contents/Resources/app/bin/code"
)]
#[case(Os::Linux, Arch::X86_64, "VSCode-linux-x64/bin/code")]
#[case(Os::Linux, Arch::Aarch64, "VSCode-linux-arm64/bin/code")]
fn test_executable_relative_path(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let runtime = VscodeRuntime::new();
    let platform = Platform { os, arch };
    assert_eq!(
        runtime.executable_relative_path("1.85.0", &platform),
        expected
    );
}

#[test]
fn test_vscode_provider_name() {
    let provider = VscodeProvider::new();
    assert_eq!(provider.name(), "vscode");
}

#[test]
fn test_vscode_provider_runtimes() {
    let provider = VscodeProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "code");
}

#[test]
fn test_vscode_provider_supports() {
    let provider = VscodeProvider::new();
    assert!(provider.supports("code"));
    assert!(provider.supports("vscode"));
    assert!(provider.supports("vs-code"));
    assert!(!provider.supports("unknown"));
}

#[test]
fn test_create_provider() {
    let provider = vx_provider_vscode::create_provider();
    assert_eq!(provider.name(), "vscode");
}
