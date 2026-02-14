//! Integration tests for vx-installer

use std::path::PathBuf;
use tempfile::TempDir;
use vx_installer::{
    ArchiveFormat, InstallConfig, InstallMethod, Installer,
    progress::{ProgressContext, ProgressStyle},
};

/// Test basic installer creation
#[tokio::test]
async fn test_installer_creation() {
    let installer = Installer::new().await;
    assert!(installer.is_ok(), "Should be able to create installer");
}

/// Test install config builder
#[test]
fn test_install_config_builder() {
    let temp_dir = TempDir::new().unwrap();
    let install_dir = temp_dir.path().join("test-tool").join("1.0.0");

    let config = InstallConfig::builder()
        .tool_name("test-tool")
        .version("1.0.0")
        .install_method(InstallMethod::Binary)
        .download_url("https://example.com/test-tool")
        .install_dir(install_dir.clone())
        .force(true)
        .checksum("abc123")
        .metadata("platform", "linux-x64")
        .build();

    assert_eq!(config.tool_name, "test-tool");
    assert_eq!(config.version, "1.0.0");
    assert!(matches!(config.install_method, InstallMethod::Binary));
    assert_eq!(
        config.download_url,
        Some("https://example.com/test-tool".to_string())
    );
    assert_eq!(config.install_dir, install_dir);
    assert!(config.force);
    assert_eq!(config.checksum, Some("abc123".to_string()));
    assert_eq!(
        config.metadata.get("platform"),
        Some(&"linux-x64".to_string())
    );
}

/// Test archive format detection
#[test]
fn test_archive_format_detection() {
    use std::path::Path;
    use vx_installer::formats::detect_format;

    assert_eq!(detect_format(Path::new("test.zip")), Some("zip"));
    assert_eq!(detect_format(Path::new("test.tar.gz")), Some("tar.gz"));
    assert_eq!(detect_format(Path::new("test.tgz")), Some("tar.gz"));
    assert_eq!(detect_format(Path::new("test.tar.xz")), Some("tar.xz"));
    assert_eq!(detect_format(Path::new("test.tar.bz2")), Some("tar.bz2"));
    assert_eq!(detect_format(Path::new("test.tar.zst")), Some("tar.zst"));
    assert_eq!(detect_format(Path::new("test.tzst")), Some("tar.zst"));
    assert_eq!(detect_format(Path::new("test.exe")), Some("exe"));
}

/// Test progress context creation
#[test]
fn test_progress_context() {
    let progress = ProgressContext::disabled();
    assert!(!progress.is_enabled());

    let style = ProgressStyle::default();
    let progress = ProgressContext::new(
        vx_installer::progress::create_progress_reporter(style, false),
        false,
    );
    assert!(!progress.is_enabled());
}

/// Test progress styles
#[test]
fn test_progress_styles() {
    let default_style = ProgressStyle::default();
    assert!(default_style.show_elapsed);
    assert!(default_style.show_eta);
    assert!(default_style.show_rate);

    let simple_style = ProgressStyle::simple();
    assert!(!simple_style.show_elapsed);
    assert!(!simple_style.show_eta);
    assert!(!simple_style.show_rate);

    let minimal_style = ProgressStyle::minimal();
    assert!(!minimal_style.show_elapsed);
    assert!(!minimal_style.show_eta);
    assert!(!minimal_style.show_rate);
}

/// Test error types
#[test]
fn test_error_types() {
    use vx_installer::Error;

    let download_error = Error::download_failed("https://example.com", "Network timeout");
    assert!(download_error.is_network_error());
    assert!(download_error.is_recoverable());

    let install_error = Error::installation_failed("tool", "1.0.0", "Failed to extract");
    assert!(!install_error.is_network_error());
    assert!(!install_error.is_recoverable());

    let extraction_error = Error::extraction_failed("/tmp/archive.zip", "Corrupted file");
    assert!(!extraction_error.is_network_error());
    assert!(!extraction_error.is_recoverable());
}

