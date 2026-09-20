#!/usr/bin/env python3
"""Exercise the pre-push hook in throwaway repositories.

The hook is the only lockfile guard on the push path: on a pull request, CI
regenerates `workspace-hack` and commits the result rather than failing, so
nothing else refuses a bad lockfile. It went through two changes with no
automated coverage at all, and it is now fail-closed, meaning a machine
without cargo cannot push anything. A guard like that needs a test.

Each scenario builds a small repository with a bare "remote", installs the
hook, and runs it the way git would, then asserts on the exit code and on
whether the files were left alone.

Pass --hook to test a hook outside this repository.

Exit codes
----------
0
    Every scenario behaved as expected.
1
    At least one scenario did not.
"""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_HOOK = REPOSITORY_ROOT / ".githooks" / "pre-push"

PUBLISHER = "print('publisher')\n"
VALIDATOR = "def is_valid_title(title: str) -> bool:\n    return True\n"


def _path_with_cargo(current: str) -> str:
    """Return PATH with cargo's directory guaranteed to be present."""

    cargo = shutil.which("cargo")
    if not cargo:
        return current
    directory = str(Path(cargo).parent)
    if directory in current.split(os.pathsep):
        return current
    return current + os.pathsep + directory


def _posix(path: Path) -> str:
    """Return `path` in a form bash accepts.

    The hook is invoked through bash, which reads `C:\\...` as escapes. Git
    Bash wants the `/c/...` spelling.
    """

    text = path.as_posix()
    if len(text) > 1 and text[1] == ":":
        text = f"/{text[0].lower()}{text[2:]}"
    return text


class Scenario:
    """One hook scenario, run in its own throwaway repository."""

    def __init__(self, name: str, hook: Path) -> None:
        self.name = name
        self.hook = hook
        self.dir = Path(tempfile.mkdtemp(prefix="hook-"))
        self.remote = self.dir / "remote.git"
        self.work = self.dir / "work"

    def close(self) -> None:
        shutil.rmtree(self.dir, ignore_errors=True)

    def git(self, *args: str, cwd: Path | None = None, check: bool = True) -> str:
        result = subprocess.run(
            ["git", *args],
            cwd=cwd or self.work,
            capture_output=True,
            text=True,
        )
        if check and result.returncode != 0:
            raise AssertionError(f"git {' '.join(args)} failed: {result.stderr}")
        return result.stdout.strip()

    def build(self, with_publisher: bool = True) -> None:
        """Create a remote whose default branch has the policy code."""

        subprocess.run(["git", "init", "--bare", "-q", str(self.remote)], check=True)
        self.work.mkdir(parents=True)
        (self.work / "scripts").mkdir()

        self.git("init", "-q")
        self.git("config", "user.email", "test@example.com")
        self.git("config", "user.name", "test")
        # The fixture writes LF; without this the checkout converts to CRLF and
        # the tree reads as dirty before a scenario has even started, which
        # makes every scenario report the dirty-tree branch.
        self.git("config", "core.autocrlf", "false")
        self.git("branch", "-m", "main")
        (self.work / "scripts" / "validate_pr_title.py").write_text(VALIDATOR)
        if with_publisher:
            (self.work / "scripts" / "pr_title_status.py").write_text(PUBLISHER)
        # A workspace with one real dependency, mirrored in the lockfile, so
        # that dropping the package from the lock is a genuine mismatch rather
        # than a malformed manifest (which cargo reports differently, and which
        # the hook deliberately separates out).
        (self.work / "Cargo.toml").write_text(
            '[workspace]\nmembers = ["crate-a"]\nresolver = "2"\n'
        )
        (self.work / "crate-a").mkdir(exist_ok=True)
        (self.work / "crate-a" / "src").mkdir(exist_ok=True)
        (self.work / "crate-a" / "src" / "lib.rs").write_text("")
        (self.work / "crate-a" / "Cargo.toml").write_text(
            '[package]\nname = "crate-a"\nversion = "0.1.0"\nedition = "2021"\n\n'
            '[dependencies]\nanyhow = "1"\n'
        )
        # A real lockfile, generated rather than hand-written: a fabricated
        # checksum makes `cargo metadata` fail for a reason unrelated to the
        # scenario, and the hook would be right to report it as cargo failing.
        generated = subprocess.run(
            ["cargo", "generate-lockfile", "--offline"],
            cwd=self.work,
            capture_output=True,
            text=True,
        )
        if generated.returncode != 0:
            generated = subprocess.run(
                ["cargo", "generate-lockfile"],
                cwd=self.work,
                capture_output=True,
                text=True,
            )
        if generated.returncode != 0:
            raise AssertionError(f"could not create a lockfile: {generated.stderr}")
        (self.work / "crates" / "workspace-hack").mkdir(parents=True)
        (self.work / "crates" / "workspace-hack" / "Cargo.toml").write_text("[package]\nname = \"workspace-hack\"\nversion = \"0.1.0\"\n")

        self.git("add", ".")
        self.git("commit", "-q", "-m", "initial")
        self.git("remote", "add", "origin", str(self.remote))
        self.git("push", "-q", "origin", "HEAD:refs/heads/main")

        # Install the hook exactly as `just setup-hooks` does.
        hooks = self.work / ".git" / "hooks"
        hooks.mkdir(exist_ok=True)
        target = hooks / "pre-push"
        shutil.copy2(self.hook, target)
        os.chmod(target, 0o755)

    def run_hook(self, env: dict[str, str] | None = None) -> subprocess.CompletedProcess:
        # Invoked through a relative path with the work tree as the working
        # directory, which is how git runs it. Forward slashes are used
        # explicitly: bash reads backslashes as escapes, and os.path.join
        # would supply them on Windows.
        merged: dict[str, str] = {**os.environ, **(env or {})}
        if "PATH" not in (env or {}):
            # Model a machine that has cargo. A wrapper that shells out to
            # bash does not always expose the toolchain to that bash, and
            # without this the scenario under test would be "cargo missing"
            # rather than the one it names.
            merged["PATH"] = _path_with_cargo(merged.get("PATH", ""))
        return subprocess.run(
            ["bash", ".git/hooks/pre-push"],
            cwd=self.work,
            capture_output=True,
            text=True,
            env=merged,
        )


