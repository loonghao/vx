//! Narrow provider-script contract helpers shared by independent Provider tests.

use std::path::PathBuf;

use serde_json::{Value, json};

use vx_starlark::{ProviderContext, StarlarkEngine};

pub fn source(provider: &str) -> (PathBuf, String) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("vx-providers")
        .join(provider)
        .join("provider.star");
    let content = std::fs::read_to_string(&path).unwrap();
    (path, content)
}

pub fn call(provider: &str, function: &str, os: &str, arch: &str, version: &str) -> Value {
    call_runtime(provider, function, os, arch, version, None)
}

pub fn call_runtime(
    provider: &str,
    function: &str,
    os: &str,
    arch: &str,
    version: &str,
    runtime: Option<&str>,
) -> Value {
    let (path, content) = source(provider);
    let mut ctx = ProviderContext::new(provider, std::env::temp_dir().join("vx-dcc-contract"))
        .with_version(version);
    ctx.platform.os = os.to_owned();
    ctx.platform.arch = arch.to_owned();
    if let Some(runtime) = runtime {
        ctx = ctx.with_runtime_name(runtime);
    }
    let args = if function == "fetch_versions" {
        vec![]
    } else {
        vec![json!(version)]
    };
    StarlarkEngine::new()
        .call_function(&path, &content, function, &ctx, &args)
        .unwrap_or_else(|error| panic!("{provider}.{function}({os}/{arch}): {error}"))
}
