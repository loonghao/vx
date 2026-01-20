//! Tests for silent installation features
//!
//! These tests verify that package managers are configured for silent,
//! non-interactive installation suitable for CI/automated environments.

use std::sync::{Arc, Mutex};
use vx_system_pm::managers::{ChocolateyManager, ScoopManager, WingetManager};
use vx_system_pm::{PackageInstallSpec, SystemPackageManager};

/// Test that Chocolatey manager can be created with progress callback
#[test]
fn test_chocolatey_with_progress_callback() {
    let messages: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let messages_clone = messages.clone();

    let _manager = ChocolateyManager::with_progress(move |msg| {
        messages_clone.lock().unwrap().push(msg.to_string());
    });

    // Manager should be created successfully
    assert!(messages.lock().unwrap().is_empty());
}

/// Test that winget manager can be created with progress callback
#[test]
fn test_winget_with_progress_callback() {
    let messages: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let messages_clone = messages.clone();

    let _manager = WingetManager::with_progress(move |msg| {
        messages_clone.lock().unwrap().push(msg.to_string());
    });

    // Manager should be created successfully
    assert!(messages.lock().unwrap().is_empty());
}

/// Test that PackageInstallSpec defaults to silent mode
#[test]
fn test_package_install_spec_defaults_to_silent() {
    let spec = PackageInstallSpec::new("test-package");

    // By default, silent should be true
    assert!(
        spec.silent,
        "PackageInstallSpec should default to silent mode"
    );
}

/// Test that PackageInstallSpec can be customized
#[test]
fn test_package_install_spec_builder() {
    let spec = PackageInstallSpec::new("imagemagick")
        .with_version("7.1.2-12")
        .with_params("/NoDesktopIcon")
        .with_install_args("/SILENT /VERYSILENT");

    assert_eq!(spec.package, "imagemagick");
    assert_eq!(spec.version, Some("7.1.2-12".to_string()));
    assert_eq!(spec.params, Some("/NoDesktopIcon".to_string()));
    assert_eq!(spec.install_args, Some("/SILENT /VERYSILENT".to_string()));
    assert!(spec.silent);
}

/// Test Chocolatey manager default configuration
#[test]
fn test_chocolatey_default_config() {
    let manager = ChocolateyManager::new();

    assert_eq!(manager.name(), "choco");
    assert_eq!(manager.supported_platforms(), vec!["windows"]);
    assert_eq!(manager.priority(), 80);
}

/// Test winget manager default configuration
#[test]
fn test_winget_default_config() {
    let manager = WingetManager::new();

    assert_eq!(manager.name(), "winget");
    assert_eq!(manager.supported_platforms(), vec!["windows"]);
    // winget has highest priority as it's built-in on Windows 11
    assert_eq!(manager.priority(), 95);
}

/// Test that silent install args are built correctly for Chocolatey
#[test]
fn test_chocolatey_silent_args() {
    let manager = ChocolateyManager::new();
    let base_args = vec!["install", "imagemagick"];
    let args = manager.build_silent_args(base_args);

    // Should contain silent installation flags
    assert!(args.contains(&"-y"), "Should include -y for auto-confirm");
    assert!(
        args.contains(&"--no-progress"),
        "Should include --no-progress"
    );
    assert!(
        args.contains(&"--limit-output"),
        "Should include --limit-output"
    );

    // Should preserve base args
    assert!(args.contains(&"install"));
    assert!(args.contains(&"imagemagick"));
}

/// Test priority ordering of Windows package managers
#[test]
fn test_windows_package_manager_priority_order() {
    let winget = WingetManager::new();
    let choco = ChocolateyManager::new();
    let scoop = ScoopManager::new();

    // winget should have highest priority (built-in on Windows 11)
    assert!(
        winget.priority() > choco.priority(),
        "winget should have higher priority than choco"
    );
    assert!(
        choco.priority() > scoop.priority(),
        "choco should have higher priority than scoop"
    );

    // Verify exact priorities
    assert_eq!(winget.priority(), 95);
    assert_eq!(choco.priority(), 80);
    assert_eq!(scoop.priority(), 60);
}
