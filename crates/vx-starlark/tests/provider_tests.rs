//! Provider loading and metadata tests for vx-starlark
//!
//! All providers are now exclusively `provider.star` (Starlark).
//! There is no longer any `provider.toml` support in this crate.

use vx_starlark::StarlarkProvider;
use vx_starlark::provider::{has_starlark_provider, is_starlark_provider};

// ============================================================
// Starlark provider detection helpers
// ============================================================

#[test]
fn test_is_starlark_provider_by_extension() {
    assert!(is_starlark_provider(std::path::Path::new("provider.star")));
    assert!(is_starlark_provider(std::path::Path::new("my_tool.star")));
    assert!(!is_starlark_provider(std::path::Path::new("provider.toml")));
    assert!(!is_starlark_provider(std::path::Path::new("provider.py")));
    assert!(!is_starlark_provider(std::path::Path::new("provider")));
}

#[test]
fn test_has_starlark_provider_checks_dir() {
    let temp = tempfile::tempdir().unwrap();
    assert!(!has_starlark_provider(temp.path()));

    std::fs::write(temp.path().join("provider.star"), "# test").unwrap();
    assert!(has_starlark_provider(temp.path()));
}

#[test]
fn test_has_starlark_provider_ignores_toml() {
    let temp = tempfile::tempdir().unwrap();
    // A directory with only provider.toml is NOT a valid Starlark provider
    std::fs::write(temp.path().join("provider.toml"), "[provider]").unwrap();
    assert!(!has_starlark_provider(temp.path()));
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
name = "test-provider"
description = "A test provider for unit tests"

runtimes = [
    {"name": "test-provider", "executable": "test-provider"},
]
"#,
    )
    .unwrap();

    let provider = StarlarkProvider::load(&star_path).await.unwrap();
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

#[tokio::test]
async fn test_from_content_creates_provider() {
    let content = r#"
name = "inline-tool"
description = "Inline test provider"

runtimes = [
    {"name": "inline-tool", "executable": "inline-tool"},
]
"#;
    let provider = StarlarkProvider::from_content("inline-tool", content)
        .await
        .unwrap();

    assert!(!provider.script_path().to_str().unwrap().is_empty());
    assert_eq!(provider.name(), "inline-tool");
}

#[tokio::test]
async fn test_from_content_parses_runtimes() {
    let content = r#"
name = "node"
description = "Node.js runtime"

runtimes = [
    {"name": "node", "executable": "node", "aliases": ["nodejs"]},
    {"name": "npm",  "executable": "npm",  "bundled_with": "node"},
    {"name": "npx",  "executable": "npx",  "bundled_with": "node"},
]
"#;
    let provider = StarlarkProvider::from_content("node", content)
        .await
        .unwrap();

    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 3);
    assert_eq!(runtimes[0].name, "node");
    assert_eq!(runtimes[1].name, "npm");
    assert_eq!(runtimes[2].name, "npx");
}

#[tokio::test]
async fn test_from_content_parses_aliases() {
    let content = r#"
name = "node"
description = "Node.js runtime"

runtimes = [
    {"name": "node", "executable": "node", "aliases": ["nodejs", "node-js"]},
]
"#;
    let provider = StarlarkProvider::from_content("node", content)
        .await
        .unwrap();

    let runtimes = provider.runtimes();
    assert_eq!(runtimes[0].aliases, vec!["nodejs", "node-js"]);
}

