//! ZIP extraction contracts through the public API.

use std::fs::File;
use std::io::{Cursor, Write};
use std::path::Path;

#[cfg(unix)]
use rstest::rstest;
use tempfile::tempdir;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use vx_runtime::Installer;
use vx_runtime_http::RealInstaller;

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

#[tokio::test]
async fn test_zip_extracts_nested_app_bundle_files() {
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

    RealInstaller::new().extract(&archive, &dest).await.unwrap();

    assert_eq!(std::fs::read(dest.join(helper)).unwrap(), b"helper");
    assert_eq!(std::fs::read(dest.join(resource)).unwrap(), b"resource");
}

#[cfg(unix)]
#[rstest]
#[case("Example.app/Contents/MacOS/Example", 0o100755, 0o755)]
#[case("Example.app/Contents/Resources/settings.json", 0o100644, 0o644)]
#[case("Example.app/Contents/Helpers/update", 0o107755, 0o755)]
#[tokio::test]
async fn test_zip_preserves_only_regular_rwx_permissions(
    #[case] name: &str,
    #[case] archived_mode: u32,
    #[case] expected_mode: u32,
) {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().unwrap();
    let archive = temp.path().join("bundle.zip");
    let dest = temp.path().join("output");
    write_zip(&archive, &[(name, archived_mode, b"contents")]);

    RealInstaller::new().extract(&archive, &dest).await.unwrap();

    assert_eq!(
        std::fs::metadata(dest.join(name))
            .unwrap()
            .permissions()
            .mode()
            & 0o7777,
        expected_mode
    );
}

// Symlink extraction must preserve framework layout without enabling writes
// through archive-created or pre-existing links.
#[cfg(unix)]
#[tokio::test]
async fn test_zip_preserves_electron_framework_chain_and_internal_parent_link() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("framework.zip");
    let dest = temp.path().join("output");
    let framework = "Example.app/Contents/Frameworks/Electron Framework.framework";
    let binary = format!("{framework}/Versions/A/Electron Framework");
    let current = format!("{framework}/Versions/Current");
    let library = format!("{framework}/Electron Framework");
    let resource = format!("{framework}/Versions/A/Resources/config.json");
    let resources = format!("{framework}/Resources");
    let helper = "Example.app/Contents/Helpers/renderer";
    write_zip(
        &archive,
        &[
            (&library, 0o120777, b"Versions/Current/Electron Framework"),
            (&current, 0o120777, b"A"),
            (&resources, 0o120777, b"Versions/Current/Resources"),
            (
                helper,
                0o120777,
                b"../Frameworks/Electron Framework.framework/Electron Framework",
            ),
            (&binary, 0o100755, b"mach-o fixture"),
            (&resource, 0o100644, b"resource"),
        ],
    );

    RealInstaller::new().extract(&archive, &dest).await.unwrap();

    for name in [
        library.as_str(),
        current.as_str(),
        resources.as_str(),
        helper,
    ] {
        assert!(
            std::fs::symlink_metadata(dest.join(name))
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }
    assert_eq!(
        std::fs::read(dest.join(&library)).unwrap(),
        b"mach-o fixture"
    );
    assert_eq!(std::fs::read(dest.join(helper)).unwrap(), b"mach-o fixture");
    assert_eq!(
        std::fs::read(dest.join(&resources).join("config.json")).unwrap(),
        b"resource"
    );
}

#[cfg(unix)]
#[rstest]
#[case("../outside")]
#[case("/outside")]
#[case("C:/outside")]
#[case("C:\\outside")]
#[case("..\\outside")]
#[case("alias/../outside")]
#[case("alias/./../outside")]
#[tokio::test]
async fn test_zip_rejects_escaping_or_noncanonical_symlink_target(#[case] target: &str) {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("unsafe-link.zip");
    let dest = temp.path().join("output");
    write_zip(
        &archive,
        &[
            ("escape", 0o120777, target.as_bytes()),
            ("alias", 0o120777, b"."),
        ],
    );

    assert!(RealInstaller::new().extract(&archive, &dest).await.is_err());
    assert!(std::fs::symlink_metadata(dest.join("escape")).is_err());
}

