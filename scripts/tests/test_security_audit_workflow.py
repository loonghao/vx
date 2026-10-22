"""Keep audit failures visible and audit the committed lock without regeneration."""

import os
from pathlib import Path
import re
import subprocess
import tempfile
import textwrap
import unittest


ROOT = Path(__file__).resolve().parents[2]
JUSTFILE = (ROOT / "justfile").read_text(encoding="utf-8")
CI = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
SCHEDULED = (ROOT / ".github/workflows/security-audit.yml").read_text(encoding="utf-8")
SCRIPT_TESTS = (ROOT / ".github/workflows/scripts-tests.yml").read_text(encoding="utf-8")


def recipe_lines() -> list[str]:
    match = re.search(r"^security-audit-ci:\n((?:[ \t].*\n|\n)*)", JUSTFILE, re.MULTILINE)
    if match is None:
        raise AssertionError("Missing security-audit-ci recipe")
    return [line.strip() for line in match.group(1).splitlines()
            if line.strip() and not line.lstrip().startswith("#")]


def job_body(workflow: str, name: str) -> str:
    match = re.search(rf"^  {re.escape(name)}:\n(.*?)(?=^  [\w-]+:|\Z)",
                      workflow, re.MULTILINE | re.DOTALL)
    if match is None:
        raise AssertionError(f"Missing CI job: {name}")
    return match.group(1)


class SecurityAuditWorkflowTests(unittest.TestCase):
    def test_recipe_audits_existing_lock_and_propagates_failure(self) -> None:
        # A leading '-' makes just ignore a failing command. Keep one command,
        # with no regeneration, error suppression, or weaker warning policy.
        self.assertEqual(recipe_lines(), ["vx cargo audit --deny warnings"])

    def test_scheduled_and_pr_audits_share_the_recipe(self) -> None:
        for workflow, name in ((CI, "security-audit"), (SCHEDULED, "cargo-audit")):
            with self.subTest(job=name):
                job = job_body(workflow, name)
                self.assertRegex(job, r"(?m)^        run: vx just security-audit-ci$")
                self.assertNotRegex(job, r"(?m)^        if:")
                self.assertNotIn("continue-on-error", job)
                self.assertNotIn("|| true", job)
                self.assertNotIn("generate-lockfile", job)
        # The weekly/manual audit must not be disabled at the job level either.
        self.assertNotRegex(job_body(SCHEDULED, "cargo-audit"), r"(?m)^    if:")

    def test_contract_runs_when_audit_entrypoints_change(self) -> None:
        triggers = SCRIPT_TESTS.split("permissions:", 1)[0]
        for path in ("justfile", ".github/workflows/security-audit.yml", ".github/workflows/ci.yml"):
            with self.subTest(path=path):
                self.assertEqual(triggers.count(f'      - "{path}"'), 2,
                                 "Both push and pull_request must run the regression tests")

    def test_recipe_command_preserves_audit_status_and_lock_bytes(self) -> None:
        commands = recipe_lines()
        self.assertEqual(commands, ["vx cargo audit --deny warnings"])
        # Execute the recipe's actual shell command with a controlled vx process.
        # This verifies orchestration, not cargo-audit's advisory database.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            lock = root / "Cargo.lock"
            original = b"# committed lock fixture\nversion = 4\n"
            lock.write_bytes(original)
            stub = root / "vx"
            stub.write_text("#!/bin/sh\nprintf '%s\\n' \"$*\" > args.txt\n"
                            'exit "$AUDIT_STATUS"\n', encoding="utf-8")
            stub.chmod(0o755)
            for status in (0, 1, 2, 101):
                with self.subTest(status=status):
                    result = subprocess.run(
                        ["bash", "-cu", commands[0]], cwd=root, check=False,
                        capture_output=True, text=True,
                        env={**os.environ, "PATH": f"{root}{os.pathsep}{os.environ['PATH']}",
                             "AUDIT_STATUS": str(status)},
                    )
                    self.assertEqual(result.returncode, status, result.stderr)
                    self.assertEqual((root / "args.txt").read_text(), "cargo audit --deny warnings\n")
                    self.assertEqual(lock.read_bytes(), original)

    def test_failed_or_cancelled_audit_fails_ci_success(self) -> None:
        gate = job_body(CI, "ci-success")
        self.assertIn("      - security-audit\n", gate)
        self.assertIn("if: always()", gate)
        script = textwrap.dedent(gate.split("        run: |\n", 1)[1])
        for status in ("success", "failure", "cancelled"):
            with self.subTest(status=status), tempfile.TemporaryDirectory() as directory:
                values = {"needs.security-audit.result": status,
                          "needs.detect-changes.outputs.should_test": "true"}
                substituted = re.sub(r"\$\{\{\s*(.*?)\s*\}\}",
                                     lambda match: values.get(match.group(1), "success"), script)
                result = subprocess.run(
                    ["bash", "-e", "-c", substituted], check=False, capture_output=True, text=True,
                    env={**os.environ, "GITHUB_STEP_SUMMARY": str(Path(directory) / "summary")},
                )
                self.assertEqual(result.returncode == 0, status == "success", result.stderr)


if __name__ == "__main__":
    unittest.main()
