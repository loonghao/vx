//! Runtime trait tests

use async_trait::async_trait;
use vx_runtime::{mock_context, Ecosystem, Platform, Runtime, RuntimeContext, VersionInfo};

/// Test runtime implementation
struct TestRuntime {
    name: &'static str,
    ecosystem: Ecosystem,
    aliases: &'static [&'static str],
}

impl TestRuntime {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            ecosystem: Ecosystem::Unknown,
            aliases: &[],
        }
    }

    fn with_ecosystem(mut self, ecosystem: Ecosystem) -> Self {
        self.ecosystem = ecosystem;
        self
    }

    fn with_aliases(mut self, aliases: &'static [&'static str]) -> Self {
        self.aliases = aliases;
        self
    }
}

#[async_trait]
impl Runtime for TestRuntime {
    fn name(&self) -> &str {
        self.name
    }

    fn ecosystem(&self) -> Ecosystem {
        self.ecosystem.clone()
    }

    fn aliases(&self) -> &[&str] {
        self.aliases
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> anyhow::Result<Vec<VersionInfo>> {
        Ok(vec![VersionInfo::new("2.0.0"), VersionInfo::new("1.0.0")])
    }
}

#[test]
fn test_runtime_name() {
    let runtime = TestRuntime::new("test-runtime");
    assert_eq!(runtime.name(), "test-runtime");
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = TestRuntime::new("node").with_ecosystem(Ecosystem::NodeJs);
    assert_eq!(runtime.ecosystem(), Ecosystem::NodeJs);
}

#[test]
fn test_runtime_aliases() {
    let runtime = TestRuntime::new("node").with_aliases(&["nodejs"]);
    assert_eq!(runtime.aliases(), &["nodejs"]);
}

#[tokio::test]
async fn test_fetch_versions() {
    let ctx = mock_context();
    let runtime = TestRuntime::new("test");

    let versions = runtime.fetch_versions(&ctx).await.unwrap();

    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0].version, "2.0.0");
    assert_eq!(versions[1].version, "1.0.0");
}

#[tokio::test]
async fn test_is_installed_false() {
    let ctx = mock_context();
    let runtime = TestRuntime::new("test");

    let installed = runtime.is_installed("1.0.0", &ctx).await.unwrap();
    assert!(!installed);
}

#[tokio::test]
async fn test_installed_versions_empty() {
    let ctx = mock_context();
    let runtime = TestRuntime::new("test");

    let versions = runtime.installed_versions(&ctx).await.unwrap();
    assert!(versions.is_empty());
}

#[test]
fn test_platform_current() {
    let platform = Platform::current();

    // Should detect something
    assert!(!platform.as_str().is_empty());
}

#[test]
fn test_ecosystem_contains() {
    assert!(Ecosystem::NodeJs.contains("node"));
    assert!(Ecosystem::NodeJs.contains("npm"));
    assert!(Ecosystem::NodeJs.contains("npx"));
    assert!(!Ecosystem::NodeJs.contains("go"));

    assert!(Ecosystem::Python.contains("uv"));
    assert!(Ecosystem::Python.contains("pip"));
    assert!(!Ecosystem::Python.contains("node"));

    assert!(Ecosystem::Go.contains("go"));
    assert!(Ecosystem::Go.contains("gofmt"));
    assert!(!Ecosystem::Go.contains("node"));
}

#[test]
fn test_ecosystem_primary_runtime() {
    assert_eq!(Ecosystem::NodeJs.primary_runtime(), Some("node"));
    assert_eq!(Ecosystem::Python.primary_runtime(), Some("uv"));
    assert_eq!(Ecosystem::Go.primary_runtime(), Some("go"));
    assert_eq!(Ecosystem::Unknown.primary_runtime(), None);
}
