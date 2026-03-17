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
fn discovery_runtime_filter_limits_output() {
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

    let mut config = DiscoveryConfig::new(&root, 2);
    config.runtime_filter = BTreeSet::from(["npm".to_string(), "npx".to_string()]);

    let result = discover_providers(&config).expect("discovery should succeed");

    assert_eq!(result.testable_runtimes, 2);
    assert_eq!(result.linux.runtimes, vec!["npm", "npx"]);
    assert_eq!(result.linux.matrix, vec!["npm,npx"]);

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
