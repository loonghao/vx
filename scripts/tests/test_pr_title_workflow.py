"""Security and delivery contract tests for the PR title workflow."""

from pathlib import Path
import os
import subprocess
import sys
import tempfile
import unittest

try:  # pragma: no cover - the runner image ships PyYAML
    import yaml

    HAVE_YAML = True
except ImportError:  # pragma: no cover
    HAVE_YAML = False


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = REPOSITORY_ROOT / ".github" / "workflows" / "pr-title.yml"
PUBLISHER = REPOSITORY_ROOT / "scripts" / "pr_title_status.py"
VALIDATOR = REPOSITORY_ROOT / "scripts" / "validate_pr_title.py"


def posix(path: Path) -> str:
    """Return `path` in a form bash accepts.

    The workflow runs on Linux, where the runner's `RUNNER_TEMP` is already a
    POSIX path. On Windows a `C:\\...` path reaches bash with its backslashes
    treated as escapes, so it is rewritten to the `/c/...` form.
    """

    text = path.as_posix()
    if len(text) > 1 and text[1] == ":":
        text = f"/{text[0].lower()}{text[2:]}"
    return text


def workflow_steps(name: str) -> dict:
    """Return the step with `name` from the workflow."""

    document = yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))
    for step in document["jobs"]["validate"]["steps"]:
        if step.get("name") == name:
            return step
    raise AssertionError(f"no step named {name!r} in pr-title.yml")


def bash_roundtrips_variables() -> bool:
    """Return whether a `bash` launched here keeps its own variables.

    The workflow runs on Linux. Some Windows shells mangle the command line
    on its way into bash, which would fail a step-level test for a reason that
    has nothing to do with the workflow, so it is skipped there.
    """

    try:
        result = subprocess.run(
            ["bash", "-c", "export _vx_probe=1; echo ${_vx_probe}"],
            capture_output=True,
            text=True,
            timeout=30,
        )
    except (OSError, subprocess.SubprocessError):
        return False
    return result.stdout.strip() == "1"


HAVE_WORKING_BASH = bash_roundtrips_variables()


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
        self.assertIn("context={STATUS_CONTEXT}", self.publisher)
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

    # Each trigger gets its own concurrency slot. The chained path is keyed by
    # the source run, so two CI runs report independently; sharing one slot
    # would let the hourly sweep cancel a manual backfill.
    def test_concurrency_slots_are_keyed_per_trigger(self) -> None:
        self.assertIn(
            "github.event_name == 'pull_request_target' && format('pr-{0}', github.event.pull_request.number)",
            self.workflow,
        )
        self.assertIn(
            "github.event_name == 'workflow_run' && format('ci-{0}', github.event.workflow_run.id)",
            self.workflow,
        )
        self.assertIn(
            "github.event_name == 'workflow_dispatch' && format('dispatch-{0}', github.run_id)",
            self.workflow,
        )

    # Scoped to the cancel line: the group expression legitimately mentions
    # `workflow_dispatch`, and the point is that only the sweep may cancel.
    def test_only_the_sweep_cancels_an_in_progress_run(self) -> None:
        cancel_lines = [
            line.strip()
            for line in self.workflow.splitlines()
            if line.strip().startswith("cancel-in-progress:")
        ]

        self.assertEqual(cancel_lines, ["cancel-in-progress: ${{ github.event_name == 'schedule' }}"])

    # The dispatch input is an arbitrary string reaching a shell word inside
    # double quotes, where `$(...)` would be expanded.
    def test_dispatch_input_is_rejected_unless_it_is_a_pull_number(self) -> None:
        self.assertIn("grep -qE '^[0-9]+$'", self.workflow)
        self.assertIn("The 'pr' input must be a pull request number", self.workflow)
        self.assertIn("PULL_NUMBER = re.compile", self.publisher)
        self.assertIn("--pr must be a pull request number", self.publisher)

    def test_the_sweep_is_documented_as_a_backstop_not_a_guarantee(self) -> None:
        self.assertIn("best-effort", self.workflow)
        self.assertIn("60 days", self.workflow)

    def test_backfill_never_overwrites_a_verdict_another_run_published(self) -> None:
        self.assertIn("def backfill_targets(", self.publisher)
        self.assertIn("def needs_status(", self.publisher)

    # The pull request head must never be a source of the code that runs here,
    # including when the base predates the publisher and a fallback is needed.
    def test_the_pull_request_head_is_never_a_source_of_the_policy_code(self) -> None:
        extraction = self.workflow.split("Extract the title policy code")[1]
        self.assertNotIn("github.event.pull_request.head.sha", extraction)
        self.assertIn(
            "FALLBACK_REF: ${{ github.event.repository.default_branch }}", self.workflow
        )
        self.assertIn("git cat-file -e", extraction)


