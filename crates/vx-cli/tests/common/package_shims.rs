//! Isolated offline fixtures shared by package command shim acceptance tests.

#![allow(dead_code)]

use super::common::{combined_output, run_command_with_timeout};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Duration;
use tempfile::TempDir;
use vx_paths::VxPaths;
use vx_paths::global_packages::{GlobalPackage, PackageRegistry};
use vx_runtime::ShimRegistry;

pub(crate) const TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) struct Fixture {
    _temp: TempDir,
    pub(crate) paths: VxPaths,
    pub(crate) cwd: PathBuf,
    binary: PathBuf,
    binary_hash: Vec<u8>,
    runtime_bin: PathBuf,
}

impl Fixture {
    pub(crate) fn new() -> Self {
        let temp = tempfile::Builder::new()
            .prefix("vx package shim ")
            .tempdir()
            .expect("create isolated fixture");
        #[cfg(unix)]
        let root = fs::canonicalize(temp.path()).unwrap();
        #[cfg(windows)]
        let root = temp.path().to_path_buf();
        let paths = VxPaths::with_base_dir(root.join("package home"));
        paths.ensure_dirs().expect("initialize isolated VX_HOME");
        let cwd = root.join("working directory");
        fs::create_dir(&cwd).unwrap();
        let binary_dir = root.join("vx bin");
        fs::create_dir(&binary_dir).unwrap();
        let binary = binary_dir.join(binary_name());
        let candidate = candidate_binary();
        if fs::hard_link(&candidate, &binary).is_err() {
            fs::copy(&candidate, &binary).expect("copy candidate vx into isolated fixture");
        }
        let binary_hash = hash_file(&binary);
        fs::write(binary_dir.join("unrelated.txt"), "keep this file").unwrap();
        let runtime_bin = paths
            .base_dir
            .join("store")
            .join("node")
            .join("22.0.0")
            .join("bin");
        fs::create_dir_all(&runtime_bin).unwrap();
        fs::write(
            runtime_bin.join(if cfg!(windows) { "node.exe" } else { "node" }),
            "runtime path fixture; never executed",
        )
        .unwrap();
        let fixture = Self {
            _temp: temp,
            paths,
            cwd,
            binary,
            binary_hash,
            runtime_bin,
        };
        let mut packages = PackageRegistry::new();
        for (package, executable) in [
            ("@openai/codex", "codex"),
            ("@anthropic-ai/claude-code", "claude"),
        ] {
            let directory = fixture.paths.global_package_dir("npm", package, "1.2.3");
            fs::create_dir_all(&directory).unwrap();
            write_package_executable(&directory, executable);
            packages.register(
                GlobalPackage::new(package, "1.2.3", "npm", directory).with_executable(executable),
            );
        }
        packages
            .save(&fixture.paths.packages_registry_file())
            .unwrap();
        fixture
    }

