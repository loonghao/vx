"""Behavior tests for the CI-skip marker validator."""

import json
from pathlib import Path
import subprocess
import sys
import unittest


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
VALIDATOR = REPOSITORY_ROOT / "scripts" / "validate_commit_markers.py"


def encode_records(records: list[tuple[str, str]]) -> str:
    """Encode commits the way the workflow does: one JSON message per line."""

    return "".join(f"{ref}\t{json.dumps(message)}\n" for ref, message in records)


class ValidateCommitMarkersTests(unittest.TestCase):
    def run_validator(
        self, arguments: list[str], stdin: str | None = None
    ) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(VALIDATOR), *arguments],
            capture_output=True,
            check=False,
            text=True,
            input=stdin,
        )

    def test_accepts_clean_messages(self) -> None:
        messages = (
            "fix(ci): resolve the merge base before diffing",
            "chore: regenerate workspace-hack (cargo-hakari)",
            "docs: mention the skip ci guard\n\nBody text without markers.",
        )

        for message in messages:
            with self.subTest(message=message):
                result = self.run_validator(["--message", message])
                self.assertEqual(result.returncode, 0, result.stderr)

    def test_rejects_every_documented_marker(self) -> None:
        markers = (
            "[skip ci]",
            "[ci skip]",
            "[no ci]",
            "[skip actions]",
            "[actions skip]",
            "[SKIP CI]",
        )

        for marker in markers:
            with self.subTest(marker=marker):
                result = self.run_validator(
                    ["--message", f"chore: regenerate workspace-hack {marker}"]
                )
                self.assertEqual(result.returncode, 1)
                self.assertIn(marker, result.stderr)

    def test_reports_line_number_of_inherited_marker(self) -> None:
        message = (
            "fix(ci): stop skipping every test job\n"
            "\n"
            "* fix(ci): stop skipping every test job\n"
            "\n"
            "* chore: regenerate workspace-hack (cargo-hakari) [skip ci]\n"
        )

        result = self.run_validator(["--message", message, "--json"])

        self.assertEqual(result.returncode, 1)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["ok"], False)
        self.assertEqual(len(payload["findings"]), 1)
        self.assertEqual(payload["findings"][0]["line_number"], 5)
        self.assertEqual(payload["findings"][0]["marker"], "[skip ci]")

    def test_scans_encoded_commit_records_from_stdin(self) -> None:
        records = [
            ("aaaaaaaaaaaa", "fix(ci): harden change detection"),
            ("c23ff168f2be", "chore: regenerate workspace-hack (cargo-hakari) [skip ci]"),
        ]

        result = self.run_validator([], stdin=encode_records(records))

        self.assertEqual(result.returncode, 1)
        self.assertIn("c23ff168f2be", result.stderr)
        self.assertNotIn("aaaaaaaaaaaa", result.stderr)

    def test_flags_marker_in_pull_request_title(self) -> None:
        records = [("pr-title", "chore(deps): bump anyhow [skip ci]")]

        result = self.run_validator([], stdin=encode_records(records))

        self.assertEqual(result.returncode, 1)
        self.assertIn("pr-title", result.stderr)

    def test_reports_unreadable_input_file(self) -> None:
        result = self.run_validator(["does-not-exist.txt"])

        self.assertEqual(result.returncode, 2)
        self.assertIn("Cannot read", result.stderr)


if __name__ == "__main__":
    unittest.main()
