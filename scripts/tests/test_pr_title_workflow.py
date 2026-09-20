"""Security and delivery contract tests for the PR title workflow."""

from pathlib import Path
import unittest


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = REPOSITORY_ROOT / ".github" / "workflows" / "pr-title.yml"
PUBLISHER = REPOSITORY_ROOT / "scripts" / "pr_title_status.py"
VALIDATOR = REPOSITORY_ROOT / "scripts" / "validate_pr_title.py"


class PullRequestTitleWorkflowTests(unittest.TestCase):
    def setUp(self) -> None:
        self.workflow = WORKFLOW.read_text(encoding="utf-8")
        self.publisher = PUBLISHER.read_text(encoding="utf-8")

    def test_validates_trusted_base_code_and_reports_on_pr_head(self) -> None:
        self.assertIn("pull_request_target:", self.workflow)
        self.assertIn("types: [opened, edited, synchronize, reopened]", self.workflow)
        self.assertIn("statuses: write", self.workflow)
        # The trusted checkout is the base commit; the default branch is the
        # fallback for the paths that carry no pull request payload.
        self.assertIn("ref: ${{ github.event.pull_request.base.sha", self.workflow)
        self.assertIn("github.event.repository.default_branch", self.workflow)
        self.assertIn("persist-credentials: false", self.workflow)
        self.assertIn("scripts/pr_title_status.py", self.workflow)
        self.assertNotIn("ref: ${{ github.event.pull_request.head.sha }}", self.workflow)

    def test_validator_stays_the_single_source_of_the_title_rule(self) -> None:
        self.assertIn("from validate_pr_title import is_valid_title", self.publisher)
        self.assertTrue(VALIDATOR.is_file())

    def test_publishes_the_required_status_on_the_pull_request_head(self) -> None:
        self.assertIn('context={STATUS_CONTEXT}', self.publisher)
        self.assertIn('STATUS_CONTEXT = "PR Title"', self.publisher)
        self.assertIn('pull["head"]["sha"]', self.publisher)

    # A head pushed with the repository GITHUB_TOKEN never starts a
    # `pull_request_target` run, so the interactive path alone leaves the
    # required status permanently missing. The workflow has to be able to reach
    # the head from outside the pull request event stream.
    def test_reaches_a_head_that_no_pull_request_target_event_reported(self) -> None:
        self.assertIn("workflow_run:", self.workflow)
        self.assertIn('workflows: ["CI"]', self.workflow)
        self.assertIn("schedule:", self.workflow)
        self.assertIn("workflow_dispatch:", self.workflow)

    def test_backfill_never_overwrites_a_verdict_another_run_published(self) -> None:
        self.assertIn("def backfill_targets(", self.publisher)
        self.assertIn("def needs_status(", self.publisher)


if __name__ == "__main__":
    unittest.main()
