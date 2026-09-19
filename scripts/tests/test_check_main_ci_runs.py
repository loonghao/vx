"""Behavior tests for the CI gate sentinel."""

from datetime import datetime, timedelta, timezone
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import textwrap
import unittest


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
SENTINEL = REPOSITORY_ROOT / "scripts" / "check_main_ci_runs.py"

NOW = datetime(2026, 9, 19, 12, 0, tzinfo=timezone.utc)
NOW_ARGUMENT = NOW.isoformat()


def commit(
    sha: str,
    message: str,
    author: str = "Hal",
    minutes_ago: int = 120,
) -> dict:
    return {
        "sha": sha,
        "message": message,
        "author": author,
        "committed_at": (NOW - timedelta(minutes=minutes_ago)).isoformat(),
    }


def push_run(sha: str) -> dict:
    return {"head_sha": sha, "event": "push"}


# The shape of the squash merge that bypassed CI on 2026-09-19: a clean subject,
# a generated bullet list of branch commits, and the marker on one of them.
INHERITED_MARKER_MESSAGE = textwrap.dedent(
    """\
    fix(ci): stop skipping every test job in change detection

    * fix(ci): stop skipping every test job in change detection

    * chore: regenerate workspace-hack (cargo-hakari) [skip ci]

    * fix(ci): harden change detection base resolution
    """
)


class CheckMainCiRunsTests(unittest.TestCase):
    def run_sentinel(
        self, arguments: list[str], stdin: str | None = None
    ) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(SENTINEL), *arguments],
            capture_output=True,
            check=False,
            text=True,
            input=stdin,
        )

    def test_flags_inherited_marker_as_bypass(self) -> None:
        document = {
            "commits": [commit("4562a26a", INHERITED_MARKER_MESSAGE)],
            "workflow_runs": [],
        }

        result = self.run_sentinel(["--input", "-", "--json", "--now", NOW_ARGUMENT], stdin=json.dumps(document))

        self.assertEqual(result.returncode, 1)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["ok"], False)
        self.assertEqual(
            [item["kind"] for item in payload["findings"]],
            ["bypass"],
        )
        self.assertEqual(payload["findings"][0]["severity"], "error")
        self.assertEqual(payload["findings"][0]["marker"], "[skip ci]")

    def test_inherited_marker_is_reported_even_when_a_run_exists(self) -> None:
        document = {
            "commits": [commit("4562a26a", INHERITED_MARKER_MESSAGE)],
            "workflow_runs": [push_run("4562a26a")],
        }

        result = self.run_sentinel(["--input", "-", "--json", "--now", NOW_ARGUMENT], stdin=json.dumps(document))

        self.assertEqual(result.returncode, 0)
        payload = json.loads(result.stdout)
        self.assertEqual([item["kind"] for item in payload["findings"]], ["marker-ineffective"])

    def test_treats_marker_in_subject_as_intentional(self) -> None:
        document = {
            "commits": [commit("aaaaaaaa", "docs: fix a typo [skip ci]")],
            "workflow_runs": [],
        }

        result = self.run_sentinel(["--input", "-", "--json", "--now", NOW_ARGUMENT], stdin=json.dumps(document))

        self.assertEqual(result.returncode, 0)
        payload = json.loads(result.stdout)
        self.assertEqual([item["kind"] for item in payload["findings"]], ["intentional-skip"])
        self.assertEqual(payload["findings"][0]["severity"], "warning")

    def test_does_not_flag_bot_authored_commits(self) -> None:
        document = {
            "commits": [
                commit(
                    "bbbbbbbb",
                    "chore: regenerate workspace-hack (cargo-hakari)",
                    author="github-actions[bot]",
                )
            ],
            "workflow_runs": [],
        }

        result = self.run_sentinel(["--input", "-", "--json", "--now", NOW_ARGUMENT], stdin=json.dumps(document))

        self.assertEqual(result.returncode, 0)
        payload = json.loads(result.stdout)
        self.assertEqual([item["kind"] for item in payload["findings"]], ["bot-commit"])

    def test_ignores_commits_inside_the_grace_period(self) -> None:
        document = {
            "commits": [commit("cccccccc", INHERITED_MARKER_MESSAGE, minutes_ago=2)],
            "workflow_runs": [],
        }

        result = self.run_sentinel(["--input", "-", "--json", "--now", NOW_ARGUMENT], stdin=json.dumps(document))

        self.assertEqual(result.returncode, 0)
        payload = json.loads(result.stdout)
        self.assertEqual([item["kind"] for item in payload["findings"]], ["too-recent"])

    def test_clean_history_reports_no_findings(self) -> None:
        document = {
            "commits": [
                commit("dddddddd", "fix(ci): harden change detection"),
                commit("eeeeeeee", "chore(deps): update rust crate anyhow to v1.0.104"),
            ],
            "workflow_runs": [push_run("dddddddd"), push_run("eeeeeeee")],
        }

        result = self.run_sentinel(["--input", "-", "--json", "--now", NOW_ARGUMENT], stdin=json.dumps(document))

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)["findings"], [])

    def test_missing_runs_warn_and_fail_under_strict(self) -> None:
        document = {
            "commits": [commit("ffffffff", "fix(ci): something without runs")],
            "workflow_runs": [],
        }

        result = self.run_sentinel(["--input", "-", "--json", "--now", NOW_ARGUMENT], stdin=json.dumps(document))
        self.assertEqual(result.returncode, 0)
        self.assertEqual(
            [item["kind"] for item in json.loads(result.stdout)["findings"]],
            ["missing-runs"],
        )

        strict = self.run_sentinel(
            ["--input", "-", "--json", "--strict"], stdin=json.dumps(document)
        )
        self.assertEqual(strict.returncode, 1)

    def test_emits_github_annotations_for_bypass(self) -> None:
        document = {
            "commits": [commit("4562a26a", INHERITED_MARKER_MESSAGE)],
            "workflow_runs": [],
        }

        result = self.run_sentinel(["--input", "-", "--now", NOW_ARGUMENT], stdin=json.dumps(document))

        self.assertEqual(result.returncode, 1)
        self.assertIn("::error::4562a26a bypass", result.stdout)

    def test_writes_json_report_to_file(self) -> None:
        document = {
            "commits": [commit("4562a26a", INHERITED_MARKER_MESSAGE)],
            "workflow_runs": [],
        }

        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "report.json"
            result = self.run_sentinel(
                ["--input", "-", "--output", str(output), "--now", NOW_ARGUMENT],
                stdin=json.dumps(document),
            )

            self.assertEqual(result.returncode, 1)
            payload = json.loads(output.read_text(encoding="utf-8"))
            self.assertEqual(payload["findings"][0]["kind"], "bypass")

    def test_rejects_invalid_json_input(self) -> None:
        result = self.run_sentinel(["--input", "-", "--now", NOW_ARGUMENT], stdin="not json")

        self.assertEqual(result.returncode, 2)
        self.assertIn("Invalid JSON input", result.stderr)


if __name__ == "__main__":
    unittest.main()
