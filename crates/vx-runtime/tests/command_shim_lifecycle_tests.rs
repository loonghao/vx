//! Ownership and environment contracts for globally exposed command shims.

use std::path::Path;
use std::process::Command;

use rstest::rstest;
use tempfile::TempDir;
use vx_runtime::{
    CommandShim, Platform, ShimRegistry, ShimType, create_command_shim,
    create_command_shim_in_home, validate_command_shim_name,
};

#[rstest]
#[case("")]
#[case(".")]
#[case("..")]
#[case("../outside")]
#[case("..\\outside")]
#[case("C:outside")]
#[case("bad\nname")]
#[case("bad name")]
#[case("bad&name")]
#[case("vx")]
#[case("VX")]
#[case("vx.exe")]
#[case("con")]
#[case("NUL.txt")]
#[case("COM1")]
#[case("lpt9")]
#[case("tool.")]
fn invalid_names_are_rejected_before_writing(#[case] name: &str) {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("commands");
    assert!(validate_command_shim_name(name).is_err());
    assert!(
        create_command_shim_in_home(
            name,
            "codex",
            &temp.path().join("vx"),
            temp.path(),
            std::slice::from_ref(&output),
            &Platform::current(),
            None,
        )
        .is_err()
    );
    assert!(!output.exists());
}

#[test]
fn every_destination_is_checked_before_writing_any_variant() {
    let temp = TempDir::new().unwrap();
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    std::fs::create_dir_all(&second).unwrap();
    let collision = second.join("codex");
    std::fs::write(&collision, "user command mentioning vx-shim").unwrap();
    let windows = Platform::new(vx_runtime::Os::Windows, vx_runtime::Arch::X86_64);
    let result = create_command_shim_in_home(
        "codex",
        "codex",
        &temp.path().join("vx"),
        temp.path(),
        &[first.clone(), second.clone()],
        &windows,
        None,
    );
    assert!(result.is_err());
    assert!(!first.exists());
    assert!(!second.join("codex.cmd").exists());
    assert_eq!(
        std::fs::read_to_string(collision).unwrap(),
        "user command mentioning vx-shim"
    );
}

#[test]
fn exact_legacy_record_migrates_and_refreshes_against_new_launcher() {
    let temp = TempDir::new().unwrap();
    let dirs = [temp.path().join("commands")];
    let legacy = create_command_shim(
        "codex",
        "codex",
        &temp.path().join("old/vx"),
        &dirs,
        &Platform::current(),
    )
    .unwrap();
    let legacy_json = serde_json::to_string(&legacy).unwrap();
    assert!(!legacy_json.contains("vx_home"));
    let previous: CommandShim = serde_json::from_str(&legacy_json).unwrap();
    assert!(previous.vx_home.is_none());
    let launcher = temp.path().join("new/vx");
    let migrated = create_command_shim_in_home(
        "codex",
        "codex",
        &launcher,
        temp.path(),
        &dirs,
        &Platform::current(),
        Some(&previous),
    )
    .unwrap();
    assert_eq!(migrated.vx_home.as_deref(), Some(temp.path()));
    assert_eq!(migrated.launcher, launcher);
    assert_eq!(migrated.created_at, legacy.created_at);
    assert!(migrated.is_complete());
    assert!(!legacy.is_complete());
    let repeated = create_command_shim_in_home(
        "codex",
        "codex",
        &launcher,
        temp.path(),
        &dirs,
        &Platform::current(),
        Some(&migrated),
    )
    .unwrap();
    assert_eq!(repeated, migrated);
}

#[test]
fn edited_legacy_wrapper_is_not_migrated_or_removed() {
    let temp = TempDir::new().unwrap();
    let dirs = [temp.path().join("commands")];
    let legacy = create_command_shim(
        "codex",
        "codex",
        &temp.path().join("vx"),
        &dirs,
        &Platform::current(),
    )
    .unwrap();
    let edited = format!(
        "{}\n# user edit",
        std::fs::read_to_string(&legacy.files[0]).unwrap()
    );
    std::fs::write(&legacy.files[0], &edited).unwrap();
    assert!(!legacy.is_complete());
    assert!(!legacy.owns_file(&legacy.files[0]));
    assert!(
        create_command_shim_in_home(
            "codex",
            "codex",
            &temp.path().join("vx"),
            temp.path(),
            &dirs,
            &Platform::current(),
            Some(&legacy),
        )
        .is_err()
    );
    legacy.remove_files().unwrap();
    assert_eq!(std::fs::read_to_string(&legacy.files[0]).unwrap(), edited);
}

