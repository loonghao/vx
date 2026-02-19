//! Tests for provider manifest parsing

use rstest::rstest;
use vx_manifest::{Ecosystem, ProviderManifest, VersionRequest};

// Minimal inline TOML fixtures (provider.toml files have been migrated to Starlark)

const YARN_TOML: &str = r#"
[provider]
name = "yarn"
description = "Yarn package manager"
ecosystem = "nodejs"

[[runtimes]]
name = "yarn"
description = "Yarn package manager"
executable = "yarn"
aliases = ["yarnpkg"]
priority = 80
auto_installable = true

[runtimes.detection]
command = "{executable} --version"
pattern = "([\\d.]+)"
system_paths = ["/usr/local/bin/yarn", "/usr/bin/yarn"]
env_hints = ["YARN_HOME"]

[runtimes.health]
check_command = "{executable} --version"
exit_code = 0
check_on = ["install"]

[[runtimes.mirrors]]
name = "taobao"
url = "https://registry.npmmirror.com"
region = "cn"

[runtimes.cache]
versions_ttl = 3600
cache_downloads = true

[[runtimes.constraints]]
when = "<2"
requires = [
    { runtime = "node", version = ">=12, <23", recommended = "20", reason = "Yarn 1.x requires Node.js" }
]

[[runtimes.constraints]]
when = ">=2, <3"
requires = [
    { runtime = "node", version = ">=14", recommended = "20", reason = "Yarn 2.x requires Node.js" }
]

[[runtimes.constraints]]
when = ">=4"
requires = [
    { runtime = "node", version = ">=18", recommended = "22", reason = "Yarn 4.x requires Node.js" }
]
"#;

const PNPM_TOML: &str = r#"
[provider]
name = "pnpm"
description = "Fast, disk space efficient package manager"
ecosystem = "nodejs"

[[runtimes]]
name = "pnpm"
description = "pnpm package manager"
executable = "pnpm"
priority = 85
auto_installable = true

[runtimes.detection]
command = "{executable} --version"
pattern = "([\\d.]+)"
system_paths = ["/usr/local/bin/pnpm"]

[runtimes.health]
check_command = "{executable} --version"
exit_code = 0
check_on = ["install"]

[[runtimes.mirrors]]
name = "taobao"
url = "https://registry.npmmirror.com"
region = "cn"

[runtimes.cache]
versions_ttl = 3600
cache_downloads = true

[[runtimes.constraints]]
when = ">=7, <8"
requires = [
    { runtime = "node", version = ">=14", reason = "pnpm 7.x requires Node.js >=14" }
]

[[runtimes.constraints]]
when = ">=8, <9"
requires = [
    { runtime = "node", version = ">=16", reason = "pnpm 8.x requires Node.js >=16" }
]

[[runtimes.constraints]]
when = ">=9"
requires = [
    { runtime = "node", version = ">=18", reason = "pnpm 9.x requires Node.js >=18" }
]
"#;

const NODE_TOML: &str = r#"
[provider]
name = "node"
description = "Node.js JavaScript runtime"
ecosystem = "nodejs"

[[runtimes]]
name = "node"
description = "Node.js runtime"
executable = "node"
priority = 100
auto_installable = true

[runtimes.detection]
command = "{executable} --version"
pattern = "v([\\d.]+)"
system_paths = ["/usr/local/bin/node", "/usr/bin/node"]
env_hints = ["NODE_HOME"]

[runtimes.health]
check_command = "{executable} --version"
exit_code = 0
check_on = ["install"]

[runtimes.env]
vars = { PATH = "{install_dir}/bin:$PATH" }

[[runtimes.env.conditional]]
when = { os = "windows" }
vars = { PATH = "{install_dir};$PATH" }

[[runtimes.mirrors]]
name = "taobao"
url = "https://npmmirror.com/mirrors/node"
region = "cn"

[runtimes.cache]
versions_ttl = 3600
cache_downloads = true

[runtimes.hooks]
post_install = ["node --version"]
post_activate = ["node --version"]

[[runtimes]]
name = "npm"
description = "Node.js package manager"
executable = "npm"
bundled_with = "node"
auto_installable = false

