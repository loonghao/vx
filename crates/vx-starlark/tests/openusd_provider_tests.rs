//! Provider packaging and runtime contracts.

use std::path::PathBuf;

use serde_json::{Value, json};

use vx_starlark::{ProviderContext, StarMetadata, StarlarkEngine};

fn script(name: &str) -> (PathBuf, String) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(format!("vx-providers/{name}/provider.star"));
    let content = std::fs::read_to_string(&path).unwrap();
    (path, content)
}

fn call(name: &str, os: &str, arch: &str, function: &str, args: &[Value]) -> Value {
    let (path, content) = script(name);
    let mut ctx = ProviderContext::new(name, std::env::temp_dir().join("dcc host with spaces"))
        .with_runtime_name(name);
    ctx.platform.os = os.to_owned();
    ctx.platform.arch = arch.to_owned();
    StarlarkEngine::new()
        .call_function(&path, &content, function, &ctx, args)
        .unwrap()
}

#[test]
fn test_openusd_routes_python_through_the_native_wheel() {
    let (_, content) = script("openusd");
    let metadata = StarMetadata::parse(&content);
    let alias = metadata.package_alias.unwrap();
    assert_eq!(alias.ecosystem, "uvx");
    assert_eq!(alias.package, "usd-core");
    assert_eq!(alias.executable.as_deref(), Some("python"));
    let versions = call("openusd", "linux", "x64", "fetch_versions", &[]);
    assert_eq!(versions["url"], "https://pypi.org/pypi/usd-core/json");
    assert!(call("openusd", "linux", "x64", "download_url", &[json!("26.8")]).is_null());
}
