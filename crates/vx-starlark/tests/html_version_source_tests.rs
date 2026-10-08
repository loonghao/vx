//! Published asset discovery through the public Provider HTTP path.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};

use rstest::rstest;
use vx_starlark::context::{PlatformInfo, VersionInfo};
use vx_starlark::{StarlarkProvider, global_version_cache};

const ROOT: &str = "https://download.example.org/stable/editor/";
static NEXT_PROVIDER: AtomicUsize = AtomicUsize::new(0);

fn server(status: &str, body: &str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind HTML source");
    let address = listener.local_addr().expect("HTML source address");
    let status = status.to_owned();
    let body = body.to_owned();
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept HTML request");
        let mut request = [0_u8; 4096];
        assert!(stream.read(&mut request).expect("read HTML request") > 0);
        write!(stream, "HTTP/1.1 {status}\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
            .expect("write HTML response");
    });
    format!("http://{address}/downloads")
}

async fn provider(url: &str, suffix: &str, mutation: &str) -> StarlarkProvider {
    let name = format!(
        "html-source-test-{}",
        NEXT_PROVIDER.fetch_add(1, Ordering::Relaxed)
    );
    let content = format!(
        r#"
load("@vx//stdlib:http.star", "fetch_html_versions")
name = "{name}"
description = "Published asset version test"
runtimes = [{{"name": name, "executable": name}}]
def fetch_versions(ctx):
    descriptor = fetch_html_versions(ctx, "{url}", "{ROOT}", "editor-", "{suffix}")
    {mutation}
    return descriptor
"#
    );
    StarlarkProvider::from_content(&name, content)
        .await
        .expect("load HTML Provider")
}

async fn versions(body: &str, suffix: &str) -> Vec<String> {
    let url = server("200 OK", body);
    provider(&url, suffix, "pass")
        .await
        .fetch_versions()
        .await
        .expect("fetch published versions")
        .into_iter()
        .map(|version| version.version)
        .collect()
}

#[tokio::test]
async fn source_tags_do_not_advertise_unpublished_binaries() {
    let body = format!(
        r#"
<a href="https://github.com/example/editor/tree/v26.08.2">26.08.2 source</a>
<a href="{ROOT}26.08/linux/editor-26.08.1-x86_64.AppImage">Linux</a>
<a href="{ROOT}26.08/windows/editor-26.08.0_standalone.exe">Windows</a>
<a href="{ROOT}26.08/macOS/editor-26.08.1-arm64.dmg">Mac ARM</a>
<a href="{ROOT}26.08/macOS/editor-26.08.0-x86_64.dmg">Mac Intel</a>
"#
    );
    assert_eq!(versions(&body, "-x86_64.AppImage").await, ["26.08.1"]);
    assert_eq!(versions(&body, "_standalone.exe").await, ["26.08.0"]);
    assert_eq!(versions(&body, "-arm64.dmg").await, ["26.08.1"]);
    assert_eq!(versions(&body, "-x86_64.dmg").await, ["26.08.0"]);
    let published =
        format!("{body}<a href='{ROOT}26.08/linux/editor-26.08.2-x86_64.AppImage'>new binary</a>");
    assert_eq!(
        versions(&published, "-x86_64.AppImage").await,
        ["26.08.2", "26.08.1"]
    );
}

#[rstest]
#[case("\"", "\"")]
#[case("'", "'")]
#[case("", "")]
#[tokio::test]
async fn anchor_href_quotes_and_attribute_boundaries(#[case] before: &str, #[case] after: &str) {
    let body = format!(
        "<A data-href='{ROOT}editor-99.0.zip' HREF = {before}{ROOT}editor-1.2.zip{after} class='download'>asset</A>"
    );
    assert_eq!(versions(&body, ".zip").await, ["1.2"]);
}

