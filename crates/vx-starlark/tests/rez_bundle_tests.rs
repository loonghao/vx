//! Rez bundle provider metadata, discovery, and platform selection tests.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use rstest::rstest;
use vx_starlark::context::PlatformInfo;
use vx_starlark::{ProviderContext, StarlarkEngine, StarlarkProvider};

const PROVIDER_SOURCE: &str = r#"
load("@vx//stdlib:rez.star", "rez_bundle_source")

name = "rust-rez-test"
description = "Rez bundle test provider"
runtimes = [{"name": "rust", "executable": "rustc"}]

_bundle = rez_bundle_source(
    "vx-org",
    "vx-rez-packages",
    "rust",
    targets = {
        "windows/x64": "x86_64-pc-windows-msvc",
        "windows/arm64": "aarch64-pc-windows-msvc",
        "linux/x64": "x86_64-unknown-linux-gnu",
        "linux/arm64": "aarch64-unknown-linux-gnu",
        "macos/x64": "x86_64-apple-darwin",
        "macos/arm64": "aarch64-apple-darwin",
    },
    unsupported_targets = {
        "windows/x86": "Rust 1.95 Rez bundles do not publish an official i686 MSVC target.",
    },
    programs = {"rust": "rustc"},
)

fetch_versions = _bundle["fetch_versions"]
download_url = _bundle["download_url"]
install_layout = _bundle["install_layout"]
rez_bundle = _bundle["rez_bundle"]
"#;

const RUST_PROVIDER_SOURCE: &str = include_str!("../../vx-providers/rust/provider.star");

fn windows_x64() -> PlatformInfo {
    PlatformInfo {
        os: "windows".to_string(),
        arch: "x64".to_string(),
        target: "x86_64-pc-windows-msvc".to_string(),
    }
}

fn windows_x86() -> PlatformInfo {
    PlatformInfo {
        os: "windows".to_string(),
        arch: "x86".to_string(),
        target: "i686-pc-windows-msvc".to_string(),
    }
}

fn windows_gnu() -> PlatformInfo {
    PlatformInfo {
        os: "windows".to_string(),
        arch: "x64".to_string(),
        target: "x86_64-pc-windows-gnu".to_string(),
    }
}

fn call(function: &str, platform: &PlatformInfo) -> serde_json::Value {
    let engine = StarlarkEngine::new();
    let context = ProviderContext::new("rust-rez-test", std::env::temp_dir())
        .with_platform(platform)
        .with_runtime_name("rust");
    engine
        .call_function(
            std::path::Path::new("<builtin:rust-rez-test>"),
            PROVIDER_SOURCE,
            function,
            &context,
            &[serde_json::json!("1.95.0")],
        )
        .expect("Rez bundle helper evaluates")
}

fn call_rust_provider(
    function: &str,
    platform: &PlatformInfo,
    runtime_name: &str,
) -> serde_json::Value {
    let engine = StarlarkEngine::new();
    let context = ProviderContext::new("rust", std::env::temp_dir())
        .with_platform(platform)
        .with_runtime_name(runtime_name);
    engine
        .call_function(
            std::path::Path::new("<builtin:rust>"),
            RUST_PROVIDER_SOURCE,
            function,
            &context,
            &[serde_json::json!("1.95.0")],
        )
        .expect("Rust provider Rez metadata evaluates")
}

#[rstest]
#[case("windows", "x64", "x86_64-pc-windows-msvc")]
#[case("windows", "arm64", "aarch64-pc-windows-msvc")]
#[case("linux", "x64", "x86_64-unknown-linux-gnu")]
#[case("linux", "arm64", "aarch64-unknown-linux-gnu")]
#[case("macos", "x64", "x86_64-apple-darwin")]
#[case("macos", "arm64", "aarch64-apple-darwin")]
fn maps_supported_platform_to_an_immutable_bundle_asset(
    #[case] os: &str,
    #[case] arch: &str,
    #[case] triple: &str,
) {
    let platform = PlatformInfo {
        os: os.to_string(),
        arch: arch.to_string(),
        target: triple.to_string(),
    };
    let descriptor = call("download_url", &platform);

    assert_eq!(descriptor["__type"], "rez_bundle_asset");
    assert_eq!(descriptor["supported"], true);
    assert_eq!(descriptor["triple"], triple);
    assert_eq!(
        descriptor["asset_name"],
        format!("rust-1.95.0-{triple}.rez.tar.zst")
    );
    assert_eq!(
        descriptor["url"],
        format!(
            "https://github.com/vx-org/vx-rez-packages/releases/download/rust-1.95.0/rust-1.95.0-{triple}.rez.tar.zst"
        )
    );
    assert_eq!(
        descriptor["checksum_url"],
        format!(
            "https://github.com/vx-org/vx-rez-packages/releases/download/rust-1.95.0/rust-1.95.0-{triple}.rez.tar.zst.sha256"
        )
    );
}