class StaleBaseRepoMixin:
    """Builds a throwaway repository whose base predates the publisher."""

    def make_repo(self, root: Path, base_has_publisher: bool) -> tuple[str, str]:
        """Create a repo and return its (base, head) revisions."""

        scripts = root / "scripts"
        scripts.mkdir(parents=True, exist_ok=True)
        (scripts / "validate_pr_title.py").write_text(
            VALIDATOR.read_text(encoding="utf-8"), encoding="utf-8"
        )
        if base_has_publisher:
            (scripts / "pr_title_status.py").write_text(
                PUBLISHER.read_text(encoding="utf-8"), encoding="utf-8"
            )

        self._git(root, "init", "-q")
        self._git(root, "config", "user.email", "test@example.com")
        self._git(root, "config", "user.name", "test")
        self._git(root, "add", "scripts/validate_pr_title.py")
        if base_has_publisher:
            self._git(root, "add", "scripts/pr_title_status.py")
        self._git(root, "commit", "-q", "-m", "base")
        base = self._git(root, "rev-parse", "HEAD")

        # The head always carries the publisher, like a branch based on
        # current main. The README gives the second commit content even when
        # the base already had the publisher.
        (scripts / "pr_title_status.py").write_text(
            PUBLISHER.read_text(encoding="utf-8"), encoding="utf-8"
        )
        (root / "README.md").write_text("branch tip\n", encoding="utf-8")
        self._git(root, "add", "scripts/pr_title_status.py", "README.md")
        self._git(root, "commit", "-q", "-m", "head")
        head = self._git(root, "rev-parse", "HEAD")
        return base, head

    def _git(self, root: Path, *args: str) -> str:
        result = subprocess.run(
            ["git", *args],
            cwd=root,
            capture_output=True,
            text=True,
            check=False,
        )
        if result.returncode != 0:
            self.fail(f"git {' '.join(args)} failed: {result.stderr}")
        return result.stdout.strip()

    def extract(self, root: Path, base: str, fallback: str, target: Path) -> str:
        """Reproduce the extraction the workflow step performs.

        Reads the code from the base when the base has it and from the
        fallback otherwise. Both files are written together because the
        publisher imports the validator.
        """

        source = base
        probe = subprocess.run(
            ["git", "cat-file", "-e", f"{base}:scripts/pr_title_status.py"],
            cwd=root,
            capture_output=True,
            check=False,
        )
        if probe.returncode != 0:
            source = fallback

        target.mkdir(parents=True, exist_ok=True)
        for name in ("pr_title_status.py", "validate_pr_title.py"):
            shown = subprocess.run(
                ["git", "show", f"{source}:scripts/{name}"],
                cwd=root,
                capture_output=True,
                check=False,
            )
            if shown.returncode != 0:
                self.fail(f"git show {source}:scripts/{name} failed")
            (target / name).write_bytes(shown.stdout)
        return source

    def run_publisher(self, root: Path, directory: Path) -> int:
        """Run an extracted publisher on one synthetic pull request event."""

        event = root / "event.json"
        event.write_text(
            '{"pull_request": {"number": 1125,'
            ' "title": "fix(ci): absorb a flake",'
            ' "head": {"sha": "' + "0" * 40 + '"}}}',
            encoding="utf-8",
        )
        return self._run_publisher_with(root, directory, event)

    def _run_publisher_with(self, root: Path, directory: Path, event: Path) -> int:
        result = subprocess.run(
            [
                sys.executable,
                str(directory / "pr_title_status.py"),
                "--event-name",
                "pull_request_target",
                "--event-path",
                str(event),
                "--repository",
                "loonghao/vx",
                "--target-url",
                "https://example.test/run",
                "--dry-run",
            ],
            cwd=root,
            capture_output=True,
            text=True,
            check=False,
        )
        # The regression this guards: the step died with "can't open file"
        # before it could publish anything.
        self.assertNotIn("can't open file", result.stderr)
        self.assertNotIn("No such file or directory", result.stderr)
        return result.returncode


