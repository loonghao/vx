//! Tests for cross-platform command shim generation and the shim registry.

use std::path::{Path, PathBuf};

use rstest::rstest;
use tempfile::TempDir;
use vx_runtime::{
    CommandShim, Platform, Shim, ShimRegistry, ShimType, VX_SHIM_MARKER, create_command_shim,
};

/// A fake `vx` launcher so tests never depend on a real vx install
fn fake_launcher(dir: &Path) -> PathBuf {
    let path = dir.join(if cfg!(windows) { "vx.exe" } else { "vx" });
    std::fs::write(&path, "fake launcher").expect("failed to write fake launcher");
    path
}

#[test]
fn test_platform_variants_cover_every_windows_shell() {
    let windows = Platform::new(vx_runtime::Os::Windows, vx_runtime::Arch::X86_64);

    let variants = ShimType::platform_variants(&windows);

    // cmd.exe/PowerShell reach the batch file through PATHEXT, Git Bash needs
    // the extension-less shell script — both must be generated.
    assert_eq!(variants, vec![ShimType::Batch, ShimType::Shell]);
}

#[test]
fn test_platform_variants_unix_need_only_a_shell_script() {
    let linux = Platform::new(vx_runtime::Os::Linux, vx_runtime::Arch::X86_64);
    let macos = Platform::new(vx_runtime::Os::MacOS, vx_runtime::Arch::Aarch64);

    assert_eq!(ShimType::platform_variants(&linux), vec![ShimType::Shell]);
    assert_eq!(ShimType::platform_variants(&macos), vec![ShimType::Shell]);
}

