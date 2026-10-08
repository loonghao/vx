//! ZIP extraction contracts through the public API.

use std::fs::File;
use std::io::{Cursor, Write};
use std::path::Path;

#[cfg(any(unix, windows))]
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

// Symlink extraction must preserve framework layout without enabling writes
// through archive-created or pre-existing links.
#[cfg(unix)]
#[test]
fn test_zip_preserves_electron_framework_chain_and_internal_parent_link() {
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

    ArchiveExtractor::new().extract(&archive, &dest).unwrap();

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
#[test]
fn test_zip_rejects_escaping_or_noncanonical_symlink_target(#[case] target: &str) {
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

    assert!(ArchiveExtractor::new().extract(&archive, &dest).is_err());
    assert!(std::fs::symlink_metadata(dest.join("escape")).is_err());
}

#[cfg(unix)]
#[test]
fn test_zip_rejects_forward_symlink_cycle() {
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

    assert!(ArchiveExtractor::new().extract(&archive, &dest).is_err());
    assert!(std::fs::symlink_metadata(dest.join("link-a")).is_err());
    assert!(std::fs::symlink_metadata(dest.join("link-b")).is_err());
}

#[cfg(unix)]
#[test]
fn test_zip_rejects_existing_target_chain_outside_root() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("target-chain.zip");
    let dest = temp.path().join("output");
    let outside = temp.path().join("outside");
    std::fs::create_dir(&dest).unwrap();
    std::fs::write(&outside, b"untouched").unwrap();
    std::os::unix::fs::symlink(&outside, dest.join("existing")).unwrap();
    write_zip(&archive, &[("new-link", 0o120777, b"existing")]);

    assert!(ArchiveExtractor::new().extract(&archive, &dest).is_err());
    assert!(std::fs::symlink_metadata(dest.join("new-link")).is_err());
    assert_eq!(std::fs::read(outside).unwrap(), b"untouched");
}

