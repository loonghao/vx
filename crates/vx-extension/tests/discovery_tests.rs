//! Tests for extension discovery

use rstest::rstest;
use std::fs;
use tempfile::TempDir;
use vx_extension::{ExtensionDiscovery, ExtensionSource};

/// Helper to create a test extension directory with config
fn create_test_extension(dir: &std::path::Path, name: &str) {
    let ext_dir = dir.join(name);
    fs::create_dir_all(&ext_dir).unwrap();
    fs::write(
        ext_dir.join("vx-extension.toml"),
        format!(
            r#"
[extension]
name = "{}"
version = "1.0.0"
description = "Test extension {}"

[runtime]
requires = "python >= 3.10"

[entrypoint]
main = "main.py"

[commands.run]
description = "Run the extension"
script = "run.py"
"#,
            name, name
        ),
    )
    .unwrap();

    // Create dummy script files
    fs::write(ext_dir.join("main.py"), "# main script").unwrap();
    fs::write(ext_dir.join("run.py"), "# run script").unwrap();
}

/// Helper to create an invalid extension (missing config)
fn create_invalid_extension(dir: &std::path::Path, name: &str) {
    let ext_dir = dir.join(name);
    fs::create_dir_all(&ext_dir).unwrap();
    // No vx-extension.toml
}

/// Helper to create an extension with invalid config
fn create_extension_with_invalid_config(dir: &std::path::Path, name: &str) {
    let ext_dir = dir.join(name);
    fs::create_dir_all(&ext_dir).unwrap();
    fs::write(
        ext_dir.join("vx-extension.toml"),
        r#"
[extension
name = "broken"
"#,
    )
    .unwrap();
}

// ============ Discovery Tests ============

#[tokio::test]
async fn test_discover_single_extension() {
    let temp_dir = TempDir::new().unwrap();
    let ext_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&ext_dir).unwrap();

    create_test_extension(&ext_dir, "test-ext");

    let discovery = ExtensionDiscovery::with_dirs(
        ext_dir.clone(),
        temp_dir.path().join("extensions-dev"),
        None,
    );

    let extensions = discovery.discover_all().await.unwrap();
    assert_eq!(extensions.len(), 1);
    assert_eq!(extensions[0].name, "test-ext");
    assert_eq!(extensions[0].source, ExtensionSource::User);
}

#[tokio::test]
async fn test_discover_multiple_extensions() {
    let temp_dir = TempDir::new().unwrap();
    let ext_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&ext_dir).unwrap();

    create_test_extension(&ext_dir, "ext-a");
    create_test_extension(&ext_dir, "ext-b");
    create_test_extension(&ext_dir, "ext-c");

    let discovery = ExtensionDiscovery::with_dirs(
        ext_dir.clone(),
        temp_dir.path().join("extensions-dev"),
        None,
    );

    let extensions = discovery.discover_all().await.unwrap();
    assert_eq!(extensions.len(), 3);

    let names: Vec<&str> = extensions.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"ext-a"));
    assert!(names.contains(&"ext-b"));
    assert!(names.contains(&"ext-c"));
}

#[tokio::test]
async fn test_discover_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let ext_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&ext_dir).unwrap();

    let discovery = ExtensionDiscovery::with_dirs(
        ext_dir.clone(),
        temp_dir.path().join("extensions-dev"),
        None,
    );

    let extensions = discovery.discover_all().await.unwrap();
    assert!(extensions.is_empty());
}

#[tokio::test]
async fn test_discover_nonexistent_directory() {
    let temp_dir = TempDir::new().unwrap();

    let discovery = ExtensionDiscovery::with_dirs(
        temp_dir.path().join("nonexistent"),
        temp_dir.path().join("extensions-dev"),
        None,
    );

    let extensions = discovery.discover_all().await.unwrap();
    assert!(extensions.is_empty());
}

#[tokio::test]
async fn test_discover_skips_invalid_extensions() {
    let temp_dir = TempDir::new().unwrap();
    let ext_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&ext_dir).unwrap();

    create_test_extension(&ext_dir, "valid-ext");
    create_invalid_extension(&ext_dir, "no-config");
    create_extension_with_invalid_config(&ext_dir, "bad-config");

    let discovery = ExtensionDiscovery::with_dirs(
        ext_dir.clone(),
        temp_dir.path().join("extensions-dev"),
        None,
    );

    let extensions = discovery.discover_all().await.unwrap();
    assert_eq!(extensions.len(), 1);
    assert_eq!(extensions[0].name, "valid-ext");
}

// ============ Find Extension Tests ============

#[tokio::test]
async fn test_find_extension_by_name() {
    let temp_dir = TempDir::new().unwrap();
    let ext_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&ext_dir).unwrap();

    create_test_extension(&ext_dir, "target-ext");
    create_test_extension(&ext_dir, "other-ext");

    let discovery = ExtensionDiscovery::with_dirs(
        ext_dir.clone(),
        temp_dir.path().join("extensions-dev"),
        None,
    );

    let found = discovery.find_extension("target-ext").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "target-ext");
}

