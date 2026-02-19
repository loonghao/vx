//! Provider loading and metadata tests for vx-starlark

use vx_starlark::StarlarkProvider;
use vx_starlark::provider::{
    ProviderFormat, has_starlark_provider, has_toml_provider, is_starlark_provider,
};

// ============================================================
// ProviderFormat detection tests
// ============================================================

#[test]
fn test_provider_format_none_for_empty_dir() {
    let temp = tempfile::tempdir().unwrap();
    assert_eq!(ProviderFormat::detect(temp.path()), ProviderFormat::None);
}

#[test]
fn test_provider_format_starlark_when_star_exists() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("provider.star"), "# test").unwrap();
    assert_eq!(
        ProviderFormat::detect(temp.path()),
        ProviderFormat::Starlark
    );
}

#[test]
fn test_provider_format_toml_when_only_toml_exists() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("provider.toml"), "# test").unwrap();
    assert_eq!(ProviderFormat::detect(temp.path()), ProviderFormat::Toml);
}

#[test]
fn test_provider_format_starlark_takes_priority_over_toml() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("provider.star"), "# test").unwrap();
    std::fs::write(temp.path().join("provider.toml"), "# test").unwrap();
    // Starlark takes priority (RFC: provider.star > provider.toml)
    assert_eq!(
        ProviderFormat::detect(temp.path()),
        ProviderFormat::Starlark
    );
}

#[test]
fn test_provider_format_filename() {
    assert_eq!(ProviderFormat::Starlark.filename(), Some("provider.star"));
    assert_eq!(ProviderFormat::Toml.filename(), Some("provider.toml"));
    assert_eq!(ProviderFormat::None.filename(), None);
}

#[test]
fn test_is_starlark_provider_by_extension() {
    assert!(is_starlark_provider(std::path::Path::new("provider.star")));
    assert!(is_starlark_provider(std::path::Path::new("my_tool.star")));
    assert!(!is_starlark_provider(std::path::Path::new("provider.toml")));
    assert!(!is_starlark_provider(std::path::Path::new("provider.py")));
}

#[test]
fn test_has_starlark_provider_checks_dir() {
    let temp = tempfile::tempdir().unwrap();
    assert!(!has_starlark_provider(temp.path()));

    std::fs::write(temp.path().join("provider.star"), "# test").unwrap();
    assert!(has_starlark_provider(temp.path()));
}

#[test]
fn test_has_toml_provider_checks_dir() {
    let temp = tempfile::tempdir().unwrap();
    assert!(!has_toml_provider(temp.path()));

    std::fs::write(temp.path().join("provider.toml"), "[provider]").unwrap();
    assert!(has_toml_provider(temp.path()));
}

// ============================================================
// StarlarkProvider loading tests
// ============================================================

#[tokio::test]
async fn test_load_returns_error_for_missing_file() {
    let result = StarlarkProvider::load("/nonexistent/path/provider.star").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_load_minimal_provider() {
    let temp = tempfile::tempdir().unwrap();
    let star_path = temp.path().join("provider.star");

    std::fs::write(
        &star_path,
        r#"
def name():
    return "test-provider"

def description():
    return "A test provider for unit tests"
"#,
    )
    .unwrap();

    let provider = StarlarkProvider::load(&star_path).await.unwrap();
    // Metadata parsing is currently simplified; just verify it loads without error
    assert!(!provider.script_path().to_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_load_with_sandbox_config() {
    use vx_starlark::SandboxConfig;

    let temp = tempfile::tempdir().unwrap();
    let star_path = temp.path().join("provider.star");
    std::fs::write(&star_path, "# minimal provider").unwrap();

    let sandbox = SandboxConfig::restrictive();
    let provider = StarlarkProvider::load_with_sandbox(&star_path, sandbox)
        .await
        .unwrap();

    assert!(!provider.script_path().to_str().unwrap().is_empty());
}

// ============================================================
// ProviderMeta tests
// ============================================================

#[test]
fn test_provider_meta_defaults() {
    use vx_starlark::provider::ProviderMeta;

    let meta = ProviderMeta {
        name: "test".to_string(),
        description: "Test provider".to_string(),
        version: "1.0.0".to_string(),
        homepage: None,
        repository: None,
        platforms: None,
    };

    assert_eq!(meta.name, "test");
    assert_eq!(meta.version, "1.0.0");
    assert!(meta.homepage.is_none());
    assert!(meta.platforms.is_none());
}

#[test]
fn test_provider_meta_with_platforms() {
    use std::collections::HashMap;
    use vx_starlark::provider::ProviderMeta;

    let mut platforms = HashMap::new();
    platforms.insert("os".to_string(), vec!["windows".to_string()]);

    let meta = ProviderMeta {
        name: "msvc".to_string(),
        description: "MSVC compiler".to_string(),
        version: "1.0.0".to_string(),
        homepage: None,
        repository: None,
        platforms: Some(platforms),
    };

    let platforms = meta.platforms.unwrap();
    assert_eq!(platforms["os"], vec!["windows"]);
}