#[cfg(unix)]
#[tokio::test]
async fn test_zip_rejects_forward_symlink_cycle() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("cycle.zip");
    let dest = temp.path().join("output");
    write_zip(
        &archive,
        &[
            ("link-a", 0o120777, b"link-b"),
            ("link-b", 0o120777, b"link-a"),
        ],
    );

    assert!(RealInstaller::new().extract(&archive, &dest).await.is_err());
    assert!(std::fs::symlink_metadata(dest.join("link-a")).is_err());
    assert!(std::fs::symlink_metadata(dest.join("link-b")).is_err());
}

#[cfg(unix)]
#[tokio::test]
async fn test_zip_rejects_existing_target_chain_outside_root() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("target-chain.zip");
    let dest = temp.path().join("output");
    let outside = temp.path().join("outside");
    std::fs::create_dir(&dest).unwrap();
    std::fs::write(&outside, b"untouched").unwrap();
    std::os::unix::fs::symlink(&outside, dest.join("existing")).unwrap();
    write_zip(&archive, &[("new-link", 0o120777, b"existing")]);

    assert!(RealInstaller::new().extract(&archive, &dest).await.is_err());
    assert!(std::fs::symlink_metadata(dest.join("new-link")).is_err());
    assert_eq!(std::fs::read(outside).unwrap(), b"untouched");
}

#[cfg(unix)]
#[rstest]
#[case(false)]
#[case(true)]
#[tokio::test]
async fn test_zip_rejects_preexisting_symlink_output_paths(#[case] leaf: bool) {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("output-link.zip");
    let dest = temp.path().join("output");
    let outside = temp.path().join("outside");
    std::fs::create_dir(&dest).unwrap();
    if leaf {
        std::fs::write(&outside, b"untouched").unwrap();
    } else {
        std::fs::create_dir(&outside).unwrap();
    }
    std::os::unix::fs::symlink(&outside, dest.join("existing")).unwrap();
    let name = if leaf { "existing" } else { "existing/escaped" };
    write_zip(&archive, &[(name, 0o100644, b"must not write")]);

    assert!(RealInstaller::new().extract(&archive, &dest).await.is_err());
    if leaf {
        assert_eq!(std::fs::read(outside).unwrap(), b"untouched");
    } else {
        assert!(!outside.join("escaped").exists());
    }
}

#[cfg(unix)]
#[rstest]
#[case("./link")]
#[case("link/escaped")]
#[tokio::test]
async fn test_zip_rejects_later_entry_through_archive_symlink(#[case] name: &str) {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("duplicate-link.zip");
    let dest = temp.path().join("output");
    write_zip(
        &archive,
        &[
            ("link", 0o120777, b"."),
            (name, 0o100644, b"must not write"),
        ],
    );

    assert!(RealInstaller::new().extract(&archive, &dest).await.is_err());
    assert!(
        std::fs::symlink_metadata(dest.join("link"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert!(!dest.join("escaped").exists());
}

#[cfg(not(unix))]
#[tokio::test]
async fn test_zip_reports_unsupported_symlinks_without_materializing_target_text() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("symlink.zip");
    let dest = temp.path().join("output");
    write_zip(&archive, &[("link", 0o120777, b"resource")]);

    let error = RealInstaller::new()
        .extract(&archive, &dest)
        .await
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("symlink extraction is unsupported")
    );
    assert!(!dest.join("link").exists());
}

#[tokio::test]
async fn test_zip_rejects_path_traversal() {
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

    RealInstaller::new().extract(&archive, &dest).await.unwrap();

    assert!(!temp.path().join("escaped").exists());
    assert_eq!(std::fs::read(dest.join("inside")).unwrap(), b"inside");
}

#[cfg(unix)]
#[tokio::test]
async fn test_zip_symlink_metadata_does_not_grant_executable_permissions() {
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

    RealInstaller::new().extract(&archive, &dest).await.unwrap();

    assert!(
        std::fs::symlink_metadata(dest.join("link"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
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