#[rstest]
#[case("a?")]
#[case("a!")]
#[case("a=")]
#[case("a\u{b}")]
#[tokio::test]
async fn similar_tag_names_are_not_anchors(#[case] tag: &str) {
    let body = format!(
        "<{tag} href='{ROOT}editor-99.0.zip'>not an anchor</{tag}><a href='{ROOT}editor-1.2.zip'>asset</a>"
    );
    assert_eq!(versions(&body, ".zip").await, ["1.2"]);
}

#[rstest]
#[case("iframe")]
#[case("xmp")]
#[case("noembed")]
#[case("noframes")]
#[tokio::test]
async fn raw_text_elements_do_not_publish_embedded_anchor_text(#[case] tag: &str) {
    let body = format!(
        "<{tag}><a href='{ROOT}editor-99.0.zip'>raw text</a></{tag}><a href='{ROOT}editor-1.2.zip'>asset</a>"
    );
    assert_eq!(versions(&body, ".zip").await, ["1.2"]);
}

#[tokio::test]
async fn plaintext_consumes_the_rest_of_the_document() {
    let body = format!(
        "<a href='{ROOT}editor-1.2.zip'>asset</a><plaintext><a href='{ROOT}editor-99.0.zip'>text</a></plaintext><a href='{ROOT}editor-98.0.zip'>still text</a>"
    );
    assert_eq!(versions(&body, ".zip").await, ["1.2"]);
}

#[tokio::test]
async fn ignores_text_nonanchors_comments_raw_text_foreign_hosts_and_nonstable_links() {
    let body = format!(
        r#"
{ROOT}editor-99.0.zip
<div href="{ROOT}editor-98.0.zip"></div>
<a data-href="{ROOT}editor-97.0.zip">not href</a>
<!-- <a href="{ROOT}editor-96.0.zip">comment</a> -->
<script>const link = '<a href="{ROOT}editor-95.0.zip">script</a>';</script>
<script>const falseClose = '</scriptx><a href="{ROOT}editor-95.1.zip">script</a>';</script>
<style>content: '<a href="{ROOT}editor-94.0.zip">style</a>';</style>
<textarea><a href="{ROOT}editor-93.0.zip">text</a></textarea>
<a href="https://download.example.org.evil/stable/editor/editor-92.0.zip">foreign</a>
<a href="https://download.example.org/daily/editor/editor-91.0.zip">daily</a>
<a href="{ROOT}../daily/editor-90.0.zip">traversal</a>
<a href="{ROOT}%2e%2e/daily/editor-90.1.zip">encoded traversal</a>
<a href="{ROOT}editor-90.2.zip" href="{ROOT}editor-1.2.zip">first duplicate wins</a>
<a href="{ROOT}editor-89.0.zip?mirror=1">query</a>
<a href="{ROOT}editor-88.0.zip#fragment">fragment</a>
<a href="{ROOT}editor-1.2.zip">valid</a>
"#
    );
    assert_eq!(versions(&body, ".zip").await, ["90.2", "1.2"]);
}

#[tokio::test]
async fn numeric_versions_are_deduplicated_sorted_and_suffix_filtered() {
    let body = [
        "2.9", "2.10", "2.10", "2.9-beta", "3", "3..1", "３.1", ".4.1", "4.1.", "2.90",
    ]
    .map(|version| format!("<a href='{ROOT}editor-{version}.zip'>download</a>"))
    .join("\n");
    let url = server("200 OK", &body);
    let provider = provider(
        &url,
        ".zip",
        "descriptor[\"exclude_version_suffixes\"] = [\".90\"]",
    )
    .await;
    let versions = provider
        .fetch_versions()
        .await
        .expect("fetch stable numeric versions");
    assert_eq!(
        versions
            .iter()
            .map(|v| v.version.as_str())
            .collect::<Vec<_>>(),
        ["2.10", "2.9"]
    );
    assert!(
        versions
            .iter()
            .all(|v| v.stable && !v.lts && v.date.is_none())
    );
}