    pub(crate) fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(&self.binary);
        command
            .args(args)
            .current_dir(&self.cwd)
            .env("VX_HOME", &self.paths.base_dir)
            .env("VX_TEST_MODE", "1")
            // No ambient codex, claude, or package manager may affect this fixture.
            .env("PATH", self.binary_dir());
        command
    }

    pub(crate) fn run(&self, args: &[&str]) -> Output {
        run_command_with_timeout(self.command(args), TIMEOUT)
            .expect("run candidate vx within timeout")
    }

    pub(crate) fn registry(&self) -> ShimRegistry {
        ShimRegistry::load(&ShimRegistry::default_path(&self.paths.config_dir)).unwrap()
    }

    pub(crate) fn binary_dir(&self) -> &Path {
        self.binary.parent().unwrap()
    }

    pub(crate) fn assert_preserved(&self) {
        assert_eq!(hash_file(&self.binary), self.binary_hash);
        assert_eq!(
            fs::read_to_string(self.binary_dir().join("unrelated.txt")).unwrap(),
            "keep this file"
        );
    }

    pub(crate) fn assert_package_output(&self, output: &Output) {
        self.assert_invocation_output(output);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains(&self.runtime_bin.display().to_string()),
            "{stdout}"
        );
    }

    pub(crate) fn assert_invocation_output(&self, output: &Output) {
        assert_eq!(
            output.status.code(),
            Some(23),
            "{}",
            combined_output(output)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        for expected in [
            "ARG1=[two words]".to_string(),
            "ARG2=[--flag]".to_string(),
            format!("CWD=[{}]", self.cwd.display()),
            format!("HOME=[{}]", self.paths.base_dir.display()),
        ] {
            assert!(stdout.contains(&expected), "missing {expected:?}: {stdout}");
        }
    }

    #[cfg(unix)]
    pub(crate) fn first_uv_install_command(&self) -> Command {
        use std::os::unix::fs::PermissionsExt;

        let source = self
            .paths
            .global_package_dir("npm", "@openai/codex", "1.2.3")
            .join("codex");
        let uv = self.binary_dir().join("uv");
        fs::write(
            &uv,
            "#!/bin/sh\nset -eu\n[ \"$1\" = tool ] && [ \"$2\" = install ] && [ \"$3\" = --tool-dir ] && [ \"$4\" = \"$UV_TOOL_DIR\" ] && [ \"$5\" = codex ] || exit 64\n/bin/mkdir -p \"$UV_TOOL_DIR/bin\"\n/bin/cp \"$VX_TEST_PACKAGE_SOURCE\" \"$UV_TOOL_DIR/bin/codex\"\nprintf installed > \"$VX_TEST_INSTALL_RECEIPT\"\n",
        ).unwrap();
        fs::set_permissions(uv, fs::Permissions::from_mode(0o755)).unwrap();

        // The self-contained uv ecosystem has no runtime auto-install branch.
        // This script exercises auto_install_package itself without npm's
        // optional Bun runtime discovery or any network-capable installer.
        let mut command = self.command(&["uv:codex", "two words", "--flag"]);
        command.env("VX_TEST_PACKAGE_SOURCE", source).env(
            "VX_TEST_INSTALL_RECEIPT",
            self.cwd.join("uv-install-receipt"),
        );
        command
    }
}

fn binary_name() -> &'static str {
    if cfg!(windows) { "vx.exe" } else { "vx" }
}

fn candidate_binary() -> PathBuf {
    // vx-cli has no binary target. CI downloads the exact root-package binary
    // into workspace/bin; local builds place it beside the test's deps folder.
    // Missing candidates must fail, never silently run an installed vx or skip.
    let candidate =
        std::env::var_os("VX_BINARY")
            .map(PathBuf::from)
            .or_else(|| option_env!("CARGO_BIN_EXE_vx").map(PathBuf::from))
            .unwrap_or_else(|| {
                let test_executable = std::env::current_exe().unwrap();
                let local = test_executable
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join(binary_name());
                let artifact = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../bin")
                    .join(binary_name());
                let candidates = if std::env::var_os("CI").is_some() {
                    [artifact, local]
                } else {
                    [local, artifact]
                };
                candidates.iter().find(|path| path.is_file()).cloned().unwrap_or_else(|| {
                panic!("candidate vx binary missing from {candidates:?}; build vx or set VX_BINARY")
            })
            });
    assert!(
        candidate.is_file(),
        "candidate vx is missing: {}",
        candidate.display()
    );
    candidate
}

fn hash_file(path: &Path) -> Vec<u8> {
    let mut file = fs::File::open(path).unwrap();
    let mut hash = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        let count = file.read(&mut buffer).unwrap();
        if count == 0 {
            return hash.finalize().to_vec();
        }
        hash.update(&buffer[..count]);
    }
}

fn write_package_executable(directory: &Path, name: &str) {
    #[cfg(windows)]
    fs::write(
        directory.join(format!("{name}.cmd")),
        "@echo off\r\necho ARG1=[%~1]\r\necho ARG2=[%~2]\r\necho CWD=[%CD%]\r\necho HOME=[%VX_HOME%]\r\necho RUNTIME_PATH=[%PATH%]\r\nexit /b 23\r\n",
    ).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let executable = directory.join(name);
        fs::write(
            &executable,
            "#!/bin/sh\nprintf 'ARG1=[%s]\\nARG2=[%s]\\nCWD=[%s]\\nHOME=[%s]\\nRUNTIME_PATH=[%s]\\n' \"$1\" \"$2\" \"$PWD\" \"$VX_HOME\" \"$PATH\"\nexit 23\n",
        ).unwrap();
        fs::set_permissions(executable, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

pub(crate) fn shim_files(directory: &Path, name: &str) -> Vec<PathBuf> {
    let mut paths = vec![directory.join(name)];
    if cfg!(windows) {
        paths.push(directory.join(format!("{name}.cmd")));
    }
    paths
}
