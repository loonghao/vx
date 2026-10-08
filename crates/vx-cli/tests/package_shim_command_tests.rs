//! Offline acceptance coverage for explicitly exposing isolated package commands.

mod common;
#[path = "common/package_shims.rs"]
mod package_shims;

use common::{assert_success, combined_output, run_command_with_timeout};
use package_shims::{Fixture, TIMEOUT, shim_files};
use rstest::rstest;
use std::fs;
#[cfg(windows)]
use std::path::PathBuf;
use std::process::Command;
use vx_paths::global_packages::{GlobalPackage, PackageRegistry};
use vx_runtime::CommandShim;
use vx_runtime::ShimRegistry;

#[test]
fn package_command_add_list_sync_execute_and_remove_are_one_lifecycle() {
    let fixture = Fixture::new();
    let packages_before = fs::read(fixture.paths.packages_registry_file()).unwrap();

    fixture.assert_package_output(&fixture.run(&["codex", "two words", "--flag"]));
    assert!(fixture.registry().is_empty());
    for directory in [
        &fixture.paths.bin_dir,
        &fixture.paths.shims_dir,
        fixture.binary_dir(),
    ] {
        assert!(
            shim_files(directory, "codex")
                .iter()
                .all(|path| !path.exists())
        );
    }

    assert_success(&fixture.run(&["shim", "add", "codex"]), "publish codex");
    let entry = fixture.registry().get("codex").unwrap().clone();
    assert!(entry.is_complete());
    for directory in [&fixture.paths.bin_dir, fixture.binary_dir()] {
        assert!(
            shim_files(directory, "codex")
                .iter()
                .all(|path| path.is_file())
        );
    }
    let list = fixture.run(&["shim", "list", "--json"]);
    assert_success(&list, "list command shims as JSON");
    let listed: Vec<CommandShim> = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(listed, vec![entry.clone()]);
    assert_success(&fixture.run(&["shim", "path"]), "show shim directories");

    fs::remove_file(&entry.files[0]).unwrap();
    assert_success(
        &fixture.run(&["shim", "sync"]),
        "restore a missing shim variant",
    );
    assert!(fixture.registry().get("codex").unwrap().is_complete());
    for directory in [&fixture.paths.bin_dir, fixture.binary_dir()] {
        let mut command = native_shell_command();
        command
            .current_dir(&fixture.cwd)
            .env("PATH", directory)
            .env_remove("VX_HOME")
            .env("VX_TEST_MODE", "1");
        let output = run_command_with_timeout(command, TIMEOUT).unwrap();
        fixture.assert_package_output(&output);
    }

    assert_success(
        &fixture.run(&["shim", "remove", "codex"]),
        "remove codex shim",
    );
    assert!(fixture.registry().is_empty());
    assert!(entry.files.iter().all(|path| !path.exists()));
    assert_eq!(
        fs::read(fixture.paths.packages_registry_file()).unwrap(),
        packages_before
    );
    fixture.assert_preserved();
}

fn native_shell_command() -> Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;

        let mut command = Command::new(std::env::var_os("COMSPEC").expect("cmd.exe is available"));
        command.raw_arg("/d /s /c \"codex \"two words\" --flag\"");
        command
    }
    #[cfg(unix)]
    {
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "codex 'two words' --flag"]);
        command
    }
}

#[cfg(windows)]
#[test]
fn git_bash_executes_the_extensionless_package_shim_without_inherited_home() {
    let fixture = Fixture::new();
    let bash = git_bash_binary();
    assert_success(
        &fixture.run(&["shim", "add", "codex"]),
        "publish codex for Git Bash",
    );
    let mut command = Command::new(bash);
    command
        .args([
            "--noprofile",
            "--norc",
            "-c",
            "PATH=\"$(/usr/bin/cygpath -u \"$VX_SHIM_TEST_BIN\"):/usr/bin:/bin\"; export PATH; codex 'two words' --flag",
        ])
        .current_dir(&fixture.cwd)
        .env("VX_SHIM_TEST_BIN", fixture.binary_dir())
        .env("MSYS2_ARG_CONV_EXCL", "*")
        .env_remove("VX_HOME")
        .env("VX_TEST_MODE", "1");
    let output = run_command_with_timeout(command, TIMEOUT).unwrap();
    fixture.assert_package_output(&output);
    fixture.assert_preserved();
}

