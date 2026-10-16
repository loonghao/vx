"""Keep native installer regression coverage connected to the required CI gate."""

import os
from pathlib import Path
import re
import subprocess
import tempfile
import textwrap
import unittest


ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
INSTALLER_JOB = "test-installer-extended-formats"


def job_body(name: str) -> str:
    """Read a top-level job without adding a YAML dependency to script tests."""
    match = re.search(rf"^  {re.escape(name)}:\n(.*?)(?=^  [\w-]+:|\Z)",
                      WORKFLOW, re.MULTILINE | re.DOTALL)
    if match is None:
        raise AssertionError(f"Missing CI job: {name}")
    return match.group(1)


class InstallerExtendedFormatsWorkflowTests(unittest.TestCase):
    def test_native_matrix_covers_all_supported_operating_systems(self) -> None:
        job = job_body(INSTALLER_JOB)
        self.assertEqual(re.findall(r"^            os: (.+)$", job, re.MULTILINE),
                         ["ubuntu-latest", "windows-latest", "macos-latest"])
        self.assertIn("runs-on: ${{ matrix.platform.os }}", job)
        self.assertIn("fail-fast: false", job)
        self.assertIn("needs: [detect-changes, build-vx]", job)
        self.assertIn("if: needs.detect-changes.outputs.should_test == 'true'", job)
        self.assertIn("name: ${{ matrix.platform.artifact }}", job)

    def test_real_regressions_are_feature_enabled_and_cannot_silently_skip(self) -> None:
        job = job_body(INSTALLER_JOB)
        command = " ".join(job.split("        run: >-\n", 1)[1].split())
        self.assertTrue(command.startswith("vx cargo nextest run --locked -p vx-installer "))
        for required in ("--features extended-formats", "--test sevenz_sfx_tests",
                         "--no-fail-fast", "--no-tests fail", "--retries 0"):
            self.assertIn(required, command)
        for forbidden in ("--no-run", "--target", "--workspace", "|| true",
                          "continue-on-error", "--run-ignored"):
            self.assertNotIn(forbidden, command)
        self.assertNotIn("continue-on-error", job)
        self.assertIn('install-nextest: "true"', job)

    def test_contract_and_success_gate_are_wired_into_ci(self) -> None:
        self.assertIn(Path(__file__).name, job_body("detect-changes"))
        self.assertIn(f"      - {INSTALLER_JOB}\n", job_body("ci-success"))
        self.assertIn("if: always()", job_body("ci-success"))

    def test_public_archive_regressions_are_feature_enabled_and_required(self) -> None:
        job = job_body(INSTALLER_JOB)
        step = job.split("      - name: Run public archive 7z path regressions\n", 1)[1]
        command = " ".join(step.split("        run: >-\n", 1)[1].split())
        self.assertTrue(command.startswith("vx cargo nextest run --locked -p vx-runtime-archive "))
        for required in ("--features extended-formats", "--test sevenz_path_tests",
                         "--no-fail-fast", "--no-tests fail", "--retries 0"):
            self.assertIn(required, command)
        for forbidden in ("--no-run", "--target", "--workspace", "|| true",
                          "continue-on-error", "--run-ignored"):
            self.assertNotIn(forbidden, command)

    def test_whole_workspace_coverage_is_preserved(self) -> None:
        coverage = job_body("coverage")
        self.assertIn("github.event.inputs.force_full_ci == 'true'", coverage)
        self.assertIn("run: vx just coverage-ci", coverage)
        self.assertIn("timeout-minutes: 45", coverage)
        self.assertIn("cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info",
                      (ROOT / "justfile").read_text(encoding="utf-8"))
        self.assertIn('check_optional_job "coverage" "${{ needs.coverage.result }}"',
                      job_body("ci-success"))

    def run_success_gate(self, selected: bool, result: str,
                         coverage: str = "skipped") -> subprocess.CompletedProcess:
        # Execute the workflow's actual shell gate, substituting Actions values.
        # This tests orchestration only; extraction itself runs natively in CI.
        script = textwrap.dedent(job_body("ci-success").split("        run: |\n", 1)[1])
        values = {
            "needs.detect-changes.outputs.should_test": str(selected).lower(),
            f"needs.{INSTALLER_JOB}.result": result,
            "needs.coverage.result": coverage,
        }
        script = re.sub(r"\$\{\{\s*(.*?)\s*\}\}",
                        lambda match: values.get(match.group(1), "success"), script)
        with tempfile.TemporaryDirectory() as directory:
            return subprocess.run(
                ["bash", "-e", "-c", script], check=False, capture_output=True, text=True,
                env={**os.environ, "GITHUB_STEP_SUMMARY": str(Path(directory) / "summary")},
            )

    def test_selected_installer_regressions_require_success(self) -> None:
        for result in ("failure", "cancelled", "skipped", ""):
            with self.subTest(result=result):
                self.assertNotEqual(self.run_success_gate(True, result).returncode, 0)
        self.assertEqual(self.run_success_gate(True, "success").returncode, 0)

    def test_unselected_installer_regressions_may_skip_but_not_fail(self) -> None:
        self.assertEqual(self.run_success_gate(False, "skipped").returncode, 0)
        for result in ("failure", "cancelled"):
            with self.subTest(result=result):
                self.assertNotEqual(self.run_success_gate(False, result).returncode, 0)

    def test_coverage_failure_or_cancellation_still_fails_the_gate(self) -> None:
        for result in ("failure", "cancelled"):
            with self.subTest(result=result):
                self.assertNotEqual(self.run_success_gate(True, "success", result).returncode, 0)


if __name__ == "__main__":
    unittest.main()
