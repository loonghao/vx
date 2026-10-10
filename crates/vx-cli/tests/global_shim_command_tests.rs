//! Offline global package lifecycle coverage through the command shim registry.

mod common;
#[path = "common/package_shims.rs"]
mod package_shims;

use common::assert_success;
use package_shims::Fixture;
use rstest::rstest;
use std::fs;
use vx_paths::global_packages::PackageRegistry;
use vx_paths::shims;

#[test]
fn global_shim_update_registers_dispatch_wrappers_without_sweeping_unknown_files() {
    let fixture = Fixture::new();
    let target = fixture.cwd.join("legacy-target");
    fs::write(&target, "unknown legacy target").unwrap();
    let mut legacy_files = Vec::new();
    for directory in [&fixture.paths.shims_dir, fixture.binary_dir()] {
        let legacy = shims::create_shim(directory, "stale", &target).unwrap();
        legacy_files.push((
            legacy.shim_path.clone(),
            fs::read(legacy.shim_path).unwrap(),
        ));
    }
    let packages_before = fs::read(fixture.paths.packages_registry_file()).unwrap();

    assert_success(
        &fixture.run(&["global", "shim-update"]),
        "register dispatch wrappers for installed packages",
    );

    let registry = fixture.registry();
    for executable in ["codex", "claude"] {
        let entry = registry
            .get(executable)
            .expect("register installed executable");
        assert!(entry.is_complete());
        assert!(entry.runtime.starts_with("npm:"));
        assert!(entry.runtime.ends_with(&format!("::{executable}")));
    }
    for (file, contents) in legacy_files {
        assert_eq!(
            fs::read(file).unwrap(),
            contents,
            "unknown wrappers must not be swept"
        );
    }
    assert_eq!(
        fs::read(fixture.paths.packages_registry_file()).unwrap(),
        packages_before
    );
    fixture.assert_preserved();
}

#[test]
fn explicit_pkg_install_publishes_an_already_installed_package_without_reinstalling() {
    let fixture = Fixture::new();
    let packages_before = fs::read(fixture.paths.packages_registry_file()).unwrap();
    assert!(fixture.registry().is_empty());

    assert_success(
        &fixture.run(&["pkg", "install", "npm:@openai/codex"]),
        "publish the already installed package without a network install",
    );

    let registry = fixture.registry();
    let entry = registry
        .get("codex")
        .expect("publish codex on explicit package install");
    assert!(entry.is_complete());
    assert_eq!(entry.runtime, "npm:@openai/codex::codex");
    assert_eq!(
        fs::read(fixture.paths.packages_registry_file()).unwrap(),
        packages_before
    );
    fixture.assert_preserved();
}

#[test]
fn package_uninstall_removes_registered_aliases_and_preserves_unknown_legacy_wrappers() {
    let fixture = Fixture::new();
    assert_success(&fixture.run(&["shim", "add", "codex"]), "publish codex");
    let custom_dir = fixture.cwd.join("custom bin");
    assert_success(
        &fixture.run(&[
            "shim",
            "add",
            "codex",
            "--as",
            "personal-codex",
            "--dir",
            custom_dir.to_str().unwrap(),
        ]),
        "publish a custom package command",
    );
    assert_success(
        &fixture.run(&["shim", "add", "claude-code"]),
        "publish another package",
    );
    let registry_before = fixture.registry();
    let codex_files: Vec<_> = ["codex", "personal-codex"]
        .into_iter()
        .flat_map(|name| registry_before.get(name).unwrap().files.clone())
        .collect();

    // This unregistered legacy wrapper has the same command name. Its file
    // shape alone must never authorize deletion by the new registry lifecycle.
    let legacy_target = fixture.cwd.join("external-codex");
    fs::write(&legacy_target, "unregistered executable").unwrap();
    let legacy = shims::create_shim(&fixture.paths.shims_dir, "codex", &legacy_target).unwrap();
    let legacy_contents = fs::read(&legacy.shim_path).unwrap();
    let package_dir = fixture
        .paths
        .global_package_dir("npm", "@openai/codex", "1.2.3");

    assert_success(
        &fixture.run(&["pkg", "uninstall", "npm:@openai/codex", "--force"]),
        "uninstall package and its registered command aliases",
    );

    let registry_after = fixture.registry();
    assert!(!registry_after.contains("codex"));
    assert!(!registry_after.contains("personal-codex"));
    assert!(registry_after.get("claude").unwrap().is_complete());
    assert!(codex_files.iter().all(|path| !path.exists()));
    assert!(!package_dir.exists());
    assert_eq!(fs::read(&legacy.shim_path).unwrap(), legacy_contents);
    assert!(legacy_target.is_file());
    let packages = PackageRegistry::load(&fixture.paths.packages_registry_file()).unwrap();
    assert!(!packages.contains("npm", "@openai/codex"));
    assert!(packages.contains("npm", "@anthropic-ai/claude-code"));
    fixture.assert_preserved();
}

#[rstest]
#[case(&["pkg", "install", "npm:@openai/codex"])]
#[case(&["global", "shim-update"])]
fn global_publication_cannot_replace_a_different_executable_from_the_same_package(
    #[case] args: &[&str],
) {
    let fixture = Fixture::new();
    assert_success(
        &fixture.run(&[
            "shim",
            "add",
            "npm:@openai/codex::alternate",
            "--as",
            "codex",
        ]),
        "publish an explicitly selected package executable",
    );
    let original = fixture.registry().get("codex").unwrap().clone();
    let contents: Vec<_> = original
        .files
        .iter()
        .map(|file| (file.clone(), fs::read(file).unwrap()))
        .collect();

    let output = fixture.run(args);

    assert!(
        !output.status.success(),
        "{}",
        common::combined_output(&output)
    );
    assert_eq!(fixture.registry().get("codex"), Some(&original));
    for (file, expected) in contents {
        assert_eq!(fs::read(file).unwrap(), expected);
    }
    fixture.assert_preserved();
}

#[test]
fn explicit_pkg_install_preserves_a_registered_package_version_pin() {
    let fixture = Fixture::new();
    assert_success(
        &fixture.run(&["shim", "add", "codex@1.2.3"]),
        "publish a pinned package command",
    );
    let pinned = fixture.registry().get("codex").unwrap().clone();
    assert!(pinned.runtime.contains("@1.2.3"));

    assert_success(
        &fixture.run(&["pkg", "install", "npm:@openai/codex"]),
        "publish an installed package without discarding its command version pin",
    );

    let current = fixture.registry();
    let entry = current.get("codex").unwrap();
    assert_eq!(entry.runtime, pinned.runtime);
    assert!(entry.is_complete());
    fixture.assert_preserved();
}