#[cfg(windows)]
fn git_bash_binary() -> PathBuf {
    if let Some(path) = std::env::var_os("VX_TEST_GIT_BASH") {
        let path = PathBuf::from(path);
        assert!(path.is_file(), "VX_TEST_GIT_BASH must identify Git Bash");
        return path;
    }
    let mut candidates = Vec::new();
    if let Ok(git) = which::which("git") {
        for ancestor in git.ancestors().skip(1).take(3) {
            candidates.push(ancestor.join("bin/bash.exe"));
            candidates.push(ancestor.join("usr/bin/bash.exe"));
        }
    }
    for variable in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
        if let Some(directory) = std::env::var_os(variable) {
            let directory = PathBuf::from(directory);
            candidates.push(directory.join("Git/bin/bash.exe"));
            candidates.push(directory.join("Programs/Git/bin/bash.exe"));
        }
    }
    candidates.into_iter().find(|path| path.is_file()).expect(
        "Git Bash is required for Windows shim acceptance; set VX_TEST_GIT_BASH to its bash.exe",
    )
}

#[rstest]
#[case("claude-code", "claude")]
#[case("npm:@openai/codex", "codex")]
fn package_alias_and_scoped_package_publish_the_actual_executable(
    #[case] selector: &str,
    #[case] expected: &str,
) {
    let fixture = Fixture::new();
    let output = fixture.run(&["shim", "add", selector]);
    assert_success(&output, "publish package executable");
    let registry = fixture.registry();
    let entry = registry
        .get(expected)
        .expect("register the package executable name");
    assert!(entry.is_complete());
    assert_eq!(registry.len(), 1);
    fixture.assert_preserved();
}

#[rstest]
#[case(
    "custom-shim-cli@2.0",
    "custom-shim-cli",
    "uvx:test-shim-package@2.0::custom-shim-cli"
)]
#[case(
    "custom-shim-cli@2.0::alternate",
    "alternate",
    "uvx:test-shim-package@2.0::alternate"
)]
#[case("custom-shim-cli@2.0::bash", "bash", "uvx:test-shim-package@2.0::bash")]
fn registered_package_executables_preserve_requested_versions_and_overrides(
    #[case] selector: &str,
    #[case] expected_name: &str,
    #[case] expected_runtime: &str,
) {
    let fixture = Fixture::new();
    let registry_path = fixture.paths.packages_registry_file();
    let mut packages = PackageRegistry::load(&registry_path).unwrap();
    packages.register(
        GlobalPackage::new(
            "test-shim-package",
            "1.2.3",
            "uvx",
            fixture
                .paths
                .global_package_dir("uvx", "test-shim-package", "1.2.3"),
        )
        .with_executable("custom-shim-cli"),
    );
    packages.save(&registry_path).unwrap();
    let packages_before = fs::read(&registry_path).unwrap();

    assert_success(
        &fixture.run(&["shim", "add", selector]),
        "publish the requested target of a registered package executable",
    );

    let registry = fixture.registry();
    assert_eq!(registry.len(), 1);
    let entry = registry
        .get(expected_name)
        .expect("register the requested command name");
    assert_eq!(entry.runtime, expected_runtime);
    assert!(entry.is_complete());
    assert_eq!(fs::read(&registry_path).unwrap(), packages_before);
    fixture.assert_preserved();
}

#[test]
fn package_shim_add_preserves_destination_collisions_outside_path() {
    let fixture = Fixture::new();
    let destination = fixture.cwd.join("other bin");
    fs::create_dir(&destination).unwrap();
    let foreign = destination.join("codex");
    let contents = "#!/bin/sh\n# this user script mentions vx-shim\necho personal\n";
    fs::write(&foreign, contents).unwrap();

    let output = fixture.run(&[
        "shim",
        "add",
        "codex",
        "--dir",
        destination.to_str().unwrap(),
    ]);

    assert!(!output.status.success(), "{}", combined_output(&output));
    assert_eq!(fs::read_to_string(&foreign).unwrap(), contents);
    assert!(fixture.registry().is_empty());
    if cfg!(windows) {
        assert!(
            !destination.join("codex.cmd").exists(),
            "preflight all variants before writing"
        );
    }
    fixture.assert_preserved();
}

