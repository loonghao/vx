"""Regression tests for Docker CI's Runtime pin change filters."""

from pathlib import Path
import re
import shutil
import subprocess
import unittest


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = REPOSITORY_ROOT / ".github" / "workflows" / "test-docker.yml"
SCRIPT_WORKFLOW = REPOSITORY_ROOT / ".github" / "workflows" / "scripts-tests.yml"


class DockerWorkflowTests(unittest.TestCase):
    def setUp(self) -> None:
        self.workflow = WORKFLOW.read_text(encoding="utf-8")

    def test_push_trigger_includes_runtime_pin_files(self) -> None:
        push = self.workflow.split("\n  push:\n", 1)[1].split("\n  pull_request:", 1)[0]
        paths = re.findall(r'^\s+- "([^"]+)"$', push, re.MULTILINE)
        for path in ("vx.toml", "vx.lock"):
            with self.subTest(path=path):
                self.assertIn(path, paths)

    def test_pull_request_trigger_keeps_required_check_unconditional(self) -> None:
        pull_request = self.workflow.split("\n  pull_request:\n", 1)[1].split(
            "\n  workflow_dispatch:", 1
        )[0]
        self.assertNotIn("paths:", pull_request)
        self.assertNotIn("paths-ignore:", pull_request)

    def test_workflow_changes_run_script_regressions(self) -> None:
        script_workflow = SCRIPT_WORKFLOW.read_text(encoding="utf-8")
        for event, following in (
            ("push", "pull_request"),
            ("pull_request", "workflow_dispatch"),
        ):
            with self.subTest(event=event):
                trigger = script_workflow.split(f"\n  {event}:\n", 1)[1].split(
                    f"\n  {following}:", 1
                )[0]
                self.assertIn('".github/workflows/test-docker.yml"', trigger)

    @unittest.skipUnless(shutil.which("grep"), "grep is required for the Linux CI filter")
    def test_change_filter_selects_pin_and_existing_docker_inputs(self) -> None:
        match = re.search(
            r'''if echo "\$CHANGED" \| grep -qE '([^']+)'; then''', self.workflow
        )
        self.assertIsNotNone(match, "could not locate the actual Docker change filter")
        pattern = match.group(1)
        cases = {
            "vx.toml": True,
            "vx.lock": True,
            "pkg/docker/Dockerfile.prebuilt": True,
            "Dockerfile": True,
            ".github/workflows/test-docker.yml": True,
            "crates/vx-cli/src/main.rs": True,
            "Cargo.toml": True,
            "Cargo.lock": True,
            "README.md\nvx.lock\ndocs/guide.md": True,
            "": False,
            "README.md\ndocs/guide.md": False,
            "docs/vx.toml": False,
            "nested/vx.lock": False,
            "vx.toml.bak": False,
            "vx.locked": False,
            "vxXtoml": False,
        }
        for changed, expected in cases.items():
            with self.subTest(changed=changed):
                result = subprocess.run(
                    ["grep", "-qE", pattern],
                    input=changed + "\n",
                    capture_output=True,
                    text=True,
                    timeout=10,
                )
                self.assertIn(result.returncode, (0, 1), result.stderr)
                self.assertEqual(result.returncode == 0, expected)


if __name__ == "__main__":
    unittest.main()