[[runtimes]]
name = "npx"
description = "Node.js package runner"
executable = "npx"
bundled_with = "node"
auto_installable = false
"#;

const PYTHON_TOML: &str = r#"
[provider]
name = "python"
description = "Python programming language"
ecosystem = "python"

[[runtimes]]
name = "python"
description = "Python interpreter"
executable = "python"
priority = 100
auto_installable = true

[runtimes.detection]
command = "{executable} --version"
pattern = "Python ([\\d.]+)"
system_paths = ["/usr/bin/python3", "/usr/local/bin/python3"]

[runtimes.health]
check_command = "{executable} --version"
exit_code = 0
check_on = ["install"]

[runtimes.env]
vars = { PYTHONHOME = "{install_dir}" }

[[runtimes.mirrors]]
name = "taobao"
url = "https://npmmirror.com/mirrors/python"
region = "cn"

[runtimes.cache]
versions_ttl = 3600
cache_downloads = true
"#;

#[test]
fn test_parse_yarn_manifest() {
    let manifest = ProviderManifest::parse(YARN_TOML).expect("Failed to parse yarn manifest");

    assert_eq!(manifest.provider.name, "yarn");
    assert_eq!(manifest.provider.ecosystem, Some(Ecosystem::NodeJs));
    assert_eq!(manifest.runtimes.len(), 1);

    let runtime = &manifest.runtimes[0];
    assert_eq!(runtime.name, "yarn");
    assert_eq!(runtime.executable, "yarn");
    assert_eq!(runtime.aliases, vec!["yarnpkg"]);
    assert_eq!(runtime.constraints.len(), 3);
}

#[test]
fn test_parse_pnpm_manifest() {
    let manifest = ProviderManifest::parse(PNPM_TOML).expect("Failed to parse pnpm manifest");

    assert_eq!(manifest.provider.name, "pnpm");
    assert_eq!(manifest.provider.ecosystem, Some(Ecosystem::NodeJs));
}

#[test]
fn test_parse_node_manifest() {
    let manifest = ProviderManifest::parse(NODE_TOML).expect("Failed to parse node manifest");

    assert_eq!(manifest.provider.name, "node");
    assert_eq!(manifest.provider.ecosystem, Some(Ecosystem::NodeJs));
    // node, npm, npx
    assert_eq!(manifest.runtimes.len(), 3);
}

/// RFC 0018: Test extended schema fields in node provider
#[test]
fn test_parse_node_manifest_extended_fields() {
    let manifest = ProviderManifest::parse(NODE_TOML).expect("Failed to parse node manifest");

    let node_runtime = &manifest.runtimes[0];
    assert_eq!(node_runtime.name, "node");

    // RFC 0018: priority and auto_installable
    assert_eq!(node_runtime.priority, Some(100));
    assert_eq!(node_runtime.auto_installable, Some(true));

    // RFC 0018: detection config
    let detection = node_runtime
        .detection
        .as_ref()
        .expect("detection should exist");
    assert_eq!(detection.command, "{executable} --version");
    assert!(!detection.pattern.is_empty());
    assert!(!detection.system_paths.is_empty());
    assert!(detection.env_hints.contains(&"NODE_HOME".to_string()));

    // RFC 0018: health config
    let health = node_runtime.health.as_ref().expect("health should exist");
    assert!(!health.check_command.is_empty());
    assert_eq!(health.exit_code, Some(0));
    assert!(health.check_on.contains(&"install".to_string()));

    // RFC 0018: env_config
    let env_config = node_runtime
        .env_config
        .as_ref()
        .expect("env_config should exist");
    assert!(env_config.vars.contains_key("PATH"));
    assert!(!env_config.conditional.is_empty());

    // RFC 0018: mirrors
    assert!(!node_runtime.mirrors.is_empty());
    let taobao_mirror = node_runtime.mirrors.iter().find(|m| m.name == "taobao");
    assert!(taobao_mirror.is_some());
    assert_eq!(taobao_mirror.unwrap().region, Some("cn".to_string()));

    // RFC 0018: cache config
    let cache = node_runtime.cache.as_ref().expect("cache should exist");
    assert_eq!(cache.versions_ttl, 3600);
    assert!(cache.cache_downloads);

    // RFC 0018: hooks
    let hooks = node_runtime.hooks.as_ref().expect("hooks should exist");
    assert!(!hooks.post_install.is_empty());
    assert!(!hooks.post_activate.is_empty());
}

