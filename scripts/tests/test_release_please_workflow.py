"""Security and delivery contract tests for Release Please workflows."""

from pathlib import Path
import unittest


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
RELEASE_WORKFLOW = REPOSITORY_ROOT / ".github" / "workflows" / "release-please.yml"
RELEASE_PR_WORKFLOW = REPOSITORY_ROOT / ".github" / "workflows" / "ci-release-pr.yml"


class ReleasePleaseWorkflowTests(unittest.TestCase):
    def test_push_workflow_validates_bot_prs_and_reports_required_statuses(self) -> None:
        workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("statuses: write", workflow)
        self.assertIn("steps.release.outputs.prs_created == 'true'", workflow)
        self.assertIn("RELEASE_PRS: ${{ steps.release.outputs.prs }}", workflow)
        self.assertIn("ref: ${{ github.sha }}", workflow)
        self.assertIn("scripts/validate_pr_title.py", workflow)
        self.assertIn("scripts/validate_release_pr.py", workflow)
        self.assertIn(
            'post_status "$head_sha" "$title_state" "PR Title"', workflow
        )
        self.assertIn(
            'post_status "$head_sha" "$release_state" "CI Success"', workflow
        )
        self.assertIn(".head.sha", workflow)

    def test_release_statuses_use_the_required_check_names(self) -> None:
        """Both publishers must post the exact contexts the ruleset requires.

        A release pull request never runs `ci.yml`, so the status posted here is
        the only thing that can satisfy "CI Success". Spelling it any other way
        (for example the `CI / CI Success` form `gh pr checks` displays) leaves
        the release pull request blocked forever.
        """

        for workflow_path in (RELEASE_WORKFLOW, RELEASE_PR_WORKFLOW):
            with self.subTest(workflow=workflow_path.name):
                workflow = workflow_path.read_text(encoding="utf-8")
                self.assertIn('"CI Success"', workflow)
                self.assertNotIn("CI / CI Success", workflow)

    def test_event_driven_fallback_uses_the_same_release_pr_validator(self) -> None:
        workflow = RELEASE_PR_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("ref: ${{ github.event.pull_request.base.sha }}", workflow)
        self.assertIn("scripts/validate_release_pr.py", workflow)
        self.assertNotIn("'Cargo.lock'\n            ];", workflow)


if __name__ == "__main__":
    unittest.main()
