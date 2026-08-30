"""Behavior tests for the Release Please pull request validator."""

from pathlib import Path
import subprocess
import sys
import unittest


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
VALIDATOR = REPOSITORY_ROOT / "scripts" / "validate_release_pr.py"
REQUIRED_FILES = (
    ".release-please-manifest.json",
    "CHANGELOG.md",
    "Cargo.toml",
)


class ValidateReleasePullRequestTests(unittest.TestCase):
    def run_validator(
        self,
        *,
        head_ref: str = "release-please--branches--main",
        base_ref: str = "main",
        author: str = "github-actions[bot]",
        files: tuple[str, ...] = REQUIRED_FILES,
    ) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                sys.executable,
                str(VALIDATOR),
                "--head-ref",
                head_ref,
                "--base-ref",
                base_ref,
                "--author",
                author,
                *files,
            ],
            capture_output=True,
            check=False,
            text=True,
        )

    def test_accepts_expected_release_please_changes(self) -> None:
        for files in (REQUIRED_FILES, (*REQUIRED_FILES, "Cargo.lock")):
            with self.subTest(files=files):
                result = self.run_validator(files=files)
                self.assertEqual(result.returncode, 0, result.stderr)

    def test_rejects_untrusted_metadata(self) -> None:
        cases = (
            {"head_ref": "feature/not-release-please"},
            {"base_ref": "maintenance"},
            {"author": "untrusted-user"},
        )

        for arguments in cases:
            with self.subTest(arguments=arguments):
                result = self.run_validator(**arguments)
                self.assertNotEqual(result.returncode, 0)

    def test_rejects_incomplete_or_unexpected_changes(self) -> None:
        cases = (
            ("CHANGELOG.md", "Cargo.toml"),
            (*REQUIRED_FILES, "crates/vx-cli/src/main.rs"),
        )

        for files in cases:
            with self.subTest(files=files):
                result = self.run_validator(files=files)
                self.assertNotEqual(result.returncode, 0)


if __name__ == "__main__":
    unittest.main()
