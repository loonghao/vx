//! ZIP extraction contracts through the public API.

use std::fs::File;
use std::io::{Cursor, Write};
use std::path::Path;

#[cfg(unix)]
use rstest::rstest;
use tempfile::tempdir;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use vx_runtime_archive::ArchiveExtractor;

fn write_zip(path: &Path, entries: &[(&str, u32, &[u8])]) {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, mode, contents) in entries {
        writer
            .start_file(*name, SimpleFileOptions::default().unix_permissions(*mode))
            .unwrap();
        writer.write_all(contents).unwrap();
    }
    let mut bytes = writer.finish().unwrap().into_inner();
    // ZipWriter strips special bits. Patch the central metadata so the fixture
    // really contains the untrusted modes that extraction must sanitize.
    let mut offset = bytes
        .windows(4)
        .position(|part| part == b"PK\x01\x02")
        .unwrap();
    for (_, mode, _) in entries {
        assert_eq!(&bytes[offset..offset + 4], b"PK\x01\x02");
        let field = |index| u16::from_le_bytes([bytes[index], bytes[index + 1]]) as usize;
        let entry_size = 46 + field(offset + 28) + field(offset + 30) + field(offset + 32);
        bytes[offset + 38..offset + 42].copy_from_slice(&(mode << 16).to_le_bytes());
        offset += entry_size;
    }
    std::fs::write(path, bytes).unwrap();
    let mut archive = zip::ZipArchive::new(File::open(path).unwrap()).unwrap();
    for (index, (_, mode, _)) in entries.iter().enumerate() {
        assert_eq!(archive.by_index(index).unwrap().unix_mode(), Some(*mode));
    }
}

#[test]
fn test_zip_extracts_nested_app_bundle_files() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("bundle.zip");
    let dest = temp.path().join("output");
    let helper = "Example.app/Contents/MacOS/Example";
    let resource = "Example.app/Contents/Resources/settings.json";
    write_zip(
        &archive,
        &[
            (helper, 0o100755, b"helper"),
            (resource, 0o100644, b"resource"),
        ],
    );

    ArchiveExtractor::new().extract(&archive, &dest).unwrap();

    assert_eq!(std::fs::read(dest.join(helper)).unwrap(), b"helper");
    assert_eq!(std::fs::read(dest.join(resource)).unwrap(), b"resource");
}

#[cfg(unix)]
#[rstest]
#[case("Example.app/Contents/MacOS/Example", 0o100755, 0o755)]
#[case("Example.app/Contents/Resources/settings.json", 0o100644, 0o644)]
#[case("Example.app/Contents/Helpers/update", 0o107755, 0o755)]
#[test]
fn test_zip_preserves_only_regular_rwx_permissions(
    #[case] name: &str,
    #[case] archived_mode: u32,
    #[case] expected_mode: u32,
) {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().unwrap();
    let archive = temp.path().join("bundle.zip");
    let dest = temp.path().join("output");
    write_zip(&archive, &[(name, archived_mode, b"contents")]);

    ArchiveExtractor::new().extract(&archive, &dest).unwrap();

    assert_eq!(
        std::fs::metadata(dest.join(name))
            .unwrap()
            .permissions()
            .mode()
            & 0o7777,
        expected_mode
    );
}

#[test]
fn test_zip_rejects_path_traversal() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("bundle.zip");
    let dest = temp.path().join("output");
    write_zip(
        &archive,
        &[
            ("../escaped", 0o100755, b"outside"),
            ("inside", 0o100644, b"inside"),
        ],
    );

    ArchiveExtractor::new().extract(&archive, &dest).unwrap();

    assert!(!temp.path().join("escaped").exists());
    assert_eq!(std::fs::read(dest.join("inside")).unwrap(), b"inside");
}

#[cfg(unix)]
#[test]
fn test_zip_symlink_metadata_does_not_grant_executable_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().unwrap();
    let archive = temp.path().join("bundle.zip");
    let dest = temp.path().join("output");
    let mut writer = ZipWriter::new(File::create(&archive).unwrap());
    writer
        .start_file(
            "resource",
            SimpleFileOptions::default().unix_permissions(0o644),
        )
        .unwrap();
    writer.write_all(b"resource").unwrap();
    writer
        .add_symlink(
            "link",
            "resource",
            SimpleFileOptions::default().unix_permissions(0o777),
        )
        .unwrap();
    writer.finish().unwrap();

    ArchiveExtractor::new().extract(&archive, &dest).unwrap();

    assert_eq!(
        std::fs::metadata(dest.join("link"))
            .unwrap()
            .permissions()
            .mode()
            & 0o111,
        0
    );
    assert_eq!(
        std::fs::metadata(dest.join("resource"))
            .unwrap()
            .permissions()
            .mode()
            & 0o7777,
        0o644
    );
}
