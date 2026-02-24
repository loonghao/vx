use vx_provider_conda::CondaUrlBuilder;
use vx_runtime::platform::Platform;
use vx_runtime::{Arch, Os};

#[test]
fn test_conda_download_url_linux() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let url = CondaUrlBuilder::conda_download_url("24.3.0-0", &platform);
    assert!(url.is_some());
    assert!(url.unwrap().contains("Miniforge3-24.3.0-0-Linux-x86_64.sh"));
}

#[test]
fn test_conda_download_url_windows() {
    let platform = Platform::new(Os::Windows, Arch::X86_64);
    let url = CondaUrlBuilder::conda_download_url("24.3.0-0", &platform);
    assert!(url.is_some());
    assert!(
        url.unwrap()
            .contains("Miniforge3-24.3.0-0-Windows-x86_64.exe")
    );
}

#[test]
fn test_conda_download_url_macos_arm() {
    let platform = Platform::new(Os::MacOS, Arch::Aarch64);
    let url = CondaUrlBuilder::conda_download_url("24.3.0-0", &platform);
    assert!(url.is_some());
    assert!(url.unwrap().contains("Miniforge3-24.3.0-0-MacOSX-arm64.sh"));
}

#[test]
fn test_micromamba_download_url_linux() {
    let platform = Platform::new(Os::Linux, Arch::X86_64);
    let url = CondaUrlBuilder::micromamba_download_url("2.0.0-0", &platform);
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("micromamba-linux-64.tar.bz2"));
    assert!(url.contains("2.0.0-0"));
}

#[test]
fn test_micromamba_download_url_windows() {
    let platform = Platform::new(Os::Windows, Arch::X86_64);
    let url = CondaUrlBuilder::micromamba_download_url("2.0.0-0", &platform);
    assert!(url.is_some());
    assert!(url.unwrap().contains("micromamba-win-64.tar.bz2"));
}

#[test]
fn test_micromamba_download_url_macos_arm() {
    let platform = Platform::new(Os::MacOS, Arch::Aarch64);
    let url = CondaUrlBuilder::micromamba_download_url("2.0.0-0", &platform);
    assert!(url.is_some());
    assert!(url.unwrap().contains("micromamba-osx-arm64.tar.bz2"));
}

#[test]
fn test_unsupported_platform_returns_none() {
    let platform = Platform::new(Os::Linux, Arch::Arm);
    assert!(CondaUrlBuilder::conda_download_url("24.3.0-0", &platform).is_none());
    assert!(CondaUrlBuilder::micromamba_download_url("2.0.0-0", &platform).is_none());
}
