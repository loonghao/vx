"""Security and delivery contract tests for the PR title workflow."""

from pathlib import Path
import unittest


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = REPOSITORY_ROOT / ".github" / "workflows" / "pr-title.yml"


class PullRequestTitleWorkflowTests(unittest.TestCase):
    def test_validates_trusted_base_code_and_reports_on_pr_head(self) -> None:
        workflow = WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("pull_request_target:", workflow)
        self.assertIn("types: [opened, edited, synchronize, reopened]", workflow)
        self.assertIn("statuses: write", workflow)
        self.assertIn("ref: ${{ github.event.pull_request.base.sha }}", workflow)
        self.assertIn("scripts/validate_pr_title.py", workflow)
        self.assertIn("github.event.pull_request.head.sha", workflow)
        self.assertIn('context="PR Title"', workflow)
        self.assertNotIn("ref: ${{ github.event.pull_request.head.sha }}", workflow)


if __name__ == "__main__":
    unittest.main()
