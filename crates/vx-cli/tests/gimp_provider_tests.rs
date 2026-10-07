//! Provider packaging and runtime contracts.

use std::path::PathBuf;

use serde_json::{Value, json};

use vx_starlark::{ProviderContext, StarlarkEngine};

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
fn test_gimp_system_manager_routes_are_platform_scoped() {
    let (_, content) = script("gimp");
    // A downloaded installer is never treated as an already installed editor.
    assert!(call("gimp", "windows", "x64", "download_url", &[json!("latest")]).is_null());
    assert!(content.contains("GIMP.GIMP.3"));
    assert!(content.contains("platforms = [\"macos\"]"));
}