@unittest.skipUnless(HAVE_YAML, "PyYAML is required to read the workflow")
class StaleBaseRegressionTests(StaleBaseRepoMixin, unittest.TestCase):
    """The interactive path must work when the base predates the publisher.

    `pull_request_target` checks out the base commit on purpose. A pull
    request whose base predates `scripts/pr_title_status.py` used to make the
    step fail with "can't open file" before it could report anything.
    """

    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)

    def test_a_base_predating_the_publisher_falls_back_and_still_reports(self) -> None:
        base, head = self.make_repo(self.root, base_has_publisher=False)

        source = self.extract(self.root, base, head, self.root / "policy")

        self.assertEqual(source, head, "a stale base must fall back")
        self.assertTrue((self.root / "policy" / "validate_pr_title.py").is_file())
        self.assertEqual(self.run_publisher(self.root, self.root / "policy"), 0)

    def test_a_base_that_has_the_publisher_is_used_as_is(self) -> None:
        base, head = self.make_repo(self.root, base_has_publisher=True)

        source = self.extract(self.root, base, head, self.root / "policy")

        self.assertEqual(source, base)
        self.assertEqual(self.run_publisher(self.root, self.root / "policy"), 0)

    def test_the_extracted_publisher_still_rejects_a_bad_title(self) -> None:
        base, head = self.make_repo(self.root, base_has_publisher=False)
        self.extract(self.root, base, head, self.root / "policy")

        event = self.root / "bad.json"
        event.write_text(
            '{"pull_request": {"number": 999, "title": "not conventional",'
            ' "head": {"sha": "' + "0" * 40 + '"}}}',
            encoding="utf-8",
        )
        self.assertEqual(
            self._run_publisher_with(self.root, self.root / "policy", event), 1
        )


@unittest.skipUnless(HAVE_YAML, "PyYAML is required to read the workflow")
@unittest.skipUnless(HAVE_WORKING_BASH, "bash cannot run this workflow step here")
class StaleBaseStepTests(StaleBaseRepoMixin, unittest.TestCase):
    """Runs the extraction step from the workflow itself, not a copy of it."""

    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)

    def test_the_workflow_step_falls_back_for_a_stale_base(self) -> None:
        base, head = self.make_repo(self.root, base_has_publisher=False)
        step = workflow_steps("Extract the title policy code")

        script = "\n".join(
            [
                f"export BASE_REF={base}",
                f"export FALLBACK_REF={head}",
                f"export RUNNER_TEMP={posix(self.root / 'policy-dir')}",
                f"export GITHUB_OUTPUT={posix(self.root / 'output.txt')}",
                step["run"],
            ]
        )
        result = subprocess.run(
            ["bash", "-c", script],
            cwd=self.root,
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("base ref predates the title policy publisher", result.stdout)
        self.assertEqual(
            self.run_publisher(self.root, self.root / "policy-dir" / "pr-title-policy"),
            0,
        )


if __name__ == "__main__":
    unittest.main()
