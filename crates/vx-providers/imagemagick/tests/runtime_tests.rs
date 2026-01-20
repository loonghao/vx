//! Tests for ImageMagick runtime

use rstest::rstest;
use vx_provider_imagemagick::{ImageMagickProvider, ImageMagickUrlBuilder, MagickRuntime};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_runtime_name() {
    let runtime = MagickRuntime::new();
    assert_eq!(runtime.name(), "magick");
}

#[test]
fn test_runtime_description() {
    let runtime = MagickRuntime::new();
    assert!(runtime.description().contains("ImageMagick"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = MagickRuntime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::System);
}

#[test]
fn test_runtime_aliases() {
    let runtime = MagickRuntime::new();
    assert!(runtime.aliases().contains(&"imagemagick"));
}

#[test]
fn test_runtime_metadata() {
    let runtime = MagickRuntime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
    assert!(meta.contains_key("repository"));
    assert!(meta.get("homepage").unwrap().contains("imagemagick.org"));
}

#[test]
fn test_provider_name() {
    let provider = ImageMagickProvider::new();
    assert_eq!(provider.name(), "imagemagick");
}

#[test]
fn test_provider_description() {
    let provider = ImageMagickProvider::new();
    assert!(provider.description().contains("ImageMagick"));
}

#[test]
fn test_provider_supports() {
    let provider = ImageMagickProvider::new();
    assert!(provider.supports("magick"));
    assert!(provider.supports("imagemagick"));
    assert!(provider.supports("convert"));
    assert!(!provider.supports("other"));
}

#[test]
fn test_provider_runtimes() {
    let provider = ImageMagickProvider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 2);

    let names: Vec<&str> = runtimes.iter().map(|r| r.name()).collect();
    assert!(names.contains(&"magick"));
    assert!(names.contains(&"convert"));
}

#[test]
fn test_provider_get_runtime() {
    let provider = ImageMagickProvider::new();
    assert!(provider.get_runtime("magick").is_some());
    assert!(provider.get_runtime("imagemagick").is_some());
    assert!(provider.get_runtime("convert").is_some());
    assert!(provider.get_runtime("other").is_none());
}

#[rstest]
#[case(Os::Linux, Arch::X86_64, true)]   // Linux x86_64: AppImage download supported
#[case(Os::Linux, Arch::Aarch64, false)] // Linux ARM: No download, use package manager
#[case(Os::MacOS, Arch::X86_64, false)]  // macOS: Use Homebrew
#[case(Os::MacOS, Arch::Aarch64, false)] // macOS ARM: Use Homebrew
#[case(Os::Windows, Arch::X86_64, false)]   // Windows: Use winget/choco/scoop
#[case(Os::Windows, Arch::Aarch64, false)]  // Windows ARM: Use winget/choco/scoop
fn test_direct_download_support(#[case] os: Os, #[case] arch: Arch, #[case] expected: bool) {
    let platform = Platform { os, arch };
    assert_eq!(
        ImageMagickUrlBuilder::is_direct_download_supported(&platform),
        expected
    );
}

#[rstest]
#[case(Os::Windows, "magick.exe")]
#[case(Os::Linux, "magick")]
#[case(Os::MacOS, "magick")]
fn test_executable_name(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform {
        os,
        arch: Arch::X86_64,
    };
    let name = ImageMagickUrlBuilder::get_executable_name(&platform);
    assert_eq!(name, expected);
}

#[test]
fn test_download_url_linux_x64() {
    let platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };
    let url = ImageMagickUrlBuilder::download_url("7.1.2-12", &platform);
    assert!(url.is_some());
    let url = url.unwrap();
    assert!(url.contains("imagemagick.org"));
    assert!(url.ends_with("/magick"));
}

#[test]
fn test_download_url_windows_uses_package_manager() {
    // Windows should use package managers (winget/choco/scoop), not direct download
    let platform = Platform {
        os: Os::Windows,
        arch: Arch::X86_64,
    };
    let url = ImageMagickUrlBuilder::download_url("7.1.2-12", &platform);
    assert!(url.is_none(), "Windows should use package managers, not direct download");
}

#[test]
fn test_download_url_macos_none() {
    let platform = Platform {
        os: Os::MacOS,
        arch: Arch::Aarch64,
    };
    let url = ImageMagickUrlBuilder::download_url("7.1.2-12", &platform);
    assert!(url.is_none());
}

/// Test executable_dir_path for magick runtime
#[test]
fn test_executable_dir_path() {
    let runtime = MagickRuntime::new();

    let linux_platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };
    assert_eq!(
        runtime.executable_dir_path("7.1.2-12", &linux_platform),
        Some("bin".to_string())
    );

    let windows_platform = Platform {
        os: Os::Windows,
        arch: Arch::X86_64,
    };
    assert_eq!(
        runtime.executable_dir_path("7.1.2-12", &windows_platform),
        Some("bin".to_string())
    );
}

/// Test that installation instructions are provided for unsupported platforms
#[test]
fn test_installation_instructions() {
    // macOS: Use Homebrew
    let macos = Platform {
        os: Os::MacOS,
        arch: Arch::Aarch64,
    };
    let instructions = ImageMagickUrlBuilder::get_installation_instructions(&macos);
    assert!(instructions.is_some());
    assert!(instructions.unwrap().contains("brew"));

    // Windows now uses package managers (winget/choco/scoop)
    let windows_x64 = Platform {
        os: Os::Windows,
        arch: Arch::X86_64,
    };
    let instructions = ImageMagickUrlBuilder::get_installation_instructions(&windows_x64);
    assert!(instructions.is_some());
    assert!(
        instructions.unwrap().contains("winget") || instructions.unwrap().contains("choco"),
        "Windows should have package manager instructions"
    );

    // Linux x86_64 supports direct download, no instructions needed
    let linux_x64 = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };
    let instructions = ImageMagickUrlBuilder::get_installation_instructions(&linux_x64);
    assert!(instructions.is_none(), "Linux x86_64 supports direct download");
}
