//! Public archive API regression coverage for untrusted 7z entry names.

#![cfg(feature = "extended-formats")]

use std::io::Cursor;
use std::path::Path;

use rstest::rstest;
use sevenz_rust::{SevenZArchiveEntry, SevenZWriter};
use tempfile::tempdir;

use vx_runtime_archive::ArchiveExtractor;

const CONTENT: &[u8] = b"archive fixture";

fn write_archive(path: &Path, name: &str) {
    let mut writer = SevenZWriter::new(Cursor::new(Vec::new())).unwrap();
    let mut entry = SevenZArchiveEntry::new();
    entry.name = name.into();
    entry.has_stream = true;
    writer
        .push_archive_entry(entry, Some(Cursor::new(CONTENT)))
        .unwrap();
    std::fs::write(path, writer.finish().unwrap().into_inner()).unwrap();
}

#[rstest]
#[case("runtime/bin/program")]
#[case("runtime\\bin\\program")]
fn test_sevenz_extracts_nested_entries(#[case] name: &str) {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("runtime.7z");
    let dest = temp.path().join("output");
    write_archive(&archive, name);

    ArchiveExtractor::new().extract(&archive, &dest).unwrap();

    assert_eq!(
        std::fs::read(dest.join("runtime/bin/program")).unwrap(),
        CONTENT
    );
}

#[rstest]
#[case("../escaped")]
#[case("..\\escaped")]
#[case("nested/../../escaped")]
#[case("nested\\..\\..\\escaped")]
fn test_sevenz_rejects_parent_traversal(#[case] name: &str) {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("unsafe.7z");
    let dest = temp.path().join("output");
    write_archive(&archive, name);

    let result = ArchiveExtractor::new().extract(&archive, &dest);

    assert!(!temp.path().join("escaped").exists());
    assert!(result.is_err());
    assert!(std::fs::read_dir(dest).unwrap().next().is_none());
}

#[rstest]
fn test_sevenz_rejects_absolute_paths() {
    let temp = tempdir().unwrap();
    let escaped = temp.path().join("escaped");
    let absolute = escaped.to_string_lossy().into_owned();
    let windows_absolute = if cfg!(windows) {
        absolute.clone()
    } else {
        format!("C:{absolute}")
    };
    // Even a broken implementation can only target this disposable directory.
    for (index, name) in [
        absolute,
        windows_absolute.replace('\\', "/"),
        windows_absolute.replace('/', "\\"),
    ]
    .into_iter()
    .enumerate()
    {
        let archive = temp.path().join(format!("unsafe-{index}.7z"));
        let dest = temp.path().join(format!("output-{index}"));
        write_archive(&archive, &name);

        let result = ArchiveExtractor::new().extract(&archive, &dest);

        assert!(!escaped.exists(), "{name}: wrote outside extraction root");
        assert!(result.is_err(), "{name}: accepted absolute entry path");
        assert!(std::fs::read_dir(dest).unwrap().next().is_none());
    }
}

#[cfg(unix)]
#[rstest]
#[case(false)]
#[case(true)]
fn test_sevenz_rejects_existing_symlink_paths(#[case] root_link: bool) {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("unsafe.7z");
    let dest = temp.path().join("output");
    let outside = temp.path().join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    let escaped = outside.join("escaped");
    std::fs::write(&escaped, b"original").unwrap();
    let name = if root_link {
        std::os::unix::fs::symlink(&outside, &dest).unwrap();
        "escaped"
    } else {
        std::fs::create_dir_all(&dest).unwrap();
        std::os::unix::fs::symlink(&outside, dest.join("alias")).unwrap();
        "alias/escaped"
    };
    write_archive(&archive, name);

    let result = ArchiveExtractor::new().extract(&archive, &dest);

    assert_eq!(std::fs::read(escaped).unwrap(), b"original");
    assert!(result.is_err());
}