#[test]
fn another_home_cannot_take_over_or_delete_a_wrapper() {
    let temp = TempDir::new().unwrap();
    let dirs = [temp.path().join("commands")];
    let launcher = temp.path().join("vx");
    let first_home = temp.path().join("first-home");
    let second_home = temp.path().join("second-home");
    let first = create_command_shim_in_home(
        "codex",
        "codex",
        &launcher,
        &first_home,
        &dirs,
        &Platform::current(),
        None,
    )
    .unwrap();
    assert!(
        create_command_shim_in_home(
            "codex",
            "codex",
            &launcher,
            &second_home,
            &dirs,
            &Platform::current(),
            Some(&first),
        )
        .is_err()
    );
    assert!(
        create_command_shim_in_home(
            "codex",
            "codex",
            &launcher,
            &second_home,
            &dirs,
            &Platform::current(),
            None,
        )
        .is_err()
    );

    let mut other = first.clone();
    other.vx_home = Some(second_home);
    assert!(other.remove_files().unwrap().is_empty());
    assert!(first.is_complete());
    assert_eq!(first.remove_files().unwrap(), first.files);
    assert!(first.remove_files().unwrap().is_empty());
}

#[test]
fn ownership_requires_expected_recorded_paths() {
    let temp = TempDir::new().unwrap();
    let dirs = [temp.path().join("commands")];
    let mut entry = create_command_shim_in_home(
        "codex",
        "codex",
        &temp.path().join("vx"),
        temp.path(),
        &dirs,
        &Platform::current(),
        None,
    )
    .unwrap();
    let outside = temp.path().join("unrelated");
    std::fs::copy(&entry.files[0], &outside).unwrap();
    entry.files = vec![outside.clone()];
    assert!(!entry.is_complete());
    assert!(entry.remove_files().unwrap().is_empty());
    assert!(outside.exists());
}

#[test]
fn bound_home_arguments_and_exit_status_reach_the_actual_child() {
    let temp = TempDir::new().unwrap();
    let bin = temp.path().join("bin space %PATH% ! ' &");
    std::fs::create_dir_all(&bin).unwrap();
    let launcher = bin.join(if cfg!(windows) { "vx.exe" } else { "vx" });
    std::fs::copy(std::env::current_exe().unwrap(), &launcher).unwrap();
    let home = temp.path().join("home space %PATH% ! ' &");
    let probe = temp.path().join("probe.json");
    let entry = create_command_shim_in_home(
        "codex",
        "command_shim_child_probe",
        &launcher,
        &home,
        &[temp.path().join("commands")],
        &Platform::current(),
        None,
    )
    .unwrap();
    let output = Command::new(&entry.files[0])
        .args(["--exact", "--nocapture"])
        .env("VX_COMMAND_SHIM_TEST_PROBE", &probe)
        .env("VX_HOME", temp.path().join("wrong-home"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(23), "{output:?}");
    let captured: serde_json::Value =
        serde_json::from_slice(&std::fs::read(probe).unwrap()).unwrap();
    assert_eq!(captured["home"], home.to_string_lossy().as_ref());
    assert_eq!(
        captured["args"],
        serde_json::json!(["command_shim_child_probe", "--exact", "--nocapture"])
    );
}

/// The copied test binary acts as a launcher without requiring an installed runtime.
#[test]
fn command_shim_child_probe() {
    let Some(probe) = std::env::var_os("VX_COMMAND_SHIM_TEST_PROBE") else {
        return;
    };
    let captured = serde_json::json!({
        "home": std::env::var("VX_HOME").unwrap(),
        "args": std::env::args().skip(1).collect::<Vec<_>>(),
    });
    std::fs::write(probe, serde_json::to_vec(&captured).unwrap()).unwrap();
    std::process::exit(23);
}

#[test]
fn windows_shell_variant_uses_git_bash_path_and_bound_home() {
    let shim = vx_runtime::Shim::new("codex", r"C:\Users\tester\.vx\vx.exe")
        .with_args(&["codex"])
        .with_env("VX_HOME", r"C:\Users\tester\home with space");
    let shell = shim.content_for(ShimType::Shell);
    assert!(shell.contains("exec \"C:/Users/tester/.vx/vx.exe\" codex \"$@\""));
    assert!(shell.contains("VX_HOME='C:\\Users\\tester\\home with space'"));
    assert!(shell.contains("export VX_HOME"));
}

#[cfg(unix)]
#[test]
fn symbolic_links_are_neither_overwritten_nor_removed() {
    use std::os::unix::fs::symlink;
    let temp = TempDir::new().unwrap();
    let dirs = [temp.path().join("commands")];
    let entry = create_command_shim_in_home(
        "codex",
        "codex",
        &temp.path().join("vx"),
        temp.path(),
        &dirs,
        &Platform::current(),
        None,
    )
    .unwrap();
    let target = temp.path().join("target");
    std::fs::rename(&entry.files[0], &target).unwrap();
    symlink(&target, &entry.files[0]).unwrap();
    assert!(!entry.is_complete());
    assert!(entry.remove_files().unwrap().is_empty());
    assert!(
        create_command_shim_in_home(
            "codex",
            "codex",
            &temp.path().join("vx"),
            temp.path(),
            &dirs,
            &Platform::current(),
            Some(&entry),
        )
        .is_err()
    );
    assert!(entry.files[0].is_symlink());
    assert!(target.exists());
}

#[test]
fn relative_launchers_and_homes_are_rejected() {
    let temp = TempDir::new().unwrap();
    let dirs = [temp.path().join("commands")];
    assert!(
        create_command_shim_in_home(
            "codex",
            "codex",
            Path::new("vx"),
            temp.path(),
            &dirs,
            &Platform::current(),
            None,
        )
        .is_err()
    );
    assert!(
        create_command_shim_in_home(
            "codex",
            "codex",
            &temp.path().join("vx"),
            Path::new("home"),
            &dirs,
            &Platform::current(),
            None,
        )
        .is_err()
    );
}

#[test]
fn corrupt_empty_or_duplicate_registry_is_not_treated_as_empty() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("command-shims.json");
    std::fs::write(&path, "  \n").unwrap();
    assert!(ShimRegistry::load(&path).is_err());
    let entry = create_command_shim(
        "codex",
        "codex",
        &temp.path().join("vx"),
        &[temp.path().join("commands")],
        &Platform::current(),
    )
    .unwrap();
    let duplicate = ShimRegistry {
        shims: vec![entry.clone(), entry],
    };
    duplicate.save(&path).unwrap();
    assert!(ShimRegistry::load(&path).is_err());
}