#[tokio::test]
async fn test_deps_for_runtime_uses_runtime_name_context() {
    let content = r#"
name = "multi-tool"
description = "Multi runtime provider"

runtimes = [
    {"name": "foo", "executable": "foo"},
    {"name": "bar", "executable": "bar"},
]

def deps(ctx, _version):
    if ctx.runtime_name == "foo":
        return [{"runtime": "node", "version": ">=18", "reason": "foo requires node"}]
    if ctx.runtime_name == "bar":
        return [{"runtime": "uv", "reason": "bar requires uv"}]
    return [{"runtime": "git", "reason": "provider-level fallback"}]
"#;
    let provider = StarlarkProvider::from_content("multi-tool", content)
        .await
        .unwrap();

    let foo_deps = provider
        .deps_for_runtime("1.0.0", Some("foo"))
        .await
        .unwrap();
    let bar_deps = provider
        .deps_for_runtime("1.0.0", Some("bar"))
        .await
        .unwrap();
    let fallback_deps = provider.deps("1.0.0").await.unwrap();

    assert_eq!(
        foo_deps[0].get("runtime").and_then(|v| v.as_str()),
        Some("node")
    );
    assert_eq!(
        bar_deps[0].get("runtime").and_then(|v| v.as_str()),
        Some("uv")
    );
    assert_eq!(
        fallback_deps[0].get("runtime").and_then(|v| v.as_str()),
        Some("git")
    );
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
        package_alias: None,
        package_prefixes: vec![],
        vx_version_req: None,
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
        package_alias: None,
        package_prefixes: vec![],
        vx_version_req: None,
    };

    let platforms = meta.platforms.unwrap();
    assert_eq!(platforms["os"], vec!["windows"]);
}

// ============================================================
// Script hash / incremental cache tests
// ============================================================

#[tokio::test]
async fn test_same_content_produces_same_hash() {
    let content = r#"
name = "tool"
description = "Test"
runtimes = [{"name": "tool", "executable": "tool"}]
"#;
    let p1 = StarlarkProvider::from_content("tool", content)
        .await
        .unwrap();
    let p2 = StarlarkProvider::from_content("tool", content)
        .await
        .unwrap();

    assert_eq!(p1.script_hash(), p2.script_hash());
}

#[tokio::test]
async fn test_different_content_produces_different_hash() {
    let content_a = r#"name = "tool-a"
description = "A"
runtimes = [{"name": "tool-a", "executable": "tool-a"}]
"#;
    let content_b = r#"name = "tool-b"
description = "B"
runtimes = [{"name": "tool-b", "executable": "tool-b"}]
"#;
    let pa = StarlarkProvider::from_content("tool-a", content_a)
        .await
        .unwrap();
    let pb = StarlarkProvider::from_content("tool-b", content_b)
        .await
        .unwrap();

    assert_ne!(pa.script_hash(), pb.script_hash());
}

#[tokio::test]
async fn test_script_hash_hex_is_64_chars() {
    let content = r#"name = "tool"
description = "Test"
runtimes = [{"name": "tool", "executable": "tool"}]
"#;
    let provider = StarlarkProvider::from_content("tool", content)
        .await
        .unwrap();
    // SHA-256 hex = 64 characters
    assert_eq!(provider.script_hash_hex().len(), 64);
}

// ============================================================
// Package prefixes tests (RFC 0027)
// ============================================================

#[test]
fn test_provider_meta_with_package_prefixes() {
    use vx_starlark::provider::ProviderMeta;

    let meta = ProviderMeta {
        name: "deno".to_string(),
        description: "Deno runtime".to_string(),
        version: "1.0.0".to_string(),
        homepage: None,
        repository: None,
        platforms: None,
        package_alias: None,
        package_prefixes: vec!["deno".to_string()],
        vx_version_req: None,
    };

    assert_eq!(meta.package_prefixes, vec!["deno"]);
}

#[test]
fn test_provider_meta_package_prefixes_default_empty() {
    use vx_starlark::provider::ProviderMeta;

    let meta = ProviderMeta {
        name: "test".to_string(),
        description: "Test".to_string(),
        version: "1.0.0".to_string(),
        homepage: None,
        repository: None,
        platforms: None,
        package_alias: None,
        package_prefixes: vec![],
        vx_version_req: None,
    };

    assert!(meta.package_prefixes.is_empty());
}