#[test]
fn test_create_all_writes_every_platform_variant() {
    let temp = TempDir::new().expect("failed to create temp dir");
    let launcher = fake_launcher(temp.path());
    let platform = Platform::current();

    let shim = Shim::new("jq", &launcher).with_args(&["jq"]);
    let written = shim
        .create_all(temp.path(), &platform)
        .expect("failed to create shims");

    assert_eq!(written.len(), ShimType::platform_variants(&platform).len());
    for path in &written {
        assert!(path.exists(), "expected {} to exist", path.display());
    }

    if platform.is_windows() {
        let names: Vec<String> = written
            .iter()
            .map(|path| {
                path.file_name()
                    .expect("file name")
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        assert!(names.contains(&"jq.cmd".to_string()), "got {:?}", names);
        assert!(names.contains(&"jq".to_string()), "got {:?}", names);
    }
}

#[test]
fn test_generated_shim_carries_the_vx_marker() {
    let temp = TempDir::new().expect("failed to create temp dir");
    let launcher = fake_launcher(temp.path());
    let platform = Platform::new(vx_runtime::Os::Windows, vx_runtime::Arch::X86_64);

    let shim = Shim::new("jq", &launcher).with_args(&["jq"]);
    let written = shim
        .create_all(temp.path(), &platform)
        .expect("failed to create shims");

    for path in written {
        let content = std::fs::read_to_string(&path).expect("failed to read shim");
        assert!(
            content.contains(VX_SHIM_MARKER),
            "missing marker in {}:\n{}",
            path.display(),
            content
        );
        assert!(
            Shim::is_managed(&path),
            "{} should be vx-managed",
            path.display()
        );
    }
}

#[test]
fn test_batch_shim_propagates_the_exit_code() {
    let shim = Shim::new("jq", "C:/tools/vx.exe").with_args(&["jq"]);

    let content = shim.content_for(ShimType::Batch);

    assert!(content.contains("exit /b %ERRORLEVEL%"), "{}", content);
    assert!(content.contains("\"C:/tools/vx.exe\" jq %*"), "{}", content);
}

#[test]
fn test_shell_shim_forwards_arguments_and_execs() {
    let shim = Shim::new("jq", "/usr/local/bin/vx").with_args(&["jq@1.8"]);

    let content = shim.content_for(ShimType::Shell);

    assert!(content.starts_with("#!/bin/sh"), "{}", content);
    assert!(
        content.contains("exec \"/usr/local/bin/vx\" jq@1.8 \"$@\""),
        "{}",
        content
    );
}

#[test]
fn test_shell_shim_emits_forward_slashes_for_windows_paths() {
    let shim = Shim::new("git", "C:\\Users\\tester\\.cargo\\bin\\vx.exe").with_args(&["git"]);

    let content = shim.content_for(ShimType::Shell);

    // Backslashes would be read as escapes by Git Bash / MSYS2
    assert!(!content.contains('\\'), "{}", content);
    assert!(
        content.contains("exec \"C:/Users/tester/.cargo/bin/vx.exe\" git \"$@\""),
        "{}",
        content
    );
}

#[test]
fn test_remove_all_only_deletes_vx_managed_files() {
    let temp = TempDir::new().expect("failed to create temp dir");
    let platform = Platform::current();
    let launcher = fake_launcher(temp.path());

    let shim = Shim::new("git", &launcher).with_args(&["git"]);
    let written = shim
        .create_all(temp.path(), &platform)
        .expect("failed to create shims");

    // A user-written wrapper with the same name must survive
    let user_file = temp.path().join("user-script");
    std::fs::write(&user_file, "#!/bin/sh\necho mine\n").expect("failed to write user file");
    let unmanaged = Shim::new("user-script", &launcher);
    std::fs::write(
        temp.path().join(unmanaged.file_name(&platform)),
        "hand written",
    )
    .expect("failed to write unmanaged shim");

    let removed = unmanaged
        .remove_all(temp.path(), &platform)
        .expect("remove_all should succeed");

    assert!(removed.is_empty(), "must not delete user files");
    assert!(
        temp.path().join(unmanaged.file_name(&platform)).exists(),
        "user file was deleted"
    );

    let removed = shim
        .remove_all(temp.path(), &platform)
        .expect("failed to remove own shims");
    assert_eq!(removed.len(), written.len());
    for path in &removed {
        assert!(!path.exists());
    }
    assert!(user_file.exists(), "unrelated file must be untouched");
}

#[test]
fn test_shim_file_name_follows_the_platform() {
    let windows = Platform::new(vx_runtime::Os::Windows, vx_runtime::Arch::X86_64);
    let linux = Platform::new(vx_runtime::Os::Linux, vx_runtime::Arch::X86_64);
    let shim = Shim::new("bunx", "/path/to/bun");

    assert_eq!(shim.file_name(&windows), "bunx.cmd");
    assert_eq!(shim.file_name(&linux), "bunx");
}

#[test]
fn test_generated_shim_applies_environment_variables() {
    let shim = Shim::new("test", "/path/to/exe")
        .with_env("FOO", "bar")
        .with_env("BAZ", "qux");

    let shell = shim.content_for(ShimType::Shell);
    assert!(shell.contains("FOO='bar'"), "{}", shell);
    assert!(shell.contains("export FOO"), "{}", shell);
    assert!(shell.contains("BAZ='qux'"), "{}", shell);

    let batch = shim.content_for(ShimType::Batch);
    assert!(batch.contains("set \"FOO=bar\""), "{}", batch);
    assert!(batch.contains("set \"BAZ=qux\""), "{}", batch);
}

#[test]
fn test_generated_shim_applies_the_working_directory() {
    let shim = Shim::new("test", "/path/to/exe").with_working_dir("/working/dir");

    assert!(
        shim.content_for(ShimType::Shell)
            .contains("cd \"/working/dir\"")
    );
    assert!(
        shim.content_for(ShimType::Batch)
            .contains("cd /d \"/working/dir\"")
    );
}

#[test]
fn test_create_marks_the_shell_script_executable_on_unix() {
    let temp = TempDir::new().expect("failed to create temp dir");
    let shim = Shim::new("test-shim", temp.path().join("target")).with_args(&["arg1"]);

    let path = shim
        .create(temp.path(), &Platform::current())
        .expect("failed to create shim");

    assert!(path.exists());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path)
            .expect("failed to stat shim")
            .permissions()
            .mode();
        assert_ne!(mode & 0o111, 0, "shim must be executable");
    }
}

#[test]
fn test_shim_builder_creates_every_forwarding_shim() {
    let temp = TempDir::new().expect("failed to create temp dir");

    let paths = vx_runtime::ShimBuilder::new()
        .dir(temp.path())
        .forward("shim1", temp.path().join("exe1"), &["arg1"])
        .forward("shim2", temp.path().join("exe2"), &["arg2", "arg3"])
        .build()
        .expect("failed to build shims");

    assert_eq!(paths.len(), 2);

    let names: Vec<String> = paths
        .iter()
        .map(|path| {
            path.file_name()
                .expect("file name")
                .to_string_lossy()
                .to_string()
        })
        .collect();
    if cfg!(windows) {
        assert!(names.contains(&"shim1.cmd".to_string()), "{:?}", names);
        assert!(names.contains(&"shim2.cmd".to_string()), "{:?}", names);
    } else {
        assert!(names.contains(&"shim1".to_string()), "{:?}", names);
        assert!(names.contains(&"shim2".to_string()), "{:?}", names);
    }
}

