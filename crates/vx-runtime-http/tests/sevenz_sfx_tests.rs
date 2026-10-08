//! RealInstaller download and extraction contracts for embedded 7z archives.

#![cfg(feature = "extended-formats")]

use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use rstest::rstest;
use sevenz_rust::{SevenZArchiveEntry, SevenZWriter};
use tempfile::tempdir;

use vx_runtime::Installer;
use vx_runtime_http::RealInstaller;

const CONTENT: &[u8] = b"portable runtime fixture";
const MAGIC: &[u8] = b"7z\xBC\xAF\x27\x1C";

fn archive_bytes(name: &str) -> Vec<u8> {
    let mut writer = SevenZWriter::new(Cursor::new(Vec::new())).unwrap();
    let mut entry = SevenZArchiveEntry::new();
    entry.name = name.into();
    entry.has_stream = true;
    writer
        .push_archive_entry(entry, Some(Cursor::new(CONTENT)))
        .unwrap();
    writer.finish().unwrap().into_inner()
}

fn sfx_bytes(stub_size: usize, name: &str, decoy: bool) -> Vec<u8> {
    let mut bytes = vec![0; stub_size];
    if stub_size > 0 {
        bytes[..2].copy_from_slice(b"MZ");
    }
    if decoy {
        bytes[64..64 + MAGIC.len()].copy_from_slice(MAGIC);
    }
    bytes.extend(archive_bytes(name));
    bytes
}

fn serve(bytes: Vec<u8>, name: &str) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/{name}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut request = Vec::new();
        let mut chunk = [0; 1024];
        while !request.windows(4).any(|part| part == b"\r\n\r\n") {
            let count = stream.read(&mut chunk).unwrap();
            assert!(count > 0);
            request.extend_from_slice(&chunk[..count]);
        }
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            bytes.len()
        )
        .unwrap();
        stream.write_all(&bytes).unwrap();
    });
    (url, server)
}

#[rstest]
#[case(0, false)]
#[case(512, false)]
#[case(215_040, true)]
#[tokio::test]
async fn test_download_extracts_plain_and_sfx_payloads(
    #[case] stub_size: usize,
    #[case] decoy: bool,
) {
    let temp = tempdir().unwrap();
    let dest = temp.path().join("output");
    let filename = if stub_size == 0 {
        "runtime.7z"
    } else {
        "runtime.exe"
    };
    let (url, server) = serve(
        sfx_bytes(stub_size, "runtime/bin/application.exe", decoy),
        filename,
    );

    RealInstaller::new()
        .download_and_extract(&url, &dest)
        .await
        .unwrap();
    server.join().unwrap();

    assert_eq!(
        std::fs::read(dest.join("runtime/bin/application.exe")).unwrap(),
        CONTENT
    );
    assert!(!dest.join("bin").join(filename).exists());
}

#[tokio::test]
async fn test_explicit_binary_layout_preserves_embedded_installer_archive() {
    let temp = tempdir().unwrap();
    let dest = temp.path().join("output");
    let bytes = sfx_bytes(512, "application.exe", false);
    let (url, server) = serve(bytes.clone(), "setup.exe");
    let metadata = HashMap::from([
        ("target_name".into(), "installer.exe".into()),
        ("target_dir".into(), "bin".into()),
    ]);

    RealInstaller::new()
        .download_with_layout(&url, &dest, &metadata)
        .await
        .unwrap();
    server.join().unwrap();

    assert_eq!(
        std::fs::read(dest.join("bin/installer.exe")).unwrap(),
        bytes
    );
    assert!(!dest.join("application.exe").exists());
}

#[tokio::test]
async fn test_named_archive_layout_extracts_and_renames_without_losing_bundle_permissions() {
    let temp = tempdir().unwrap();
    let dest = temp.path().join("output");
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, mode) in [
        ("bin/downloaded.exe", 0o755),
        ("Example.app/Contents/MacOS/Example", 0o755),
        ("Example.app/Contents/Resources/settings.json", 0o644),
    ] {
        writer
            .start_file(
                name,
                zip::write::SimpleFileOptions::default().unix_permissions(mode),
            )
            .unwrap();
        writer.write_all(CONTENT).unwrap();
    }
    let (url, server) = serve(writer.finish().unwrap().into_inner(), "runtime.zip");
    let metadata = HashMap::from([
        ("source_name".into(), "downloaded.exe".into()),
        ("target_name".into(), "application.exe".into()),
        ("target_dir".into(), "bin".into()),
    ]);

    RealInstaller::new()
        .download_with_layout(&url, &dest, &metadata)
        .await
        .unwrap();
    server.join().unwrap();

    assert_eq!(
        std::fs::read(dest.join("bin/application.exe")).unwrap(),
        CONTENT
    );
    assert!(!dest.join("bin/downloaded.exe").exists());
    assert_eq!(
        std::fs::read(dest.join("Example.app/Contents/MacOS/Example")).unwrap(),
        CONTENT
    );
    assert_eq!(
        std::fs::read(dest.join("Example.app/Contents/Resources/settings.json")).unwrap(),
        CONTENT
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        for (name, mode) in [
            ("Example.app/Contents/MacOS/Example", 0o755),
            ("Example.app/Contents/Resources/settings.json", 0o644),
        ] {
            assert_eq!(
                std::fs::metadata(dest.join(name))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o7777,
                mode
            );
        }
    }
}