/// RFC 0018: Test extended schema fields in yarn provider
#[test]
fn test_parse_yarn_manifest_extended_fields() {
    let manifest = ProviderManifest::parse(YARN_TOML).expect("Failed to parse yarn manifest");

    let yarn_runtime = &manifest.runtimes[0];
    assert_eq!(yarn_runtime.name, "yarn");

    // RFC 0018: priority and auto_installable
    assert_eq!(yarn_runtime.priority, Some(80));
    assert_eq!(yarn_runtime.auto_installable, Some(true));

    // RFC 0018: detection config
    assert!(yarn_runtime.detection.is_some());

    // RFC 0018: health config
    assert!(yarn_runtime.health.is_some());

    // RFC 0018: mirrors
    assert!(!yarn_runtime.mirrors.is_empty());

    // RFC 0018: cache config
    assert!(yarn_runtime.cache.is_some());
}

/// RFC 0018: Test extended schema fields in python provider
#[test]
fn test_parse_python_manifest_extended_fields() {
    let manifest = ProviderManifest::parse(PYTHON_TOML).expect("Failed to parse python manifest");

    let python_runtime = &manifest.runtimes[0];
    assert_eq!(python_runtime.name, "python");

    // RFC 0018: priority and auto_installable
    assert_eq!(python_runtime.priority, Some(100));
    assert_eq!(python_runtime.auto_installable, Some(true));

    // RFC 0018: detection config
    let detection = python_runtime
        .detection
        .as_ref()
        .expect("detection should exist");
    assert!(detection.pattern.contains("Python"));

    // RFC 0018: health config
    assert!(python_runtime.health.is_some());

    // RFC 0018: env_config
    let env_config = python_runtime
        .env_config
        .as_ref()
        .expect("env_config should exist");
    assert!(env_config.vars.contains_key("PYTHONHOME"));

    // RFC 0018: mirrors
    assert!(!python_runtime.mirrors.is_empty());

    // RFC 0018: cache config
    assert!(python_runtime.cache.is_some());
}

/// RFC 0018: Test extended schema fields in pnpm provider
#[test]
fn test_parse_pnpm_manifest_extended_fields() {
    let manifest = ProviderManifest::parse(PNPM_TOML).expect("Failed to parse pnpm manifest");

    let pnpm_runtime = &manifest.runtimes[0];
    assert_eq!(pnpm_runtime.name, "pnpm");

    // RFC 0018: priority and auto_installable
    assert_eq!(pnpm_runtime.priority, Some(85));
    assert_eq!(pnpm_runtime.auto_installable, Some(true));

    // RFC 0018: detection config
    assert!(pnpm_runtime.detection.is_some());

    // RFC 0018: health config
    assert!(pnpm_runtime.health.is_some());

    // RFC 0018: mirrors
    assert!(!pnpm_runtime.mirrors.is_empty());

    // RFC 0018: cache config
    assert!(pnpm_runtime.cache.is_some());
}

#[rstest]
#[case("20.0.0", ">=12, <23", true)]
#[case("18.0.0", ">=12", true)]
#[case("20.0.0", "<23", true)]
#[case("18.0.0", ">=20", false)]
#[case("18.0.0", ">=18", true)]
#[case("16.0.0", ">=20", false)]
fn test_constraint_matching(
    #[case] version: &str,
    #[case] constraint: &str,
    #[case] expected: bool,
) {
    let req = VersionRequest::parse(constraint);
    assert_eq!(
        req.satisfies(version),
        expected,
        "Version {} should {} satisfy {}",
        version,
        if expected { "" } else { "not" },
        constraint
    );
}

