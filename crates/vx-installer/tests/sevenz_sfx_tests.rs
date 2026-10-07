//! Regression coverage for 7z archives embedded in self-extracting executables.

#![cfg(feature = "extended-formats")]

use std::io::{Cursor, Write};
use std::path::Path;

use sevenz_rust::{SevenZArchiveEntry, SevenZWriter};
use tempfile::TempDir;

use vx_installer::formats::{ArchiveExtractor, FormatHandler, sevenz::SevenZipHandler};
use vx_installer::progress::ProgressContext;

const CONTENT: &[u8] = b"portable runtime fixture";

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

fn write_archive(path: &Path, stub_size: usize, name: &str) {
    let mut output = std::fs::File::create(path).unwrap();
    if stub_size > 0 {
        let mut stub = vec![0; stub_size];
        stub[..2].copy_from_slice(b"MZ");
        output.write_all(&stub).unwrap();
    }
    output.write_all(&archive_bytes(name)).unwrap();
}

#[tokio::test]
async fn test_sevenz_extracts_plain_and_sfx_archives() {
    let temp = TempDir::new().unwrap();
    let handler = SevenZipHandler::new();
    for stub_size in [0, 512, 215_040] {
        let extension = if stub_size == 0 { "7z" } else { "exe" };
        let archive = temp.path().join(format!("runtime-{stub_size}.{extension}"));
        let target = temp.path().join(format!("extracted-{stub_size}"));
        write_archive(&archive, stub_size, "runtime/bin/application.exe");
        assert!(handler.can_handle(&archive));
        let extracted = ArchiveExtractor::new()
            .extract(&archive, &target, &ProgressContext::disabled())
            .await
            .unwrap();
        let executable = target.join("runtime/bin/application.exe");
        assert_eq!(std::fs::read(&executable).unwrap(), CONTENT);
        assert_eq!(extracted, vec![executable]);
    }
}

#[tokio::test]
async fn test_sevenz_rejects_malformed_self_extracting_executables() {
    let temp = TempDir::new().unwrap();
    let handler = SevenZipHandler::new();
    for (index, content) in [
        b"MZ plain executable".as_slice(),
        b"MZ truncated archive 7z\xBC\xAF\x27\x1C".as_slice(),
    ]
    .into_iter()
    .enumerate()
    {
        let archive = temp.path().join(format!("malformed-{index}.exe"));
        let target = temp.path().join(format!("extracted-{index}"));
        std::fs::write(&archive, content).unwrap();
        assert!(
            handler
                .extract(&archive, &target, &ProgressContext::disabled())
                .await
                .is_err()
        );
        assert!(std::fs::read_dir(target).unwrap().next().is_none());
    }
}

#[tokio::test]
async fn test_sevenz_sfx_signature_scan_is_bounded() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("oversized-stub.exe");
    let target = temp.path().join("extracted");
    write_archive(&archive, 4 * 1024 * 1024, "runtime.exe");
    let handler = SevenZipHandler::new();
    assert!(!handler.can_handle(&archive));
    let error = handler
        .extract(&archive, &target, &ProgressContext::disabled())
        .await
        .unwrap_err();
    assert!(error.to_string().contains("signature not found"));
}

#[tokio::test]
async fn test_sevenz_sfx_rejects_paths_outside_target() {
    let temp = TempDir::new().unwrap();
    let handler = SevenZipHandler::new();
    for (index, name) in [
        "../escaped.exe",
        "..\\escaped.exe",
        "/escaped.exe",
        "C:/escaped.exe",
    ]
    .into_iter()
    .enumerate()
    {
        let archive = temp.path().join(format!("unsafe-{index}.exe"));
        let target = temp.path().join(format!("extracted-{index}"));
        write_archive(&archive, 512, name);
        let error = handler
            .extract(&archive, &target, &ProgressContext::disabled())
            .await
            .unwrap_err();
        assert!(error.to_string().contains("Unsafe path"), "{name}: {error}");
        assert!(!temp.path().join("escaped.exe").exists());
        assert!(std::fs::read_dir(target).unwrap().next().is_none());
    }
}

#[cfg(unix)]
#[tokio::test]
async fn test_sevenz_sfx_rejects_existing_symlink_ancestors() {
    let temp = TempDir::new().unwrap();
    let archive = temp.path().join("runtime.exe");
    let target = temp.path().join("extracted");
    let outside = temp.path().join("outside");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    std::os::unix::fs::symlink(&outside, target.join("runtime")).unwrap();
    write_archive(&archive, 512, "runtime/escaped.exe");
    let error = SevenZipHandler::new()
        .extract(&archive, &target, &ProgressContext::disabled())
        .await
        .unwrap_err();
    assert!(error.to_string().contains("Symlink"));
    assert!(!outside.join("escaped.exe").exists());
}