#[rstest]
#[case("jq", "jq")]
#[case("git", "git")]
#[case("node@22", "node")]
fn test_create_command_shim_records_runtime_and_files(
    #[case] runtime: &str,
    #[case] expected_name: &str,
) {
    let temp = TempDir::new().expect("failed to create temp dir");
    let launcher = fake_launcher(temp.path());
    let dirs = vec![temp.path().join("bin")];
    let platform = Platform::current();

    let entry = create_command_shim(expected_name, runtime, &launcher, &dirs, &platform)
        .expect("failed to create command shim");

    assert_eq!(entry.name, expected_name);
    assert_eq!(entry.runtime, runtime);
    assert_eq!(entry.launcher, launcher);
    assert_eq!(entry.dirs, dirs);
    assert!(entry.is_complete());
    assert!(!entry.created_at.is_empty());

    let content = std::fs::read_to_string(&entry.files[0]).expect("failed to read shim");
    assert!(content.contains(VX_SHIM_MARKER));
    assert!(content.contains(runtime));
}

#[test]
fn test_registry_round_trip_is_sorted() {
    let temp = TempDir::new().expect("failed to create temp dir");
    let file = temp.path().join("command-shims.json");

    let mut registry = ShimRegistry::default();
    registry.upsert(sample_shim("jq", "jq"));
    registry.upsert(sample_shim("git", "git@2.53.0"));
    registry.save(&file).expect("failed to save registry");

    let loaded = ShimRegistry::load(&file).expect("failed to load registry");

    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded.names(), vec!["git", "jq"]);
    assert!(loaded.contains("jq"));
    assert!(!loaded.contains("rg"));
}

#[test]
fn test_registry_upsert_replaces_and_keeps_sorted() {
    let mut registry = ShimRegistry::default();
    registry.upsert(sample_shim("jq", "jq"));
    registry.upsert(sample_shim("jq", "jq@1.8"));

    assert_eq!(registry.len(), 1);
    assert_eq!(registry.get("jq").expect("missing shim").runtime, "jq@1.8");
}

#[test]
fn test_registry_remove_returns_the_entry() {
    let mut registry = ShimRegistry::default();
    registry.upsert(sample_shim("jq", "jq"));

    let removed = registry.remove("jq").expect("expected an entry");
    assert_eq!(removed.name, "jq");
    assert!(registry.is_empty());
    assert!(registry.remove("jq").is_none());
}

#[test]
fn test_load_or_default_survives_a_missing_file() {
    let temp = TempDir::new().expect("failed to create temp dir");

    let registry = ShimRegistry::load_or_default(&temp.path().join("nope.json"));

    assert!(registry.is_empty());
}

#[test]
fn test_load_or_default_survives_a_corrupt_file() {
    let temp = TempDir::new().expect("failed to create temp dir");
    let file = temp.path().join("command-shims.json");
    std::fs::write(&file, "{ not json").expect("failed to write corrupt file");

    let registry = ShimRegistry::load_or_default(&file);

    assert!(registry.is_empty());
}

#[test]
fn test_is_complete_detects_deleted_files() {
    let temp = TempDir::new().expect("failed to create temp dir");
    let shim = create_command_shim(
        "jq",
        "jq",
        &fake_launcher(temp.path()),
        &[temp.path().join("bin")],
        &Platform::current(),
    )
    .expect("failed to create command shim");

    assert!(shim.is_complete());
    std::fs::remove_file(&shim.files[0]).expect("failed to remove shim file");
    assert!(!shim.is_complete());
}

/// A registry entry with placeholder paths
fn sample_shim(name: &str, runtime: &str) -> CommandShim {
    CommandShim {
        name: name.to_string(),
        runtime: runtime.to_string(),
        launcher: PathBuf::from("/usr/local/bin/vx"),
        vx_home: None,
        dirs: vec![PathBuf::from("/usr/local/bin")],
        files: vec![PathBuf::from("/usr/local/bin").join(name)],
        created_at: "2026-01-01T00:00:00Z".to_string(),
    }
}