/// Test install method variants
#[test]
fn test_install_methods() {
    let binary_method = InstallMethod::Binary;
    assert!(matches!(binary_method, InstallMethod::Binary));

    let zip_method = InstallMethod::Archive {
        format: ArchiveFormat::Zip,
    };
    assert!(matches!(
        zip_method,
        InstallMethod::Archive {
            format: ArchiveFormat::Zip
        }
    ));

    let tar_gz_method = InstallMethod::Archive {
        format: ArchiveFormat::TarGz,
    };
    assert!(matches!(
        tar_gz_method,
        InstallMethod::Archive {
            format: ArchiveFormat::TarGz
        }
    ));

    let script_method = InstallMethod::Script {
        url: "https://example.com/install.sh".to_string(),
    };
    assert!(matches!(script_method, InstallMethod::Script { .. }));

    let package_method = InstallMethod::PackageManager {
        manager: "apt".to_string(),
        package: "nodejs".to_string(),
    };
    assert!(matches!(
        package_method,
        InstallMethod::PackageManager { .. }
    ));
}

/// Test that installer can check installation status
#[tokio::test]
async fn test_installation_check() {
    let installer = Installer::new().await.unwrap();
    let temp_dir = TempDir::new().unwrap();
    let install_dir = temp_dir.path().join("nonexistent-tool").join("1.0.0");

    let config = InstallConfig::builder()
        .tool_name("nonexistent-tool")
        .version("1.0.0")
        .install_dir(install_dir)
        .build();

    // Should return false for non-existent installation
    let is_installed = installer.is_installed(&config).await.unwrap();
    assert!(
        !is_installed,
        "Non-existent tool should not be reported as installed"
    );
}

/// Test version information
#[test]
fn test_version_info() {
    let version = vx_installer::VERSION;
    assert!(!version.is_empty(), "Version should not be empty");

    let user_agent = vx_installer::USER_AGENT;
    assert!(
        user_agent.contains("vx-installer"),
        "User agent should contain vx-installer"
    );
    assert!(
        user_agent.contains(version),
        "User agent should contain version"
    );
}

/// Test that all format handlers can be created
#[test]
fn test_format_handlers() {
    use std::path::Path;
    use vx_installer::formats::{ArchiveExtractor, FormatHandler};
    use vx_installer::formats::{binary::BinaryHandler, tar::TarHandler, zip::ZipHandler};

    let zip_handler = ZipHandler::new();
    assert_eq!(zip_handler.name(), "zip");
    assert!(zip_handler.can_handle(Path::new("test.zip")));
    assert!(!zip_handler.can_handle(Path::new("test.tar.gz")));

    let tar_handler = TarHandler::new();
    assert_eq!(tar_handler.name(), "tar");
    assert!(tar_handler.can_handle(Path::new("test.tar.gz")));
    assert!(tar_handler.can_handle(Path::new("test.tgz")));
    assert!(!tar_handler.can_handle(Path::new("test.zip")));

    let binary_handler = BinaryHandler::new();
    assert_eq!(binary_handler.name(), "binary");

    let _extractor = ArchiveExtractor::new();
    // ArchiveExtractor::new() succeeds without panic - test passes
}

/// Test downloader configuration
#[test]
fn test_downloader_config() {
    use std::time::Duration;
    use vx_installer::downloader::DownloadConfig;

    let config = DownloadConfig::new("https://example.com/file.zip", "/tmp/file.zip")
        .with_checksum("abc123")
        .with_max_retries(5)
        .with_timeout(Duration::from_secs(600))
        .with_overwrite(true);

    assert_eq!(config.url, "https://example.com/file.zip");
    assert_eq!(config.output_path, PathBuf::from("/tmp/file.zip"));
    assert_eq!(config.checksum, Some("abc123".to_string()));
    assert_eq!(config.max_retries, 5);
    assert_eq!(config.timeout, Duration::from_secs(600));
    assert!(config.overwrite);
}

/// Test that the installer can be used with vx-core adapter
#[tokio::test]
async fn test_vx_core_integration() {
    // This test verifies that the types are compatible
    use vx_installer::{ArchiveFormat, InstallConfig, InstallMethod};

    let config = InstallConfig::builder()
        .tool_name("test")
        .version("1.0.0")
        .install_method(InstallMethod::Archive {
            format: ArchiveFormat::Zip,
        })
        .build();

    // Should be able to serialize/deserialize
    let json = serde_json::to_string(&config).unwrap();
    let _deserialized: InstallConfig = serde_json::from_str(&json).unwrap();
}