#[tokio::test]
async fn reports_the_declared_reason_for_an_unsupported_official_target() {
    let descriptor = call("download_url", &windows_x86());
    assert_eq!(descriptor["supported"], false);
    assert_eq!(
        descriptor["reason"],
        "Rust 1.95 Rez bundles do not publish an official i686 MSVC target."
    );

    let provider = StarlarkProvider::from_content("rust-rez-test", PROVIDER_SOURCE)
        .await
        .expect("provider loads");
    let error = provider
        .download_url_for_runtime("1.95.0", Some("rust"), Some(&windows_x86()))
        .await
        .expect_err("unsupported targets fail with the declared reason");
    assert!(
        error
            .to_string()
            .contains("do not publish an official i686 MSVC target"),
        "unexpected error: {error}"
    );
}

#[test]
fn exact_unsupported_target_overrides_a_supported_os_arch_pair() {
    let source = PROVIDER_SOURCE.replace(
        r#""windows/x86": "Rust 1.95 Rez bundles do not publish an official i686 MSVC target.","#,
        r#""windows/x86": "Rust 1.95 Rez bundles do not publish an official i686 MSVC target.",
        "x86_64-pc-windows-gnu": "The Windows bundle policy requires the MSVC ABI.","#,
    );
    let engine = StarlarkEngine::new();
    let context = ProviderContext::new("rust-rez-test", std::env::temp_dir())
        .with_platform(&windows_gnu())
        .with_runtime_name("rust");
    let descriptor = engine
        .call_function(
            std::path::Path::new("<builtin:rust-rez-test>"),
            &source,
            "download_url",
            &context,
            &[serde_json::json!("1.95.0")],
        )
        .expect("Rez bundle helper evaluates");

    assert_eq!(descriptor["supported"], false);
    assert_eq!(
        descriptor["reason"],
        "The Windows bundle policy requires the MSVC ABI."
    );
}

#[test]
fn rust_provider_exposes_bundle_metadata_without_replacing_rustup() {
    let asset = call_rust_provider("rez_download_url", &windows_x64(), "rustc");
    assert_eq!(asset["supported"], true);
    assert_eq!(
        asset["asset_name"],
        "rust-1.95.0-x86_64-pc-windows-msvc.rez.tar.zst"
    );

    // No `rust` Rez bundle has been published, so activation stays with
    // rustup. The metadata is still declared and correct — see
    // `rust_provider_activates_its_bundle_once_published` for the request the
    // runtime would receive after a release exists.
    let rustc_request = call_rust_provider("rez_bundle", &windows_x64(), "rustc");
    assert_eq!(rustc_request["enabled"], false);
    assert_eq!(rustc_request["program"], "rustc");

    let rustup_request = call_rust_provider("rez_bundle", &windows_x64(), "rust");
    assert_eq!(rustup_request["enabled"], false);
}

/// The metadata above is what activation will use once a `rust` bundle is
/// published; this pins its exact shape so flipping the flag is a one-line
/// change with a known outcome.
#[test]
fn rust_provider_activates_its_bundle_once_published() {
    let request = call("rez_bundle", &windows_x64());
    assert_eq!(request["enabled"], true);
    assert_eq!(request["repository"], ".");
    assert_eq!(
        request["requirements"],
        serde_json::json!(["rust-1.95.0", "platform-windows", "arch-x86_64"])
    );
    assert_eq!(request["program"], "rustc");
}