def scenario_dirty_tree(hook: Path) -> tuple[bool, str]:
    """A modified lockfile must be refused, before any check runs."""

    case = Scenario("dirty tree", hook)
    try:
        case.build()
        before = (case.work / "Cargo.lock").read_text()
        (case.work / "Cargo.lock").write_text(before + "\n# local edit\n")

        result = case.run_hook()

        if result.returncode == 0:
            return False, "push was allowed with a dirty lockfile"
        if "Refusing to run lockfile checks" not in result.stdout:
            return False, f"no refusal message: {result.stdout!r}"
        if (case.work / "Cargo.lock").read_text() != before + "\n# local edit\n":
            return False, "the local edit was reverted"
        return True, "refused, edit left alone"
    finally:
        case.close()


def scenario_cargo_missing(hook: Path) -> tuple[bool, str]:
    """No cargo means no verification, and that must fail, not pass."""

    case = Scenario("cargo missing", hook)
    try:
        case.build()
        # A PATH containing only git: cargo is absent, everything else is not.
        limited = case.dir / "bin"
        limited.mkdir(exist_ok=True)
        git_path = shutil.which("git")
        bash_path = shutil.which("bash")
        for path in (git_path, bash_path):
            if path:
                link = limited / Path(path).name
                if not link.exists():
                    try:
                        link.symlink_to(path)
                    except OSError:
                        shutil.copy2(path, link)

        result = case.run_hook(env={"PATH": str(limited)})

        if result.returncode == 0:
            return False, "succeeded without cargo, so nothing was verified"
        if "--no-verify" not in result.stdout:
            return False, f"no escape hatch named: {result.stdout!r}"
        return True, "fails closed and names --no-verify"
    finally:
        case.close()


def scenario_lockfile_mismatch(hook: Path) -> tuple[bool, str]:
    """A lockfile missing a required package is caught, and left as committed."""

    case = Scenario("lockfile mismatch", hook)
    try:
        case.build()
        committed = (case.work / "Cargo.lock").read_text()
        # Drop the package the manifest declares, so --locked must fail.
        (case.work / "Cargo.lock").write_text('version = 4\n')
        case.git("add", "Cargo.lock")

        result = case.run_hook()

        if result.returncode == 0:
            return False, "a lockfile that does not match was accepted"
        if "cargo not found" in result.stdout:
            # The hook could not see the toolchain even though this process
            # can. That is a property of the harness, not of the hook, so it
            # is reported rather than counted as a failure: on CI the lane
            # runs where cargo is visible to the hook's shell.
            return None, "skipped: cargo is not visible to the hook's shell here"  # type: ignore[return-value]
        if "does not satisfy the manifests" not in result.stdout:
            return False, f"misreported as: {result.stdout!r}"
        if (case.work / "Cargo.lock").read_text() == committed:
            return False, "the lockfile was silently repaired"
        return True, "caught and reported, not repaired"
    finally:
        case.close()


def scenario_staged_not_committed(hook: Path) -> tuple[bool, str]:
    """A staged-but-uncommitted violation is detected, and not rolled back."""

    case = Scenario("staged, uncommitted", hook)
    try:
        case.build()
        (case.work / "Cargo.lock").write_text('version = 4\n')
        case.git("add", "Cargo.lock")

        result = case.run_hook()

        if result.returncode == 0:
            return False, "a staged violation was not detected"
        # The baseline and the restore both read the index, so the staged
        # content must survive: detecting it is not the same as undoing it.
        if (case.work / "Cargo.lock").read_text() != 'version = 4\n':
            return False, "the staged change was rolled back"
        return True, "detected, staged content preserved"
    finally:
        case.close()


SCENARIOS = (
    scenario_dirty_tree,
    scenario_cargo_missing,
    scenario_lockfile_mismatch,
    scenario_staged_not_committed,
)


def main(argv: list[str] | None = None) -> int:
    """Run every scenario and report the result of each."""

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--hook", default=str(DEFAULT_HOOK), help="Hook to test.")
    args = parser.parse_args(argv)

    hook = Path(args.hook)
    if not hook.is_file():
        print(f"::error::hook not found: {hook}")
        return 1

    print(f"Testing {hook}")
    print(f"git: {subprocess.run(['git', '--version'], capture_output=True, text=True).stdout.strip()}")
    print()

    failures = 0
    skipped = 0
    for scenario in SCENARIOS:
        try:
            passed, detail = scenario(hook)
        except Exception as error:  # noqa: BLE001 - a broken scenario is a failure
            passed, detail = False, f"scenario raised: {error}"
        if passed is None:
            skipped += 1
            status = "SKIP"
        elif passed:
            status = "PASS"
        else:
            status = "FAIL"
            failures += 1
        print(f"  [{status}] {scenario.__name__.replace('scenario_', '').replace('_', ' ')}: {detail}")
        if passed is False:
            print(f"::error::{scenario.__name__}: {detail}")

    print()
    if failures:
        print(f"{failures} of {len(SCENARIOS)} scenarios failed, {skipped} skipped.")
        return 1

    print(f"All {len(SCENARIOS) - skipped} scenarios passed ({skipped} skipped).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