#[rstest]
#[case(false)]
#[case(true)]
#[tokio::test]
async fn test_plain_executable_and_fake_signature_remain_binary(#[case] fake_signature: bool) {
    let temp = tempdir().unwrap();
    let dest = temp.path().join("output");
    let mut bytes = b"MZ ordinary executable".to_vec();
    if fake_signature {
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&[0; 40]);
    }
    let (url, server) = serve(bytes.clone(), "runtime.exe");

    RealInstaller::new()
        .download_and_extract(&url, &dest)
        .await
        .unwrap();
    server.join().unwrap();

    assert_eq!(std::fs::read(dest.join("bin/runtime.exe")).unwrap(), bytes);
}

#[tokio::test]
async fn test_signature_beyond_scan_limit_is_not_extracted() {
    let temp = tempdir().unwrap();
    let dest = temp.path().join("output");
    let bytes = sfx_bytes(4 * 1024 * 1024, "application.exe", false);
    let (url, server) = serve(bytes.clone(), "runtime.exe");

    RealInstaller::new()
        .download_and_extract(&url, &dest)
        .await
        .unwrap();
    server.join().unwrap();

    assert_eq!(std::fs::read(dest.join("bin/runtime.exe")).unwrap(), bytes);
    assert!(!dest.join("application.exe").exists());
}

#[rstest]
#[case(false)]
#[case(true)]
#[tokio::test]
async fn test_valid_header_with_corrupt_payload_fails_download_extraction(#[case] truncated: bool) {
    let temp = tempdir().unwrap();
    let dest = temp.path().join("output");
    let mut bytes = sfx_bytes(512, "application.exe", false);
    if truncated {
        bytes.truncate(512 + 40);
    } else {
        bytes[512 + 32] ^= 0xff;
    }
    let (url, server) = serve(bytes, "runtime.exe");

    let result = RealInstaller::new().download_and_extract(&url, &dest).await;
    server.join().unwrap();

    assert!(result.is_err());
    assert!(!dest.join("bin/runtime.exe").exists());
}

#[rstest]
#[case(b"7z\xBC\xAF\x27\x1C".as_slice())]
#[case(b"MZ truncated header 7z\xBC\xAF\x27\x1C".as_slice())]
#[tokio::test]
async fn test_explicit_sevenz_rejects_truncated_header(#[case] bytes: &[u8]) {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("runtime.7z.exe");
    let dest = temp.path().join("output");
    std::fs::write(&archive, bytes).unwrap();

    assert!(RealInstaller::new().extract(&archive, &dest).await.is_err());
    assert!(std::fs::read_dir(dest).unwrap().next().is_none());
}

#[rstest]
#[case("../escaped.exe")]
#[case("..\\escaped.exe")]
#[case("/escaped.exe")]
#[case("C:/escaped.exe")]
#[case("C:\\escaped.exe")]
#[tokio::test]
async fn test_sfx_rejects_paths_outside_destination(#[case] name: &str) {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("runtime.exe");
    let dest = temp.path().join("output");
    std::fs::write(&archive, sfx_bytes(512, name, false)).unwrap();

    let error = RealInstaller::new()
        .extract(&archive, &dest)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("Unsafe path"), "{name}: {error}");
    assert!(!temp.path().join("escaped.exe").exists());
    assert!(std::fs::read_dir(dest).unwrap().next().is_none());
}

#[cfg(unix)]
#[tokio::test]
async fn test_sfx_rejects_existing_symlink_ancestor() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("runtime.exe");
    let dest = temp.path().join("output");
    let outside = temp.path().join("outside");
    std::fs::create_dir_all(&dest).unwrap();
    std::fs::create_dir(&outside).unwrap();
    std::os::unix::fs::symlink(&outside, dest.join("runtime")).unwrap();
    std::fs::write(&archive, sfx_bytes(512, "runtime/escaped.exe", false)).unwrap();

    let error = RealInstaller::new()
        .extract(&archive, &dest)
        .await
        .unwrap_err();

    assert!(error.to_string().contains("Symlink"), "{error}");
    assert!(!outside.join("escaped.exe").exists());
}
