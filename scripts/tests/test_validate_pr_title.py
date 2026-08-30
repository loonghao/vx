"""Behavior tests for the pull request title validator."""

from pathlib import Path
import subprocess
import sys
import unittest


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
VALIDATOR = REPOSITORY_ROOT / "scripts" / "validate_pr_title.py"


class ValidatePullRequestTitleTests(unittest.TestCase):
    def run_validator(self, title: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(VALIDATOR), title],
            capture_output=True,
            check=False,
            text=True,
        )

    def test_accepts_release_please_compatible_titles(self) -> None:
        titles = (
            "fix: preserve nested Windows PATH resolution",
            "feat(cli)!: change command behavior",
            "chore(deps): update rust crate flate2",
            "chore: release v0.9.31",
        )

        for title in titles:
            with self.subTest(title=title):
                result = self.run_validator(title)
                self.assertEqual(result.returncode, 0, result.stderr)

    def test_rejects_titles_release_please_cannot_parse(self) -> None:
        titles = (
            "Fix nested Windows PATH inheritance",
            "feature: add a runtime",
            "fix missing colon",
            "fix: ",
            "fix: valid first line\nchore: hidden second line",
        )

        for title in titles:
            with self.subTest(title=title):
                result = self.run_validator(title)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("Conventional Commit", result.stderr)


if __name__ == "__main__":
    unittest.main()
