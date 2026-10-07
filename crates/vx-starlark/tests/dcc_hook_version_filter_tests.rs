//! Numeric tag filtering for JSON version sources used by DCC providers.

use std::io::{Read, Write};
use std::net::TcpListener;

use rstest::rstest;

use vx_starlark::StarlarkProvider;

fn json_server(body: &str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind version source");
    let address = listener.local_addr().expect("version source address");
    let body = body.to_string();
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept version request");
        let mut request = [0_u8; 4096];
        let received = stream.read(&mut request).expect("read version request");
        assert!(received > 0, "version source received an empty request");
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
        .expect("write version response");
    });
    format!("http://{address}/versions")
}

async fn provider(url: &str, filter: &str) -> StarlarkProvider {
    let content = format!(
        r#"
load("@vx//stdlib:http.star", "fetch_json_versions")
name = "dcc-json-version-filter-test"
description = "Version filter regression test"
runtimes = [{{"name": name, "executable": name}}]

def fetch_versions(ctx):
    descriptor = fetch_json_versions(ctx, "{url}", "github_tags")
    {filter}
    return descriptor
"#
    );
    StarlarkProvider::from_content("dcc-json-version-filter-test", &content)
        .await
        .expect("load JSON source provider")
}

#[tokio::test]
async fn test_dcc_hook_numeric_filter_applies_after_normalizing_tags() {
    let url = json_server(
        r#"[{"name":"v5.2.9"},{"name":"v6.0"},{"name":"v42.0-beta2"},{"name":"5.3.0.1"},{"name":"7"},{"name":"8..1"},{"name":".9.0"},{"name":"1.2."},{"name":"１.2"},{"name":"release-5.0"}]"#,
    );
    let provider = provider(&url, "descriptor[\"version_filter\"] = \"numeric\"").await;
    let versions = provider
        .fetch_versions()
        .await
        .expect("fetch numeric versions");
    assert_eq!(
        versions
            .into_iter()
            .map(|version| version.version)
            .collect::<Vec<_>>(),
        ["5.2.9", "6.0", "5.3.0.1"]
    );
}

#[tokio::test]
async fn test_dcc_hook_unfiltered_source_preserves_existing_tag_behavior() {
    let url = json_server(r#"[{"name":"v5.2.9"},{"name":"v42.0-beta2"}]"#);
    let provider = provider(&url, "pass").await;
    let versions = provider
        .fetch_versions()
        .await
        .expect("fetch unfiltered versions");
    assert_eq!(versions.len(), 2);
    assert!(
        versions
            .iter()
            .any(|version| version.version == "42.0-beta2")
    );
}

#[tokio::test]
async fn test_calendar_release_prerelease_suffixes_are_excluded() {
    let url = json_server(
        r#"[{"name":"v26.08.2"},{"name":"v26.11.90"},{"name":"v26.07.80"},{"name":"v26.08.1"}]"#,
    );
    let provider = provider(&url, "descriptor[\"version_filter\"] = \"numeric\"\n    descriptor[\"exclude_version_suffixes\"] = [\".80\", \".90\"]").await;
    let versions = provider
        .fetch_versions()
        .await
        .expect("fetch stable releases");
    assert_eq!(
        versions.into_iter().map(|v| v.version).collect::<Vec<_>>(),
        ["26.08.2", "26.08.1"]
    );
}

#[rstest]
#[case("17")]
#[case("[17]")]
#[case("[\"\"]")]
#[tokio::test]
async fn test_malformed_excluded_suffixes_fail_before_network(#[case] suffixes: &str) {
    let provider = provider(
        "http://127.0.0.1:1/never-requested",
        &format!("descriptor[\"exclude_version_suffixes\"] = {suffixes}"),
    )
    .await;
    let error = provider
        .fetch_versions()
        .await
        .expect_err("invalid suffixes must fail");
    assert!(
        error.to_string().contains("exclude_version_suffixes"),
        "{error}"
    );
}

#[rstest]
#[case("\"unknown\"")]
#[case("17")]
#[tokio::test]
async fn test_dcc_hook_invalid_version_filter_fails_before_network(#[case] filter: &str) {
    let provider = provider(
        "http://127.0.0.1:1/never-requested",
        &format!("descriptor[\"version_filter\"] = {filter}"),
    )
    .await;
    let error = provider
        .fetch_versions()
        .await
        .expect_err("invalid filter must fail");
    assert!(
        error
            .to_string()
            .contains("Unsupported fetch_json_versions version_filter"),
        "{error}"
    );
}