#[rstest]
#[case("404 Not Found")]
#[case("503 Service Unavailable")]
#[case("302 Found")]
#[tokio::test]
async fn unsuccessful_status_fails_even_when_body_contains_asset(#[case] status: &str) {
    let url = server(
        status,
        &format!("<a href='{ROOT}editor-1.2.zip'>download</a>"),
    );
    let error = provider(&url, ".zip", "pass")
        .await
        .fetch_versions()
        .await
        .expect_err("HTTP failure must fail closed");
    assert!(
        error
            .to_string()
            .contains(status.split(' ').next().unwrap()),
        "{error}"
    );
}

#[rstest]
#[case("<p>No binary published</p>")]
#[case("<a href='https://download.example.org/stable/editor/editor-1.2.tar.gz'>source only</a>")]
#[case("<a href='https://download.example.org/stable/editor/editor-1.2.zip")]
#[case("<!-- <a href='https://download.example.org/stable/editor/editor-1.2.zip'>never closed")]
#[tokio::test]
async fn missing_matching_assets_is_an_error(#[case] body: &str) {
    let url = server("200 OK", body);
    let error = provider(&url, ".zip", "pass")
        .await
        .fetch_versions()
        .await
        .expect_err("no published binary must fail");
    assert!(
        error.to_string().contains("no matching published assets"),
        "{error}"
    );
}

#[rstest]
#[case("descriptor[\"url\"] = 17", "url")]
#[case(
    "descriptor[\"href_prefix\"] = \"https://example.org/stable\"",
    "href_prefix"
)]
#[case("descriptor[\"href_prefix\"] = \"not-a-url/\"", "href_prefix")]
#[case("descriptor[\"filename_prefix\"] = \"\"", "filename_prefix")]
#[case("descriptor[\"filename_suffix\"] = 17", "filename_suffix")]
#[case("descriptor[\"version_filter\"] = \"anything\"", "version_filter")]
#[case(
    "descriptor[\"exclude_version_suffixes\"] = [17]",
    "exclude_version_suffixes"
)]
#[tokio::test]
async fn malformed_descriptor_fails_before_network(#[case] mutation: &str, #[case] field: &str) {
    let error = provider("http://127.0.0.1:1/never-requested", ".zip", mutation)
        .await
        .fetch_versions()
        .await
        .expect_err("invalid descriptor must fail");
    assert!(error.to_string().contains(field), "{error}");
}

#[tokio::test]
async fn html_cache_is_platform_scoped_and_reused_by_the_public_provider_path() {
    let url = server(
        "200 OK",
        &format!("<a href='{ROOT}editor-1.2.zip'>download</a>"),
    );
    let provider = provider(&url, ".zip", "pass").await;
    let poison = vec![VersionInfo {
        version: "99.0".into(),
        lts: false,
        stable: true,
        date: Some("other-platform".into()),
    }];
    let cache = global_version_cache();
    // A legacy, unscoped list must not satisfy an HTML query for this platform.
    cache
        .put(&provider.meta().name, &provider.script_hash_hex(), &poison)
        .await;
    let platform = PlatformInfo::current();
    let other_os = if platform.os == "windows" {
        "linux"
    } else {
        "windows"
    };
    cache
        .put(
            &format!("{}/html/{other_os}/{}", provider.meta().name, platform.arch),
            &provider.script_hash_hex(),
            &poison,
        )
        .await;
    for _ in 0..2 {
        let versions = provider
            .fetch_versions()
            .await
            .expect("fetch or reuse platform versions");
        assert_eq!(versions[0].version, "1.2");
        assert!(versions[0].date.is_none());
    }
    let scope = format!(
        "{}/html/{}/{}",
        provider.meta().name,
        platform.os,
        platform.arch
    );
    assert_eq!(
        cache
            .get(&scope, &provider.script_hash_hex())
            .await
            .expect("platform cache entry")[0]
            .version,
        "1.2"
    );
}
