use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use vx_star_metadata::{DiscoveryConfig, discover_providers};

#[test]
fn discovery_uses_provider_and_runtime_platform_constraints() {
    let root = create_temp_dir();

    write_provider(
        &root,
        "provider-platforms",
        r#"
name = "provider-platforms"
platforms = {"os": ["windows", "macos"]}
"#,
    );

    write_provider(
        &root,
        "runtime-platforms",
        r#"
name = "runtime-platforms"
runtimes = [
    runtime_def("runtime-all"),
    runtime_def("runtime-linux", platform_constraint = {"os": ["linux"]}),
]
"#,
    );

    write_provider(
        &root,
        "supported-platforms",
        r#"
name = "supported-platforms"
def supported_platforms():
    return [
        {"os": "linux", "arch": "x64"},
        {"os": "macos", "arch": "arm64"},
    ]

runtimes = [runtime_def("runtime-supported")]
"#,
    );

    let mut config = DiscoveryConfig::new(&root, 2);
    config.skip_always = BTreeSet::from(["runtime-all".to_string()]);

    let result = discover_providers(&config).expect("discovery should succeed");

    assert_eq!(result.total_runtimes, 4);
    assert_eq!(result.testable_runtimes, 3);
    assert_eq!(
        result.linux.runtimes,
        vec!["runtime-linux", "runtime-supported"]
    );
    assert_eq!(
        result.macos.runtimes,
        vec!["provider-platforms", "runtime-supported"]
    );
    assert_eq!(result.windows.runtimes, vec!["provider-platforms"]);
    assert_eq!(result.linux.matrix, vec!["runtime-linux,runtime-supported"]);

    fs::remove_dir_all(root).ok();
}

#[test]
fn discovery_excludes_bundled_runtimes() {
    let root = create_temp_dir();

    write_provider(
        &root,
        "node",
        r#"
name = "node"
runtimes = [
    runtime_def("node"),
    bundled_runtime_def("npm", bundled_with = "node"),
    bundled_runtime_def("npx", bundled_with = "node"),
]
"#,
    );

    let config = DiscoveryConfig::new(&root, 10);

    let result = discover_providers(&config).expect("discovery should succeed");

    // Only "node" should be discovered; npm and npx are bundled and excluded
    assert_eq!(result.total_runtimes, 1);
    assert_eq!(result.testable_runtimes, 1);
    assert_eq!(result.linux.runtimes, vec!["node"]);

    fs::remove_dir_all(root).ok();
}

#[test]
fn discovery_runtime_filter_on_bundled_returns_empty() {
    let root = create_temp_dir();

    write_provider(
        &root,
        "node",
        r#"
name = "node"
runtimes = [
    runtime_def("node"),
    bundled_runtime_def("npm", bundled_with = "node"),
    bundled_runtime_def("npx", bundled_with = "node"),
]
"#,
    );

    // Filtering for bundled runtimes should return empty since they are excluded
    let mut config = DiscoveryConfig::new(&root, 2);
    config.runtime_filter = BTreeSet::from(["npm".to_string(), "npx".to_string()]);

    let result = discover_providers(&config).expect("discovery should succeed");

    assert_eq!(result.testable_runtimes, 0);
    assert!(result.linux.runtimes.is_empty());

    fs::remove_dir_all(root).ok();
}

fn create_temp_dir() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("vx-star-discovery-tests-{unique}"));
    fs::create_dir_all(&dir).expect("temp dir should be created");
    dir
}

fn write_provider(root: &Path, name: &str, source: &str) {
    let provider_dir = root.join(name);
    fs::create_dir_all(&provider_dir).expect("provider dir should be created");
    fs::write(provider_dir.join("provider.star"), source).expect("provider.star should be written");
}

#[test]
fn discovery_excludes_bundled_runtimes_positional_arg() {
    let root = create_temp_dir();

    // Use positional second arg form (as in real ffmpeg provider.star)
    write_provider(
        &root,
        "ffmpeg",
        r#"
name = "ffmpeg"
runtimes = [
    runtime_def("ffmpeg"),
    bundled_runtime_def("ffprobe", "ffmpeg",
        description = "FFmpeg media stream analyzer",
    ),
    bundled_runtime_def("ffplay", "ffmpeg",
        description = "FFmpeg media player",
    ),
]
"#,
    );

    let config = DiscoveryConfig::new(&root, 10);
    let result = discover_providers(&config).expect("discovery should succeed");

    // Only "ffmpeg" should be discovered; ffprobe and ffplay are bundled
    assert_eq!(result.total_runtimes, 1);
    assert_eq!(result.testable_runtimes, 1);
    assert_eq!(result.linux.runtimes, vec!["ffmpeg"]);

    fs::remove_dir_all(root).ok();
}

#[test]
fn discovery_excludes_package_alias_providers() {
    let root = create_temp_dir();

    // Regular provider (should be discovered)
    write_provider(
        &root,
        "ripgrep",
        r#"
name = "ripgrep"
runtimes = [runtime_def("rg", aliases = ["ripgrep"])]
"#,
    );

    // Package alias provider (should be excluded)
    write_provider(
        &root,
        "openclaw",
        r#"
name = "openclaw"
package_alias = {"ecosystem": "npm", "package": "openclaw"}
runtimes = [
    runtime_def("openclaw", aliases = ["claw"]),
    bundled_runtime_def("clawhub", bundled_with = "openclaw"),
]
"#,
    );

    // Another package alias provider
    write_provider(
        &root,
        "vite",
        r#"
name = "vite"
package_alias = {"ecosystem": "npm", "package": "vite"}
runtimes = [runtime_def("vite")]
"#,
    );

    let config = DiscoveryConfig::new(&root, 10);
    let result = discover_providers(&config).expect("discovery should succeed");

    // Only "rg" should be discovered; openclaw and vite are package_alias providers
    assert_eq!(result.total_runtimes, 1);
    assert_eq!(result.testable_runtimes, 1);
    assert_eq!(result.linux.runtimes, vec!["rg"]);
    assert!(
        !result.linux.runtimes.contains(&"openclaw".to_string()),
        "openclaw should NOT appear (package_alias)"
    );
    assert!(
        !result.linux.runtimes.contains(&"vite".to_string()),
        "vite should NOT appear (package_alias)"
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn discovery_multiline_platforms_excludes_from_other_os() {
    let root = create_temp_dir();

    // Multi-line platforms dict (as in real rcedit provider.star)
    write_provider(
        &root,
        "rcedit",
        r#"
name = "rcedit"

platforms = {
    "os": ["windows"],
}

runtimes = [runtime_def("rcedit")]
"#,
    );

    let config = DiscoveryConfig::new(&root, 10);
    let result = discover_providers(&config).expect("discovery should succeed");

    // rcedit should only appear for windows, not linux or macos
    assert!(
        result.linux.runtimes.is_empty(),
        "rcedit should NOT appear in Linux: {:?}",
        result.linux.runtimes
    );
    assert!(
        result.macos.runtimes.is_empty(),
        "rcedit should NOT appear in macOS: {:?}",
        result.macos.runtimes
    );
    assert_eq!(result.windows.runtimes, vec!["rcedit"]);

    fs::remove_dir_all(root).ok();
}
