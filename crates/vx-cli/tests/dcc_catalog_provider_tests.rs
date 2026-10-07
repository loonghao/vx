//! Provider packaging and runtime contracts.

use std::path::PathBuf;

use serde_json::Value;

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

#[rstest::rstest]
#[case("blender")]
#[case("freecad")]
#[case("gimp")]
#[case("godot")]
#[case("krita")]
#[case("material-maker")]
#[case("openscad")]
#[case("openusd")]
#[case("renderdoc")]
#[case("tiled")]
#[case("comfyui")]
#[case("obs")]
#[case("kdenlive")]
#[case("openscreen")]
#[case("tracy")]
fn test_every_catalog_host_is_discoverable(#[case] name: &str) {
    let (_, content) = script(name);
    let metadata = StarMetadata::parse(&content);
    assert_eq!(metadata.name.as_deref(), Some(name));
    assert!(
        metadata
            .runtimes
            .iter()
            .any(|r| r.name.as_deref() == Some(name))
    );
    // Evaluate with the actual stdlib as well as the lightweight embedding parser.
    assert!(!call(name, "windows", "x64", "store_root", &[]).is_null());
}