#[tokio::test]
async fn test_package_prefixes_parsed_from_starlark() {
    let content = r#"
name = "deno"
description = "Deno runtime"
package_prefixes = ["deno", "deno-land"]

runtimes = [{"name": "deno", "executable": "deno"}]
"#;
    let provider = StarlarkProvider::from_content("deno", content)
        .await
        .unwrap();

    let meta = provider.meta();
    assert_eq!(meta.package_prefixes, vec!["deno", "deno-land"]);
}

#[tokio::test]
async fn test_package_prefixes_empty_when_not_declared() {
    let content = r#"
name = "node"
description = "Node.js runtime"

runtimes = [{"name": "node", "executable": "node"}]
"#;
    let provider = StarlarkProvider::from_content("node", content)
        .await
        .unwrap();

    let meta = provider.meta();
    assert!(meta.package_prefixes.is_empty());
}

#[tokio::test]
async fn test_install_layout_accepts_legacy_type_field() {
    use vx_starlark::provider::InstallLayout;

    let content = r#"
name = "legacy-layout"
description = "Legacy install_layout test"

runtimes = [{"name": "legacy-layout", "executable": "legacy-layout"}]

def install_layout(_ctx, _version):
    return {
        "type": "archive",
        "strip_prefix": "legacy-layout-1.0.0",
        "executable_paths": ["bin/legacy-layout"],
    }
"#;

    let provider = StarlarkProvider::from_content("legacy-layout", content)
        .await
        .unwrap();

    let layout = provider.install_layout("1.0.0").await.unwrap();
    match layout {
        Some(InstallLayout::Archive {
            strip_prefix,
            executable_paths,
            ..
        }) => {
            assert_eq!(strip_prefix.as_deref(), Some("legacy-layout-1.0.0"));
            assert_eq!(executable_paths, vec!["bin/legacy-layout"]);
        }
        other => panic!("unexpected install layout: {other:?}"),
    }
}

// ============================================================
// New provider tests (worktrunk, starship, sccache, cargo-nextest, cargo-deny)
// ============================================================

#[tokio::test]
async fn test_load_worktrunk_provider() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let provider_dir = manifest_dir
        .parent()  // crates/
        .unwrap()
        .join("vx-providers")
        .join("worktrunk");
    let star_path = provider_dir.join("provider.star");
    let provider = vx_starlark::StarlarkProvider::load(&star_path)
        .await
        .unwrap();
    assert_eq!(provider.name(), "worktrunk");
}

#[tokio::test]
async fn test_load_starship_provider() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let provider_dir = manifest_dir
        .parent()  // crates/
        .unwrap()
        .join("vx-providers")
        .join("starship");
    let star_path = provider_dir.join("provider.star");
    let provider = vx_starlark::StarlarkProvider::load(&star_path)
        .await
        .unwrap();
    assert_eq!(provider.name(), "starship");
}

#[tokio::test]
async fn test_load_sccache_provider() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let provider_dir = manifest_dir
        .parent()  // crates/
        .unwrap()
        .join("vx-providers")
        .join("sccache");
    let star_path = provider_dir.join("provider.star");
    let provider = vx_starlark::StarlarkProvider::load(&star_path)
        .await
        .unwrap();
    assert_eq!(provider.name(), "sccache");
}

#[tokio::test]
async fn test_load_cargo_nextest_provider() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let provider_dir = manifest_dir
        .parent()  // crates/
        .unwrap()
        .join("vx-providers")
        .join("cargo-nextest");
    let star_path = provider_dir.join("provider.star");
    let provider = vx_starlark::StarlarkProvider::load(&star_path)
        .await
        .unwrap();
    assert_eq!(provider.name(), "cargo-nextest");
}

#[tokio::test]
async fn test_load_cargo_deny_provider() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let provider_dir = manifest_dir
        .parent()  // crates/
        .unwrap()
        .join("vx-providers")
        .join("cargo-deny");
    let star_path = provider_dir.join("provider.star");
    let provider = vx_starlark::StarlarkProvider::load(&star_path)
        .await
        .unwrap();
    assert_eq!(provider.name(), "cargo-deny");
}