#[cfg(unix)]
#[rstest]
#[case(false)]
#[case(true)]
#[test]
fn test_zip_rejects_preexisting_symlink_output_paths(#[case] leaf: bool) {
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

    assert!(ArchiveExtractor::new().extract(&archive, &dest).is_err());
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
#[test]
fn test_zip_rejects_later_entry_through_archive_symlink(#[case] name: &str) {
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

    assert!(ArchiveExtractor::new().extract(&archive, &dest).is_err());
    assert!(
        std::fs::symlink_metadata(dest.join("link"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert!(!dest.join("escaped").exists());
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

#[cfg(windows)]
#[rstest]
#[case(true)]
#[case(false)]
#[test]
fn test_zip_windows_materializes_google_cloud_file_and_implicit_directory_links(
    #[case] targets_first: bool,
) {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("google-cloud.zip");
    let dest = temp.path().join("output");
    let links: Vec<(&str, u32, &[u8])> = vec![
        ("funcsigs/docs/index.rst", 0o120777, b"../README.rst"),
        (
            "google-auth-library-python-httplib2/docs/CHANGELOG.md",
            0o120777,
            b"../CHANGELOG.md",
        ),
        (
            "google-auth-library-python-httplib2/docs/README.rst",
            0o120777,
            b"../README.rst",
        ),
        ("mock/docs/changelog.txt", 0o120777, b"../ChangeLog"),
        (
            "requests/tests/certs/mtls/client/ca",
            0o120777,
            b"../../expired/ca",
        ),
        ("requests/tests/certs/valid/ca", 0o120777, b"../expired/ca"),
    ];
    let targets: Vec<(&str, u32, &[u8])> = vec![
        ("funcsigs/README.rst", 0o100644, b"funcsigs documentation"),
        (
            "google-auth-library-python-httplib2/CHANGELOG.md",
            0o100644,
            b"auth changelog",
        ),
        (
            "google-auth-library-python-httplib2/README.rst",
            0o100644,
            b"auth documentation",
        ),
        ("mock/ChangeLog", 0o100644, b"mock changelog"),
        (
            "requests/tests/certs/expired/ca/Makefile",
            0o100644,
            b"make certificate",
        ),
        (
            "requests/tests/certs/expired/ca/ca-private.key",
            0o100644,
            b"fixture key",
        ),
        (
            "requests/tests/certs/expired/ca/ca.cnf",
            0o100644,
            b"fixture config",
        ),
        (
            "requests/tests/certs/expired/ca/ca.crt",
            0o100644,
            b"fixture certificate",
        ),
        (
            "requests/tests/certs/expired/ca/ca.srl",
            0o100644,
            b"fixture serial",
        ),
    ];
    let entries = if targets_first {
        [targets.clone(), links.clone()].concat()
    } else {
        [links.clone(), targets.clone()].concat()
    };
    write_zip(&archive, &entries);
    ArchiveExtractor::new().extract(&archive, &dest).unwrap();
    for (index, (name, _, _)) in links[..4].iter().enumerate() {
        let contents = std::fs::read(dest.join(name)).unwrap();
        assert_eq!(contents, targets[index].2);
        assert!(
            std::fs::symlink_metadata(dest.join(name))
                .unwrap()
                .is_file()
        );
    }
    for directory in [
        "requests/tests/certs/mtls/client/ca",
        "requests/tests/certs/valid/ca",
    ] {
        for (name, _, contents) in &targets[4..] {
            let name = Path::new(name).file_name().unwrap();
            assert_eq!(
                std::fs::read(dest.join(directory).join(name)).unwrap(),
                *contents
            );
        }
        assert!(
            !std::fs::symlink_metadata(dest.join(directory))
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }
}

#[cfg(windows)]
#[test]
fn test_zip_windows_resolves_forward_chains_and_directory_link_contents() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("forward.zip");
    let dest = temp.path().join("output");
    write_zip(
        &archive,
        &[
            ("copied", 0o120777, b"contents"),
            ("contents/alias", 0o120777, b"../final"),
            ("final", 0o120777, b"payload"),
            ("payload", 0o100644, b"actual bytes"),
            ("contents/nested/late", 0o100644, b"late bytes"),
        ],
    );
    ArchiveExtractor::new().extract(&archive, &dest).unwrap();
    assert_eq!(
        std::fs::read(dest.join("copied/alias")).unwrap(),
        b"actual bytes"
    );
    assert_eq!(
        std::fs::read(dest.join("copied/nested/late")).unwrap(),
        b"late bytes"
    );
    assert_eq!(std::fs::read(dest.join("final")).unwrap(), b"actual bytes");
}

#[cfg(windows)]
#[test]
fn test_zip_windows_resolves_case_varied_directory_targets_before_copying() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("case-varied.zip");
    let dest = temp.path().join("output");
    write_zip(
        &archive,
        &[
            ("copied", 0o120777, b"actualdir"),
            ("ActualDir/helper", 0o120777, b"../payload.txt"),
            ("ActualDir/keep.txt", 0o100644, b"keep bytes"),
            ("payload.txt", 0o100644, b"actual bytes"),
        ],
    );
    ArchiveExtractor::new().extract(&archive, &dest).unwrap();
    assert_eq!(
        std::fs::read(dest.join("copied/helper")).unwrap(),
        b"actual bytes"
    );
    assert_eq!(
        std::fs::read(dest.join("copied/keep.txt")).unwrap(),
        b"keep bytes"
    );
}

#[cfg(windows)]
#[rstest]
#[case("../outside")]
#[case("/outside")]
#[case("C:/outside")]
#[case("C:\\outside")]
#[case("..\\outside")]
#[case("alias/../outside")]
#[case("alias/./../outside")]
#[test]
fn test_zip_windows_rejects_unsafe_link_targets(#[case] target: &str) {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("unsafe.zip");
    let dest = temp.path().join("output");
    write_zip(&archive, &[("escape", 0o120777, target.as_bytes())]);
    assert!(ArchiveExtractor::new().extract(&archive, &dest).is_err());
    assert!(!dest.join("escape").exists());
}

#[cfg(windows)]
#[rstest]
#[case("missing")]
#[case("cycle")]
#[case("self_copy")]
#[case("directory_cycle")]
#[case("partial_failure")]
#[test]
fn test_zip_windows_rejects_missing_cyclic_and_self_containing_links(#[case] scenario: &str) {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("invalid.zip");
    let dest = temp.path().join("output");
    let entries: Vec<(&str, u32, &[u8])> = match scenario {
        "missing" => vec![("link", 0o120777, b"missing")],
        "cycle" => vec![("link", 0o120777, b"second"), ("second", 0o120777, b"link")],
        "self_copy" => vec![("link", 0o120777, b".")],
        "partial_failure" => vec![
            ("good", 0o120777, b"payload"),
            ("payload", 0o100644, b"good bytes"),
            ("link", 0o120777, b"missing"),
        ],
        "directory_cycle" => vec![("a/link", 0o120777, b"../b"), ("b/link", 0o120777, b"../a")],
        _ => unreachable!(),
    };
    write_zip(&archive, &entries);
    assert!(ArchiveExtractor::new().extract(&archive, &dest).is_err());
    for name in ["link", "second", "a/link", "b/link", "good"] {
        assert!(!dest.join(name).exists());
    }
}

#[cfg(windows)]
#[rstest]
#[case("./link")]
#[case("link/escaped")]
#[test]
fn test_zip_windows_rejects_entries_through_pending_links(#[case] name: &str) {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("duplicate.zip");
    let dest = temp.path().join("output");
    write_zip(
        &archive,
        &[
            ("link", 0o120777, b"target"),
            (name, 0o100644, b"escaped"),
            ("target", 0o100644, b"original"),
        ],
    );
    assert!(ArchiveExtractor::new().extract(&archive, &dest).is_err());
    assert!(!dest.join("link").exists());
}

#[cfg(windows)]
#[test]
fn test_zip_windows_never_overwrites_existing_link_destination() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("existing.zip");
    let dest = temp.path().join("output");
    std::fs::create_dir_all(&dest).unwrap();
    std::fs::write(dest.join("link"), b"keep existing bytes").unwrap();
    write_zip(
        &archive,
        &[
            ("link", 0o120777, b"target"),
            ("target", 0o100644, b"replacement"),
        ],
    );
    assert!(ArchiveExtractor::new().extract(&archive, &dest).is_err());
    assert_eq!(
        std::fs::read(dest.join("link")).unwrap(),
        b"keep existing bytes"
    );
}

#[cfg(windows)]
#[rstest]
#[case("source")]
#[case("source_child")]
#[case("output_ancestor")]
#[case("output_leaf")]
#[test]
fn test_zip_windows_rejects_preexisting_junctions(#[case] scenario: &str) {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("junction.zip");
    let dest = temp.path().join("output");
    let outside = temp.path().join("outside");
    std::fs::create_dir_all(&dest).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("keep"), b"untouched").unwrap();
    let junction = if scenario == "source_child" {
        std::fs::create_dir_all(dest.join("source")).unwrap();
        dest.join("source").join("junction")
    } else {
        dest.join("junction")
    };
    // A directory junction works without Developer Mode or symlink privileges.
    let output = std::process::Command::new("cmd")
        .args(["/D", "/C", "mklink", "/J"])
        .arg(&junction)
        .arg(&outside)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let entries: Vec<(&str, u32, &[u8])> = match scenario {
        "source" => vec![("copied", 0o120777, b"junction/keep")],
        "source_child" => vec![("copied", 0o120777, b"source")],
        "output_ancestor" => vec![("junction/keep", 0o100644, b"changed")],
        "output_leaf" => vec![("junction", 0o100644, b"changed")],
        _ => unreachable!(),
    };
    write_zip(&archive, &entries);
    assert!(ArchiveExtractor::new().extract(&archive, &dest).is_err());
    assert_eq!(std::fs::read(outside.join("keep")).unwrap(), b"untouched");
    assert!(!dest.join("copied").exists());
    std::fs::remove_dir(junction).unwrap();
}