#[test]
fn rust_provider_rejects_the_windows_gnu_bundle_target() {
    let asset = call_rust_provider("rez_download_url", &windows_gnu(), "rustc");
    assert_eq!(asset["supported"], false);
    assert!(
        asset["reason"]
            .as_str()
            .expect("reason is a string")
            .contains("MSVC ABI")
    );
}

#[test]
fn exposes_adapter_requirements_and_required_repository_paths() {
    let request = call("rez_bundle", &windows_x64());
    assert_eq!(request["enabled"], true);
    assert_eq!(request["bundle_schema_version"], 1);
    assert_eq!(request["repository"], ".");
    assert_eq!(
        request["requirements"],
        serde_json::json!(["rust-1.95.0", "platform-windows", "arch-x86_64"])
    );
    assert_eq!(request["program"], "rustc");

    let layout = call("install_layout", &windows_x64());
    assert_eq!(layout["type"], "archive");
    assert_eq!(
        layout["required_paths"],
        serde_json::json!([
            "rust/1.95.0/package.py",
            "platform/windows/package.py",
            "arch/x86_64/package.py"
        ])
    );
}

#[tokio::test]
async fn discovers_unique_versions_from_a_release_index() {
    let body = serde_json::json!({
        "schema_version": 1,
        "bundles": [
            {"tool": "rust", "version": "1.95.0"},
            {"tool": "rust", "version": "1.95.0"},
            {"tool": "rust", "version": "1.94.1"},
            {"tool": "python", "version": "3.14.0"}
        ]
    })
    .to_string();
    let (url, server) = serve_json_once(body);
    let provider = provider_with_versions_url(&url).await;

    let versions = provider.fetch_versions().await.expect("index is parsed");
    server.join().expect("test server exits");
    assert_eq!(
        versions
            .iter()
            .map(|version| version.version.as_str())
            .collect::<Vec<_>>(),
        ["1.95.0", "1.94.1"]
    );
}

#[tokio::test]
async fn discovers_only_complete_tool_releases_from_github_metadata() {
    let body = serde_json::json!([
        {
            "tag_name": "rust-1.95.0",
            "prerelease": false,
            "published_at": "2026-04-16T00:00:00Z",
            "assets": [
                {"name": "index.json"},
                {"name": "rust-1.95.0-x86_64-pc-windows-msvc.rez.tar.zst"}
            ]
        },
        {
            "tag_name": "rust-1.94.0",
            "prerelease": false,
            "assets": [{"name": "index.json"}]
        },
        {
            "tag_name": "python-3.14.0",
            "prerelease": false,
            "assets": [
                {"name": "index.json"},
                {"name": "python-3.14.0-x86_64-pc-windows-msvc.rez.tar.zst"}
            ]
        }
    ])
    .to_string();
    let (url, server) = serve_json_once(body);
    let provider = provider_with_versions_url(&url).await;

    let versions = provider
        .fetch_versions()
        .await
        .expect("releases are parsed");
    server.join().expect("test server exits");
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].version, "1.95.0");
    assert_eq!(
        versions[0].date.as_deref(),
        Some("2026-04-16T00:00:00+00:00")
    );
}

async fn provider_with_versions_url(url: &str) -> StarlarkProvider {
    let source = format!(
        r#"
load("@vx//stdlib:rez.star", "rez_bundle_source")
name = "rust-rez-discovery"
description = "Rez bundle discovery test"
runtimes = [{{"name": "rust", "executable": "rustc"}}]
_bundle = rez_bundle_source(
    "vx-org",
    "vx-rez-packages",
    "rust",
    targets = {{"windows/x64": "x86_64-pc-windows-msvc"}},
    versions_url = "{url}",
)
fetch_versions = _bundle["fetch_versions"]
download_url = _bundle["download_url"]
install_layout = _bundle["install_layout"]
"#
    );
    StarlarkProvider::from_content("rust-rez-discovery", source)
        .await
        .expect("provider loads")
}

fn serve_json_once(body: String) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("test server binds");
    let address = listener.local_addr().expect("test server address");
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("request arrives");
        let mut request = [0_u8; 4096];
        let _ = stream.read(&mut request).expect("request is readable");
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
        .expect("response is written");
    });
    (format!("http://{address}/index.json"), handle)
}
