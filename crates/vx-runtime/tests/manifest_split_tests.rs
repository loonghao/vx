//! Tests for manifest parsing and platform constraint logic
//!
//! Updated from RFC 0029 Phase 2 tests to use the current vx-manifest API.

use std::sync::Arc;

use async_trait::async_trait;
use vx_manifest::{PlatformConstraint, ProviderManifest};
use vx_runtime::{Provider, Runtime, RuntimeContext, VersionInfo};

// ========== Test helpers ==========

#[allow(dead_code)]
struct DummyRuntime {
    name: &'static str,
}

#[async_trait]
impl Runtime for DummyRuntime {
    fn name(&self) -> &str {
        self.name
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> anyhow::Result<Vec<VersionInfo>> {
        Ok(vec![VersionInfo::new("1.0.0")])
    }
}

#[allow(dead_code)]
struct DummyProvider {
    name: &'static str,
}

impl Provider for DummyProvider {
    fn name(&self) -> &str {
        self.name
    }

    fn description(&self) -> &str {
        "Dummy provider for testing"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(DummyRuntime { name: self.name })]
    }
}

fn parse_manifest(toml_content: &str) -> ProviderManifest {
    ProviderManifest::parse(toml_content).expect("failed to parse manifest TOML")
}

// ========== ProviderManifest parsing tests ==========

#[test]
fn manifest_parse_basic() {
    let manifest = parse_manifest(
        r#"
[provider]
name = "test-tool"

[[runtimes]]
name = "test-tool"
executable = "test-tool"
"#,
    );

    assert_eq!(manifest.provider.name, "test-tool");
    assert_eq!(manifest.runtimes.len(), 1);
    assert_eq!(manifest.runtimes[0].name, "test-tool");
    assert_eq!(manifest.runtimes[0].executable, "test-tool");
}

#[test]
fn manifest_parse_with_aliases() {
    let manifest = parse_manifest(
        r#"
[provider]
name = "test"

[[runtimes]]
name = "test-runtime"
executable = "test-bin"
aliases = ["tr", "test-rt"]
"#,
    );

    assert_eq!(manifest.runtimes[0].aliases, vec!["tr", "test-rt"]);
}

#[test]
fn manifest_parse_with_platform_constraint() {
    let manifest = parse_manifest(
        r#"
[provider]
name = "tool"

[provider.platforms]
os = ["windows", "linux"]

[[runtimes]]
name = "win-tool"
executable = "win-tool"
platform_constraint = { os = ["windows"] }

[[runtimes]]
name = "any-tool"
executable = "any-tool"
"#,
    );

    assert_eq!(manifest.runtimes.len(), 2);

    let win_tool = manifest
        .runtimes
        .iter()
        .find(|r| r.name == "win-tool")
        .unwrap();
    let win_constraint = win_tool.platform_constraint.as_ref().unwrap();
    assert_eq!(win_constraint.os, vec![vx_manifest::Os::Windows]);

    let any_tool = manifest
        .runtimes
        .iter()
        .find(|r| r.name == "any-tool")
        .unwrap();
    assert!(any_tool.platform_constraint.is_none());
}

#[test]
fn manifest_parse_multiple_runtimes() {
    let manifest = parse_manifest(
        r#"
[provider]
name = "node"

[[runtimes]]
name = "node"
executable = "node"
aliases = ["nodejs"]

[[runtimes]]
name = "npm"
executable = "npm"
"#,
    );

    assert_eq!(manifest.runtimes.len(), 2);
    assert!(manifest.runtimes.iter().any(|r| r.name == "node"));
    assert!(manifest.runtimes.iter().any(|r| r.name == "npm"));
}

#[test]
fn manifest_parse_with_ecosystem() {
    let manifest = parse_manifest(
        r#"
[provider]
name = "my-provider"
description = "A test provider"
ecosystem = "python"

[[runtimes]]
name = "tool"
executable = "tool"
"#,
    );

    assert_eq!(
        manifest.provider.ecosystem,
        Some(vx_manifest::Ecosystem::Python)
    );
    assert_eq!(
        manifest.provider.description.as_deref(),
        Some("A test provider")
    );
}

// ========== PlatformConstraint::intersect tests ==========

#[test]
fn platform_intersect_both_empty() {
    let a = PlatformConstraint::new();
    let b = PlatformConstraint::new();
    let result = a.intersect(&b);
    assert!(result.is_empty());
}

#[test]
fn platform_intersect_one_empty() {
    let a = PlatformConstraint::windows_only();
    let b = PlatformConstraint::new();
    let result = a.intersect(&b);
    assert_eq!(result.os, vec![vx_manifest::Os::Windows]);
}

#[test]
fn platform_intersect_overlapping_os() {
    use vx_manifest::Os;
    let a = PlatformConstraint {
        os: vec![Os::Windows, Os::Linux],
        ..Default::default()
    };
    let b = PlatformConstraint {
        os: vec![Os::Linux, Os::MacOS],
        ..Default::default()
    };
    let result = a.intersect(&b);
    assert_eq!(result.os, vec![Os::Linux]);
}

#[test]
fn platform_intersect_disjoint_os() {
    let a = PlatformConstraint::windows_only();
    let b = PlatformConstraint::linux_only();
    let result = a.intersect(&b);
    assert!(result.os.is_empty());
}

#[test]
fn platform_intersect_excludes_union() {
    use vx_manifest::{Os, PlatformExclusion};
    let a = PlatformConstraint {
        exclude: vec![PlatformExclusion {
            os: Some(Os::Windows),
            arch: None,
        }],
        ..Default::default()
    };
    let b = PlatformConstraint {
        exclude: vec![PlatformExclusion {
            os: Some(Os::Linux),
            arch: None,
        }],
        ..Default::default()
    };
    let result = a.intersect(&b);
    assert_eq!(result.exclude.len(), 2);
}

// ========== Manifest constraints tests ==========

#[test]
fn manifest_parse_constraints() {
    let manifest = parse_manifest(
        r#"
[provider]
name = "test-provider"

[[runtimes]]
name = "yarn"
executable = "yarn"

[[runtimes.constraints]]
when = "^1"
requires = [
  { runtime = "node", version = ">=12, <23", recommended = "20", reason = "Yarn 1.x requires Node.js 12-22" }
]
"#,
    );

    let yarn = manifest.runtimes.iter().find(|r| r.name == "yarn").unwrap();
    assert_eq!(yarn.constraints.len(), 1);
    assert_eq!(yarn.constraints[0].when, "^1");
    assert_eq!(yarn.constraints[0].requires.len(), 1);
    assert_eq!(yarn.constraints[0].requires[0].runtime, "node");
}