#[cfg(windows)]
#[test]
fn windows_registry_uses_case_insensitive_command_identity() {
    let temp = TempDir::new().unwrap();
    let dirs = [temp.path().join("commands")];
    let launcher = temp.path().join("vx.exe");
    let entry = create_command_shim_in_home(
        "codex",
        "codex",
        &launcher,
        temp.path(),
        &dirs,
        &Platform::current(),
        None,
    )
    .unwrap();
    let mut registry = ShimRegistry::default();
    registry.upsert(entry.clone());
    assert!(registry.contains("CODEX"));
    let updated = create_command_shim_in_home(
        "CODEX",
        "codex",
        &launcher,
        temp.path(),
        &dirs,
        &Platform::current(),
        registry.get("CODEX"),
    )
    .unwrap();
    assert_eq!(updated.name, "codex");
    assert_eq!(updated.files, entry.files);
    registry.upsert(updated);
    assert_eq!(registry.len(), 1);
    assert!(registry.remove("CODEX").is_some());
    assert!(registry.is_empty());

    let path = temp.path().join("command-shims.json");
    let mut differently_cased = entry.clone();
    differently_cased.name = "CODEX".into();
    ShimRegistry {
        shims: vec![entry, differently_cased],
    }
    .save(&path)
    .unwrap();
    assert!(ShimRegistry::load(&path).is_err());
}

#[cfg(unix)]
#[test]
fn replaced_output_directory_cannot_redirect_update_or_removal() {
    use std::os::unix::fs::symlink;
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("commands");
    let saved = temp.path().join("saved-commands");
    let entry = create_command_shim_in_home(
        "codex",
        "codex",
        &temp.path().join("vx"),
        temp.path(),
        std::slice::from_ref(&output),
        &Platform::current(),
        None,
    )
    .unwrap();
    std::fs::rename(&output, &saved).unwrap();
    symlink(&saved, &output).unwrap();
    assert!(!entry.is_complete());
    assert!(entry.remove_files().is_err());
    assert!(
        create_command_shim_in_home(
            "codex",
            "codex",
            &temp.path().join("vx"),
            temp.path(),
            std::slice::from_ref(&output),
            &Platform::current(),
            Some(&entry),
        )
        .is_err()
    );
    assert!(saved.join("codex").exists());
    assert!(output.is_symlink());
}
