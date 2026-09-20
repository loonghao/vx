"""Unit tests for the `PR Title` status publisher."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
import unittest.mock
from pathlib import Path
from typing import Any

SCRIPTS_DIR = Path(__file__).resolve().parents[1]
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import pr_title_status as pub  # noqa: E402


def make_pull(
    number: int,
    sha: str,
    title: str,
    branch: str = "renovate/example",
    owner: str = "loonghao",
) -> dict[str, Any]:
    """Return a pull request payload shaped like the one the API returns."""

    return {
        "number": number,
        "title": title,
        "head": {
            "sha": sha,
            "ref": branch,
            "repo": {"owner": {"login": owner}},
        },
    }


def make_status(context: str, state: str = "success") -> dict[str, Any]:
    """Return one commit status entry."""

    return {"context": context, "state": state}


class FakeGitHub:
    """The slice of the GitHub API the publisher uses."""

    def __init__(
        self,
        pulls: list[dict[str, Any]],
        statuses: dict[str, list[dict[str, Any]]] | None = None,
        fail: bool = False,
    ) -> None:
        self.pulls = {str(pull["number"]): pull for pull in pulls}
        self._statuses = statuses or {}
        self.published: list[dict[str, Any]] = []
        self.calls: list[str] = []
        self.fail = fail
        self.owner = "loonghao"
        self.repository = "loonghao/vx"

    def _maybe_fail(self) -> None:
        if self.fail:
            raise pub.GhError("gh api failed: simulated")

    def pull(self, number: str) -> dict[str, Any]:
        self._maybe_fail()
        self.calls.append(f"pull:{number}")
        return self.pulls[str(number)]

    def open_pulls(self, head: str | None = None) -> list[dict[str, Any]]:
        self._maybe_fail()
        self.calls.append(f"open_pulls:{head}")
        if not head:
            return list(self.pulls.values())

        owner, _, branch = head.partition(":")
        return [
            pull
            for pull in self.pulls.values()
            if pull["head"]["ref"] == branch and pull["head"]["repo"]["owner"]["login"] == owner
        ]

    def statuses(self, sha: str) -> list[dict[str, Any]]:
        self._maybe_fail()
        self.calls.append(f"statuses:{sha}")
        return self._statuses.get(sha, [])

    def publish_status(
        self,
        sha: str,
        state: str,
        description: str,
        target_url: str,
    ) -> None:
        self._maybe_fail()
        self.published.append(
            {
                "sha": sha,
                "state": state,
                "description": description,
                "target_url": target_url,
            }
        )


class GhRunnerTests(unittest.TestCase):
    def test_every_call_goes_through_gh_api(self) -> None:
        calls: list[list[str]] = []

        def fake_run(argv: list[str], **_kwargs: object) -> object:
            calls.append(argv)

            class Result:
                returncode = 0
                stdout = "{}"
                stderr = ""

            return Result()

        with unittest.mock.patch.object(pub.subprocess, "run", fake_run):
            github = pub.GitHub("loonghao/vx")
            github.open_pulls()
            github.publish_status("0" * 40, "success", "description", "https://example.test")

        self.assertEqual(len(calls), 2)
        for argv in calls:
            self.assertEqual(argv[:2], ["gh", "api"])

    # Without pagination a repository with more open pull requests than one
    # page would lose the tail of the list silently.
    def test_open_pulls_follows_every_page(self) -> None:
        calls: list[list[str]] = []

        def fake_run(argv: list[str], **_kwargs: object) -> object:
            calls.append(argv)

            class Result:
                returncode = 0
                stdout = "[]"
                stderr = ""

            return Result()

        with unittest.mock.patch.object(pub.subprocess, "run", fake_run):
            pub.GitHub("loonghao/vx").open_pulls()

        self.assertIn("--paginate", calls[0])

    def test_open_pulls_keeps_the_head_filter(self) -> None:
        calls: list[list[str]] = []

        def fake_run(argv: list[str], **_kwargs: object) -> object:
            calls.append(argv)

            class Result:
                returncode = 0
                stdout = "[]"
                stderr = ""

            return Result()

        with unittest.mock.patch.object(pub.subprocess, "run", fake_run):
            pub.GitHub("loonghao/vx").open_pulls("loonghao:renovate/example")

        endpoint = calls[0][-1]
        self.assertIn("head=loonghao:renovate/example", endpoint)
        self.assertIn("state=open", endpoint)

    def test_a_failing_call_is_reported_as_an_error(self) -> None:
        def fake_run(argv: list[str], **_kwargs: object) -> object:
            class Result:
                returncode = 1
                stdout = ""
                stderr = "boom"

            return Result()

        with unittest.mock.patch.object(pub.subprocess, "run", fake_run):
            with self.assertRaises(pub.GhError):
                pub.GitHub("loonghao/vx").open_pulls()


class StatusSelectionTests(unittest.TestCase):
    def test_missing_context_needs_a_status(self) -> None:
        self.assertTrue(pub.needs_status([]))
        self.assertTrue(pub.needs_status([make_status("CI Success")]))

    def test_existing_context_does_not_need_a_status(self) -> None:
        self.assertFalse(pub.needs_status([make_status("PR Title")]))
        self.assertFalse(
            pub.needs_status([make_status("CI Success"), make_status("PR Title", "failure")])
        )

    def test_backfill_keeps_only_heads_without_the_status(self) -> None:
        stale = make_pull(1, "a" * 40, "chore(deps): update rust crate rstest to 0.27")
        fresh = make_pull(2, "b" * 40, "chore(deps): update rust crate rstest to 0.28")
        statuses = {stale["head"]["sha"]: [make_status("PR Title")]}

        selected = pub.pulls_without_status([stale, fresh], statuses)

        self.assertEqual([pull["number"] for pull in selected], [2])

    def test_backfill_never_overwrites_an_existing_verdict(self) -> None:
        failed = make_pull(3, "c" * 40, "not a conventional title")
        github = FakeGitHub([failed], {failed["head"]["sha"]: [make_status("PR Title", "failure")]})

        self.assertEqual(pub.backfill_targets(github, [failed]), [])
        self.assertEqual(github.published, [])


class WorkflowRunTargetTests(unittest.TestCase):
    def test_head_filter_uses_the_run_head_repository(self) -> None:
        event = {
            "workflow_run": {
                "head_branch": "renovate/example",
                "head_repository": {"owner": {"login": "fork-owner"}},
            }
        }

        self.assertEqual(
            pub.head_filter_for_workflow_run(event, "loonghao"),
            "fork-owner:renovate/example",
        )

    def test_head_filter_falls_back_to_the_repository_owner(self) -> None:
        event = {"workflow_run": {"head_branch": "main"}}

        self.assertEqual(pub.head_filter_for_workflow_run(event, "loonghao"), "loonghao:main")

    def test_head_filter_without_a_branch_has_no_target(self) -> None:
        self.assertIsNone(pub.head_filter_for_workflow_run({}, "loonghao"))
        self.assertIsNone(
            pub.head_filter_for_workflow_run({"workflow_run": {"head_branch": ""}}, "loonghao")
        )


class ResolveTargetsTests(unittest.TestCase):
    def test_pull_request_target_reports_on_the_event_pull_request(self) -> None:
        pull = make_pull(7, "d" * 40, "fix(ci): publish the title status")
        event = {"pull_request": pull}

        targets = pub.resolve_targets(FakeGitHub([pull]), "pull_request_target", event)

        self.assertEqual([item["number"] for item in targets], [7])

    def test_pull_request_target_without_a_payload_has_no_target(self) -> None:
        self.assertEqual(pub.resolve_targets(FakeGitHub([]), "pull_request_target", {}), [])

    def test_workflow_run_backfills_only_the_missing_head(self) -> None:
        reported = make_pull(8, "e" * 40, "chore(deps): update rust crate uuid to v1.26.1")
        missing = make_pull(9, "f" * 40, "chore(deps): update rust crate tokio to v1.53.1")
        github = FakeGitHub(
            [reported, missing],
            {reported["head"]["sha"]: [make_status("PR Title")]},
        )
        event = {
            "workflow_run": {
                "head_branch": "renovate/example",
                "head_repository": {"owner": {"login": "loonghao"}},
            }
        }

        targets = pub.resolve_targets(github, "workflow_run", event)

        self.assertEqual([pull["number"] for pull in targets], [9])

    def test_workflow_run_without_a_branch_has_no_target(self) -> None:
        github = FakeGitHub([make_pull(10, "0" * 40, "chore: something")])

        self.assertEqual(
            pub.resolve_targets(github, "workflow_run", {"workflow_run": {}}),
            [],
        )
        self.assertNotIn("open_pulls:None", github.calls)

    def test_sweep_backfills_every_open_pull_request(self) -> None:
        missing = make_pull(11, "1" * 40, "chore(deps): update rust crate log to 0.4.35")
        reported = make_pull(12, "2" * 40, "chore(deps): update rust crate anyhow to 1.0.105")
        github = FakeGitHub(
            [missing, reported],
            {reported["head"]["sha"]: [make_status("PR Title")]},
        )

        targets = pub.resolve_targets(github, "schedule", {})

        self.assertEqual([pull["number"] for pull in targets], [11])

    def test_requested_pull_request_bypasses_the_backfill_filter(self) -> None:
        reported = make_pull(13, "3" * 40, "chore: regenerate workspace-hack (cargo-hakari)")
        github = FakeGitHub([reported], {reported["head"]["sha"]: [make_status("PR Title")]})

        targets = pub.resolve_targets(github, "workflow_dispatch", {}, requested_pr="13")

        self.assertEqual([pull["number"] for pull in targets], [13])


class EvaluationTests(unittest.TestCase):
    def test_conventional_title_is_valid(self) -> None:
        github = FakeGitHub([])
        pull = make_pull(14, "4" * 40, "chore(deps): update rust crate rstest to 0.27")

        result = pub.evaluate(github, pull, "https://example.test/run/1")

        self.assertTrue(result.valid)
        self.assertEqual(result.state, "success")
        self.assertEqual(result.head_sha, pull["head"]["sha"])
        self.assertEqual(
            github.published,
            [
                {
                    "sha": pull["head"]["sha"],
                    "state": "success",
                    "description": pub.VALID_DESCRIPTION,
                    "target_url": "https://example.test/run/1",
                }
            ],
        )

    def test_non_conventional_title_is_invalid(self) -> None:
        github = FakeGitHub([])
        pull = make_pull(15, "5" * 40, "update rust crate rstest")

        result = pub.evaluate(github, pull, "https://example.test/run/2")

        self.assertFalse(result.valid)
        self.assertEqual(result.state, "failure")
        self.assertEqual(github.published[0]["description"], pub.INVALID_DESCRIPTION)

    def test_dry_run_publishes_nothing(self) -> None:
        github = FakeGitHub([])
        pull = make_pull(16, "6" * 40, "fix(windows): preserve nested PATH resolution")

        result = pub.evaluate(github, pull, "https://example.test/run/3", dry_run=True)

        self.assertTrue(result.valid)
        self.assertFalse(result.published)
        self.assertEqual(github.published, [])


class MainTests(unittest.TestCase):
    def setUp(self) -> None:
        self._environment = dict(pub.os.environ)
        pub.os.environ.pop("GITHUB_STEP_SUMMARY", None)

    def tearDown(self) -> None:
        pub.os.environ.clear()
        pub.os.environ.update(self._environment)

    def test_event_name_is_required(self) -> None:
        self.assertEqual(pub.main(["--repository", "loonghao/vx"]), pub.EXIT_UNAVAILABLE)

    def test_repository_is_required(self) -> None:
        self.assertEqual(pub.main(["--event-name", "schedule"]), pub.EXIT_UNAVAILABLE)

    def test_unpublished_status_reports_the_infrastructure_failure(self) -> None:
        github = FakeGitHub([make_pull(17, "7" * 40, "chore: something")], fail=True)

        exit_code = pub.run(github, "schedule", {}, "https://example.test/run/4")

        self.assertEqual(exit_code, pub.EXIT_UNAVAILABLE)

    def test_valid_titles_exit_clean(self) -> None:
        github = FakeGitHub([make_pull(18, "8" * 40, "chore(deps): update rust crate log to 0.4.35")])

        exit_code = pub.run(github, "schedule", {}, "https://example.test/run/5")

        self.assertEqual(exit_code, pub.EXIT_VALID)
        self.assertEqual(len(github.published), 1)

    def test_invalid_title_exits_with_the_policy_code(self) -> None:
        github = FakeGitHub([make_pull(19, "9" * 40, "update rust crate log")])

        exit_code = pub.run(github, "schedule", {}, "https://example.test/run/6")

        self.assertEqual(exit_code, pub.EXIT_INVALID)
        # The failure status is what blocks the merge, so it is published even
        # when the run itself is not failed.
        self.assertEqual(github.published[0]["state"], "failure")

    def test_report_and_summary_are_written(self) -> None:
        github = FakeGitHub([make_pull(20, "a" * 40, "chore(deps): update rust crate log to 0.4.35")])
        with tempfile.TemporaryDirectory() as tmp:
            report = Path(tmp) / "report.json"
            summary = Path(tmp) / "summary.md"
            pub.os.environ["GITHUB_STEP_SUMMARY"] = str(summary)

            exit_code = pub.run(
                github,
                "schedule",
                {},
                "https://example.test/run/7",
                report=str(report),
            )

            payload = json.loads(report.read_text(encoding="utf-8"))
            body = summary.read_text(encoding="utf-8")

        self.assertEqual(exit_code, pub.EXIT_VALID)
        self.assertEqual(payload["context"], pub.STATUS_CONTEXT)
        self.assertEqual(len(payload["evaluated"]), 1)
        self.assertTrue(payload["valid"])
        self.assertIn("PR Title status", body)
        self.assertIn("#20", body)

    def test_load_event_tolerates_a_missing_payload(self) -> None:
        self.assertEqual(pub.load_event(""), {})
        self.assertEqual(pub.load_event("/nonexistent/event.json"), {})

    # The dispatch input is an arbitrary string. Anything that is not a bare
    # number must be refused here as well, so the script is safe to call from
    # anywhere and not only from the guarded workflow step.
    def test_a_pull_request_number_must_be_digits(self) -> None:
        for value in ("12$(date)", "1;rm -rf", "-1", "12 34", "abc", "1.5", "+5"):
            with self.subTest(value=value):
                self.assertEqual(
                    pub.main(["--event-name", "workflow_dispatch", "--repository", "loonghao/vx", "--pr", value]),
                    pub.EXIT_UNAVAILABLE,
                )

    def test_a_numeric_pull_request_number_is_accepted(self) -> None:
        self.assertTrue(pub.PULL_NUMBER.match("1124"))
        self.assertTrue(pub.PULL_NUMBER.match("1"))


if __name__ == "__main__":
    unittest.main()