#[test]
fn test_yarn_v1_constraints() {
    let manifest = ProviderManifest::parse(YARN_TOML).unwrap();
    let runtime = manifest.get_runtime("yarn").unwrap();

    // Test Yarn 1.x constraints
    let deps = runtime.get_dependencies_for_version("1.22.22");
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].runtime, "node");
    assert_eq!(deps[0].version, ">=12, <23");
    assert_eq!(deps[0].recommended, Some("20".to_string()));

    // Node 20 should satisfy the constraint
    assert!(deps[0].satisfies("20.0.0"));
    // Node 10 should not satisfy
    assert!(!deps[0].satisfies("10.0.0"));
    // Node 23 should not satisfy
    assert!(!deps[0].satisfies("23.0.0"));
}

#[test]
fn test_yarn_v4_constraints() {
    let manifest = ProviderManifest::parse(YARN_TOML).unwrap();
    let runtime = manifest.get_runtime("yarn").unwrap();

    // Test Yarn 4.x constraints
    let deps = runtime.get_dependencies_for_version("4.0.0");
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].runtime, "node");
    assert_eq!(deps[0].version, ">=18");
    assert_eq!(deps[0].recommended, Some("22".to_string()));

    // Node 18 should satisfy
    assert!(deps[0].satisfies("18.0.0"));
    // Node 16 should not satisfy
    assert!(!deps[0].satisfies("16.0.0"));
}

#[test]
fn test_pnpm_constraints() {
    let manifest = ProviderManifest::parse(PNPM_TOML).unwrap();
    let runtime = manifest.get_runtime("pnpm").unwrap();

    // pnpm 7.x
    let deps_v7 = runtime.get_dependencies_for_version("7.33.0");
    assert_eq!(deps_v7.len(), 1);
    assert_eq!(deps_v7[0].version, ">=14");

    // pnpm 8.x
    let deps_v8 = runtime.get_dependencies_for_version("8.15.0");
    assert_eq!(deps_v8.len(), 1);
    assert_eq!(deps_v8[0].version, ">=16");

    // pnpm 9.x
    let deps_v9 = runtime.get_dependencies_for_version("9.0.0");
    assert_eq!(deps_v9.len(), 1);
    assert_eq!(deps_v9[0].version, ">=18");
}

#[test]
fn test_get_runtime_by_alias() {
    let manifest = ProviderManifest::parse(YARN_TOML).unwrap();

    // Should find by name
    assert!(manifest.get_runtime("yarn").is_some());
    // Should find by alias
    assert!(manifest.get_runtime("yarnpkg").is_some());
    // Should not find unknown
    assert!(manifest.get_runtime("unknown").is_none());
}

#[test]
fn test_ecosystem_parsing() {
    let toml = r#"
[provider]
name = "test"
ecosystem = "python"

[[runtimes]]
name = "test"
executable = "test"
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    assert_eq!(manifest.provider.ecosystem, Some(Ecosystem::Python));
}

#[test]
fn test_hooks_parsing() {
    let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "test"
executable = "test"

[runtimes.hooks]
pre_run = ["hook1", "hook2"]
post_install = ["hook3"]
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    let hooks = manifest.runtimes[0].hooks.as_ref().unwrap();
    assert_eq!(hooks.pre_run, vec!["hook1", "hook2"]);
    assert_eq!(hooks.post_install, vec!["hook3"]);
}

#[test]
fn test_platform_config_parsing() {
    let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "test"
executable = "test"

[runtimes.platforms.windows]
executable_extensions = [".cmd", ".exe"]

[runtimes.platforms.unix]
executable_extensions = []
"#;
    let manifest = ProviderManifest::parse(toml).unwrap();
    let platforms = manifest.runtimes[0].platforms.as_ref().unwrap();

    let windows = platforms.windows.as_ref().unwrap();
    assert_eq!(windows.executable_extensions, vec![".cmd", ".exe"]);

    let unix = platforms.unix.as_ref().unwrap();
    assert!(unix.executable_extensions.is_empty());
}

#[test]
fn test_validation_missing_provider_name() {
    let toml = r#"
[provider]
description = "test"

[[runtimes]]
name = "test"
executable = "test"
"#;
    let result = ProviderManifest::parse(toml);
    assert!(result.is_err());
}

#[test]
fn test_validation_missing_runtime_executable() {
    let toml = r#"
[provider]
name = "test"

[[runtimes]]
name = "test"
"#;
    let result = ProviderManifest::parse(toml);
    assert!(result.is_err());
}
