//! Version source retry and stale-cache fallback tests.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tempfile::tempdir;
use vx_starlark::{StarlarkProvider, VersionCache};

fn spawn_flaky_json_server(failures: usize) -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let address = listener.local_addr().expect("read test server address");
    let attempts = Arc::new(AtomicUsize::new(0));
    let server_attempts = Arc::clone(&attempts);

    std::thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request);
            let attempt = server_attempts.fetch_add(1, Ordering::SeqCst) + 1;

            let (status, body) = if attempt <= failures {
                ("503 Service Unavailable", r#"{"error":"temporary"}"#)
            } else {
                ("200 OK", r#"{"versions":{"1.2.3":{}}}"#)
            };
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("write test response");

            if attempt > failures {
                break;
            }
        }
    });

    (format!("http://{address}/versions"), attempts)
}

#[tokio::test]
async fn descriptor_version_source_retries_transient_failures() {
    let (url, attempts) = spawn_flaky_json_server(2);
    let content = format!(
        r#"
load("@vx//stdlib:http.star", "fetch_json_versions")

name = "retry-version-source-test"
description = "retry test"
runtimes = [{{"name": name, "executable": name}}]

def fetch_versions(ctx):
    return fetch_json_versions(ctx, "{url}", "npm_registry")
"#
    );
    let provider = StarlarkProvider::from_content("retry-version-source-test", &content)
        .await
        .expect("load provider");

    let versions = provider.fetch_versions().await.expect("fetch versions");

    assert_eq!(attempts.load(Ordering::SeqCst), 3);
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].version, "1.2.3");
}

#[tokio::test]
async fn stale_cache_requires_expiry_and_matching_script_hash() {
    let cache_dir = tempdir().expect("create cache dir");
    let stale_cache = VersionCache::new(cache_dir.path(), 0);
    let versions = vec![vx_starlark::context::VersionInfo {
        version: "1.2.3".to_string(),
        lts: false,
        stable: true,
        date: None,
    }];
    stale_cache.put("ffmpeg", "same-script", &versions).await;

    assert_eq!(
        stale_cache
            .get_stale("ffmpeg", "same-script")
            .await
            .expect("matching stale entry")[0]
            .version,
        "1.2.3"
    );
    assert!(
        stale_cache
            .get_stale("ffmpeg", "changed-script")
            .await
            .is_none()
    );

    let fresh_cache = VersionCache::new(cache_dir.path().join("fresh"), 3600);
    fresh_cache.put("ffmpeg", "same-script", &versions).await;
    assert!(
        fresh_cache
            .get_stale("ffmpeg", "same-script")
            .await
            .is_none()
    );
}
