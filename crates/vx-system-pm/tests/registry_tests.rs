//! Tests for PackageManagerRegistry

use vx_system_pm::PackageManagerRegistry;

#[test]
fn test_registry_creation() {
    let registry = PackageManagerRegistry::new();
    
    // Should have default managers registered
    assert!(registry.get("choco").is_ok());
    assert!(registry.get("winget").is_ok());
    assert!(registry.get("brew").is_ok());
    assert!(registry.get("apt").is_ok());
}

#[test]
fn test_unknown_manager() {
    let registry = PackageManagerRegistry::new();
    
    let result = registry.get("unknown-package-manager");
    assert!(result.is_err());
}

#[test]
fn test_for_current_platform() {
    let registry = PackageManagerRegistry::new();
    let managers = registry.for_current_platform();
    
    // Should return at least some managers for the current platform
    #[cfg(windows)]
    {
        assert!(managers.iter().any(|m| m.name() == "choco"));
        assert!(managers.iter().any(|m| m.name() == "winget"));
    }
    
    #[cfg(target_os = "macos")]
    {
        assert!(managers.iter().any(|m| m.name() == "brew"));
    }
    
    #[cfg(target_os = "linux")]
    {
        // Linux should have apt and/or brew
        let has_linux_pm = managers.iter().any(|m| m.name() == "apt" || m.name() == "brew");
        assert!(has_linux_pm);
    }
}

#[tokio::test]
async fn test_get_available() {
    let registry = PackageManagerRegistry::new();
    let available = registry.get_available().await;
    
    // This just tests that the method works without panicking
    // The actual availability depends on the system
    for pm in &available {
        println!("Available package manager: {}", pm.name());
    }
}