#[test]
fn sync_and_remove_preserve_a_user_edited_package_shim() {
    let fixture = Fixture::new();
    assert_success(&fixture.run(&["shim", "add", "codex"]), "publish codex");
    let entry = fixture.registry().get("codex").unwrap().clone();
    let edited = &entry.files[0];
    let mut contents = fs::read_to_string(edited).unwrap();
    // Keep the ownership marker: it alone does not prove the file is unchanged.
    contents.push_str("\n# manually edited by the user\n");
    fs::write(edited, &contents).unwrap();

    let _ = fixture.run(&["shim", "sync"]);
    assert_eq!(fs::read_to_string(edited).unwrap(), contents);
    let _ = fixture.run(&["shim", "remove", "codex"]);
    assert_eq!(fs::read_to_string(edited).unwrap(), contents);
    fixture.assert_preserved();
}

// UvInstaller resolves bare `uv`; a shell-script fixture is executable on
// Unix. Windows npm package dispatch is covered separately through cmd/Bash.
#[cfg(unix)]
#[test]
fn implicit_first_install_with_a_fake_uv_installer_does_not_publish_commands() {
    let fixture = Fixture::new();
    let command = fixture.first_uv_install_command();
    assert!(
        !PackageRegistry::load(&fixture.paths.packages_registry_file())
            .unwrap()
            .contains("uv", "codex")
    );

    let output = run_command_with_timeout(command, TIMEOUT).unwrap();

    fixture.assert_invocation_output(&output);
    let packages = PackageRegistry::load(&fixture.paths.packages_registry_file()).unwrap();
    let package = packages.get("uv", "codex").unwrap();
    assert_eq!(package.version, "latest");
    assert_eq!(package.executables, vec!["codex"]);
    assert_eq!(
        fs::read_to_string(fixture.cwd.join("uv-install-receipt")).unwrap(),
        "installed"
    );
    assert!(fixture.registry().is_empty());
    for directory in [
        &fixture.paths.bin_dir,
        &fixture.paths.shims_dir,
        fixture.binary_dir(),
    ] {
        assert!(
            shim_files(directory, "codex")
                .iter()
                .all(|path| !path.exists())
        );
    }
    fixture.assert_preserved();
}

#[test]
fn relative_home_is_bound_before_the_command_runs_in_another_directory() {
    let fixture = Fixture::new();
    let mut add = fixture.command(&["shim", "add", "codex"]);
    add.env("VX_HOME", "../package home");
    assert_success(
        &run_command_with_timeout(add, TIMEOUT).unwrap(),
        "publish with relative VX_HOME",
    );

    let mut command = native_shell_command();
    command
        .current_dir(fixture.binary_dir())
        .env("PATH", fixture.binary_dir())
        .env_remove("VX_HOME")
        .env("VX_TEST_MODE", "1");
    let output = run_command_with_timeout(command, TIMEOUT).unwrap();
    assert_eq!(
        output.status.code(),
        Some(23),
        "{}",
        combined_output(&output)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&format!("HOME=[{}]", fixture.paths.base_dir.display())),
        "{stdout}"
    );
    assert!(
        stdout.contains(&format!("CWD=[{}]", fixture.binary_dir().display())),
        "{stdout}"
    );
    fixture.assert_preserved();
}

#[test]
fn another_home_and_a_corrupt_registry_cannot_replace_existing_package_commands() {
    let fixture = Fixture::new();
    assert_success(
        &fixture.run(&["shim", "add", "codex"]),
        "publish codex in its owner home",
    );
    let entry = fixture.registry().get("codex").unwrap().clone();
    let files: Vec<_> = entry
        .files
        .iter()
        .map(|file| (file.clone(), fs::read(file).unwrap()))
        .collect();
    let other_home = fixture.cwd.join("other home");
    let mut other_add = fixture.command(&["shim", "add", "codex"]);
    other_add.env("VX_HOME", &other_home);
    let output = run_command_with_timeout(other_add, TIMEOUT).unwrap();
    assert!(!output.status.success(), "{}", combined_output(&output));
    assert!(
        ShimRegistry::load(&other_home.join("config/command-shims.json"))
            .unwrap()
            .is_empty()
    );

    let registry_path = ShimRegistry::default_path(&fixture.paths.config_dir);
    let corrupt = "{ invalid registry, do not replace";
    fs::write(&registry_path, corrupt).unwrap();
    let output = fixture.run(&["shim", "add", "claude-code"]);
    assert!(!output.status.success(), "{}", combined_output(&output));
    assert_eq!(fs::read_to_string(registry_path).unwrap(), corrupt);
    for (file, contents) in files {
        assert_eq!(fs::read(file).unwrap(), contents);
    }
    fixture.assert_preserved();
}