#[tokio::test]
async fn test_find_extension_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let ext_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&ext_dir).unwrap();

    create_test_extension(&ext_dir, "existing-ext");

    let discovery = ExtensionDiscovery::with_dirs(
        ext_dir.clone(),
        temp_dir.path().join("extensions-dev"),
        None,
    );

    let found = discovery.find_extension("nonexistent").await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_find_extension_or_error() {
    let temp_dir = TempDir::new().unwrap();
    let ext_dir = temp_dir.path().join("extensions");
    fs::create_dir_all(&ext_dir).unwrap();

    create_test_extension(&ext_dir, "my-ext");

    let discovery = ExtensionDiscovery::with_dirs(
        ext_dir.clone(),
        temp_dir.path().join("extensions-dev"),
        None,
    );

    // Found case
    let result = discovery.find_extension_or_error("my-ext").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().name, "my-ext");

    // Not found case
    let result = discovery.find_extension_or_error("nonexistent").await;
    assert!(result.is_err());
}

// ============ Priority Tests ============

#[tokio::test]
async fn test_extension_priority_dev_over_user() {
    let temp_dir = TempDir::new().unwrap();
    let user_dir = temp_dir.path().join("extensions");
    let dev_dir = temp_dir.path().join("extensions-dev");
    fs::create_dir_all(&user_dir).unwrap();
    fs::create_dir_all(&dev_dir).unwrap();

    // Create same extension in both directories
    create_test_extension(&user_dir, "shared-ext");
    create_test_extension(&dev_dir, "shared-ext");

    let discovery = ExtensionDiscovery::with_dirs(user_dir, dev_dir, None);

    // Find should return dev version (higher priority)
    let found = discovery.find_extension("shared-ext").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().source, ExtensionSource::Dev);
}

#[tokio::test]
async fn test_extension_priority_project_over_user() {
    let temp_dir = TempDir::new().unwrap();
    let user_dir = temp_dir.path().join("extensions");
    let project_dir = temp_dir
        .path()
        .join("project")
        .join(".vx")
        .join("extensions");
    fs::create_dir_all(&user_dir).unwrap();
    fs::create_dir_all(&project_dir).unwrap();

    create_test_extension(&user_dir, "shared-ext");
    create_test_extension(&project_dir, "shared-ext");

    let discovery = ExtensionDiscovery::with_dirs(
        user_dir,
        temp_dir.path().join("extensions-dev"),
        Some(project_dir),
    );

    let found = discovery.find_extension("shared-ext").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().source, ExtensionSource::Project);
}

#[tokio::test]
async fn test_discover_all_sorted_by_priority() {
    let temp_dir = TempDir::new().unwrap();
    let user_dir = temp_dir.path().join("extensions");
    let dev_dir = temp_dir.path().join("extensions-dev");
    let project_dir = temp_dir
        .path()
        .join("project")
        .join(".vx")
        .join("extensions");
    fs::create_dir_all(&user_dir).unwrap();
    fs::create_dir_all(&dev_dir).unwrap();
    fs::create_dir_all(&project_dir).unwrap();

    create_test_extension(&user_dir, "user-ext");
    create_test_extension(&dev_dir, "dev-ext");
    create_test_extension(&project_dir, "project-ext");

    let discovery = ExtensionDiscovery::with_dirs(user_dir, dev_dir, Some(project_dir));

    let extensions = discovery.discover_all().await.unwrap();
    assert_eq!(extensions.len(), 3);

    // Should be sorted by priority: dev > project > user
    assert_eq!(extensions[0].source, ExtensionSource::Dev);
    assert_eq!(extensions[1].source, ExtensionSource::Project);
    assert_eq!(extensions[2].source, ExtensionSource::User);
}

// ============ Source Priority Tests ============

#[rstest]
#[case(ExtensionSource::Dev, 4)]
#[case(ExtensionSource::Project, 3)]
#[case(ExtensionSource::User, 2)]
#[case(ExtensionSource::Builtin, 1)]
fn test_extension_source_priority(#[case] source: ExtensionSource, #[case] expected: u8) {
    assert_eq!(source.priority(), expected);
}

#[test]
fn test_extension_source_display() {
    assert_eq!(format!("{}", ExtensionSource::Dev), "dev");
    assert_eq!(format!("{}", ExtensionSource::Project), "project");
    assert_eq!(format!("{}", ExtensionSource::User), "user");
    assert_eq!(format!("{}", ExtensionSource::Builtin), "builtin");
}

// ============ Directory Accessor Tests ============

#[tokio::test]
async fn test_directory_accessors() {
    let temp_dir = TempDir::new().unwrap();
    let user_dir = temp_dir.path().join("extensions");
    let dev_dir = temp_dir.path().join("extensions-dev");
    let project_dir = temp_dir
        .path()
        .join("project")
        .join(".vx")
        .join("extensions");

    let discovery =
        ExtensionDiscovery::with_dirs(user_dir.clone(), dev_dir.clone(), Some(project_dir.clone()));

    assert_eq!(discovery.user_extensions_dir(), user_dir);
    assert_eq!(discovery.dev_extensions_dir(), dev_dir);
    assert_eq!(
        discovery.project_extensions_dir(),
        Some(project_dir.as_path())
    );
}

#[tokio::test]
async fn test_directory_accessors_no_project() {
    let temp_dir = TempDir::new().unwrap();
    let user_dir = temp_dir.path().join("extensions");
    let dev_dir = temp_dir.path().join("extensions-dev");

    let discovery = ExtensionDiscovery::with_dirs(user_dir, dev_dir, None);

    assert!(discovery.project_extensions_dir().is_none());
}
