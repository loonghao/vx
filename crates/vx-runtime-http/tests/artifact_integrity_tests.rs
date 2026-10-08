//! Fixed artifact digests apply to downloads and cached bytes before installation.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use rstest::rstest;
use sha2::{Digest, Sha256};
use vx_cache::{CacheLookupResult, DownloadCache};
use vx_runtime::Installer;
use vx_runtime_http::RealInstaller;

const PAYLOAD: &[u8] = b"fixed runtime artifact";

fn metadata() -> HashMap<String, String> {
    HashMap::from([("sha256".into(), hex::encode(Sha256::digest(PAYLOAD)))])
}

fn serve() -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/runtime.bin", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut request = Vec::new();
        let mut buffer = [0; 1024];
        while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
            let count = stream.read(&mut buffer).unwrap();
            assert!(count > 0);
            request.extend_from_slice(&buffer[..count]);
        }
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            PAYLOAD.len()
        )
        .unwrap();
        stream.write_all(PAYLOAD).unwrap();
    });
    (url, server)
}

#[tokio::test]
async fn verified_download_is_reusable_without_a_server() {
    let temp = tempfile::tempdir().unwrap();
    let installer = RealInstaller::with_download_cache(temp.path().join("cache"));
    let (url, server) = serve();
    let first = temp.path().join("first");
    installer
        .download_with_layout(&url, &first, &metadata())
        .await
        .unwrap();
    server.join().unwrap();
    let offline = temp.path().join("offline");
    installer
        .download_with_layout(&url, &offline, &metadata())
        .await
        .unwrap();
    assert_eq!(
        std::fs::read(offline.join("bin/runtime.bin")).unwrap(),
        PAYLOAD
    );
}

#[tokio::test]
async fn wrong_digest_is_neither_installed_nor_cached() {
    let temp = tempfile::tempdir().unwrap();
    let cache_dir = temp.path().join("cache");
    let installer = RealInstaller::with_download_cache(cache_dir.clone());
    let (url, server) = serve();
    let dest = temp.path().join("install");
    let digest = HashMap::from([("sha256".into(), "0".repeat(64))]);
    let error = installer
        .download_with_layout(&url, &dest, &digest)
        .await
        .unwrap_err();
    server.join().unwrap();
    assert!(error.to_string().contains("SHA256 mismatch"));
    assert!(!dest.exists(), "unverified bytes must not be extracted");
    assert!(matches!(
        DownloadCache::new(cache_dir).lookup(&url),
        CacheLookupResult::Miss
    ));
}

#[tokio::test]
async fn same_size_cache_corruption_is_rejected_and_evicted() {
    let temp = tempfile::tempdir().unwrap();
    let cache_dir = temp.path().join("cache");
    let cache = DownloadCache::new(cache_dir.clone());
    let url = "http://127.0.0.1:1/runtime.bin";
    let source = temp.path().join("source");
    std::fs::write(&source, PAYLOAD).unwrap();
    cache.store(url, &source, None, None, None).unwrap();
    let path = cache.file_path(&DownloadCache::cache_key(url));
    let mut corrupt = PAYLOAD.to_vec();
    corrupt[0] ^= 1;
    std::fs::write(path, corrupt).unwrap();
    let dest = temp.path().join("install");
    let error = RealInstaller::with_download_cache(cache_dir)
        .download_with_layout(url, &dest, &metadata())
        .await
        .unwrap_err();
    assert!(error.to_string().contains("SHA256 mismatch"));
    assert!(!dest.exists());
    assert!(matches!(cache.lookup(url), CacheLookupResult::Miss));
}

#[rstest]
#[case("")]
#[case("abc")]
#[case(&"g".repeat(64))]
#[tokio::test]
async fn malformed_digest_fails_before_network_access(#[case] digest: &str) {
    let temp = tempfile::tempdir().unwrap();
    let dest = temp.path().join("install");
    let metadata = HashMap::from([("sha256".into(), digest.into())]);
    let error = RealInstaller::new()
        .download_with_layout("http://127.0.0.1:1/runtime.bin", &dest, &metadata)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("Invalid artifact SHA256"));
    assert!(!dest.exists());
}
