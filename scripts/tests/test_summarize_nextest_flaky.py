"""Unit tests for the nextest flaky-test summarizer."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

SCRIPTS_DIR = Path(__file__).resolve().parents[1]
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import summarize_nextest_flaky as summary  # noqa: E402


def junit(*cases: str, name: str = "nextest-run") -> str:
    """Wrap `<testcase>` fragments into a full JUnit document."""

    body = "\n".join(cases)
    return (
        f'<?xml version="1.0" encoding="UTF-8"?>\n'
        f'<testsuites name="{name}" tests="{len(cases)}" failures="0">\n'
        f'    <testsuite name="vx-cli::rust" tests="{len(cases)}">\n'
        f"{body}\n"
        f"    </testsuite>\n"
        f"</testsuites>\n"
    )


def passing(test: str) -> str:
    return f'<testcase name="{test}" classname="vx-cli::rust" />'


def _flaky_attempt(message: str, text: str, attempt: int) -> str:
    """One `<flakyFailure>`, with a body only when nextest would write one."""

    attribute = f' message="{message} #{attempt}"' if message else ""
    if not text:
        return f"<flakyFailure{attribute} type=\"test failure\" />"
    return (
        f"<flakyFailure{attribute} type=\"test failure\">{text}</flakyFailure>"
    )


def flaky(
    test: str, attempts: int = 1, message: str = "boom", text: str = ""
) -> str:
    """A test that failed `attempts` times and then passed.

    `message` is the attribute nextest fills with the panic location and
    `text` the element body it fills with the panic reason, so both are
    settable here.
    """

    attempts_markup = [
        _flaky_attempt(message, text, n) for n in range(1, attempts + 1)
    ]
    return (
        f'<testcase name="{test}" classname="vx-cli::rust">\n'
        f'{"\n".join(attempts_markup)}\n</testcase>'
    )


def failing(test: str, reruns: int = 2) -> str:
    """A test that failed every attempt - a hard failure, not a flake."""

    rerun = "\n".join(
        f'<rerunFailure message="boom" type="test failure" />' for _ in range(reruns)
    )
    return (
        f'<testcase name="{test}" classname="vx-cli::rust">\n'
        f'<failure message="boom" type="test failure" />\n{rerun}\n</testcase>'
    )


class CollectFlakyTests(unittest.TestCase):
    def _collect(self, xml: str) -> list[str]:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "junit.xml"
            path.write_text(xml, encoding="utf-8")
            return [test.display for test in summary.collect_flaky(path)]

    def test_reports_a_test_that_needed_one_retry(self) -> None:
        self.assertEqual(
            self._collect(junit(flaky("test_a", attempts=1))),
            ["vx-cli::rust test_a"],
        )

    def test_ignores_a_test_that_passed_first_time(self) -> None:
        self.assertEqual(self._collect(junit(passing("test_a"))), [])

    def test_ignores_a_test_that_failed_every_attempt(self) -> None:
        # A deterministic regression is already red; it is not what the retry
        # policy hides, so it must not appear in a flaky report.
        self.assertEqual(self._collect(junit(failing("test_a"))), [])

    def test_separates_flaky_from_hard_failures(self) -> None:
        self.assertEqual(
            self._collect(junit(passing("p"), flaky("f"), failing("x"))),
            ["vx-cli::rust f"],
        )

    def test_counts_every_failed_attempt_plus_the_pass(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "junit.xml"
            path.write_text(junit(flaky("test_a", attempts=2)), encoding="utf-8")
            found = summary.collect_flaky(path)
        self.assertEqual(found[0].attempts, 3)

    def test_keeps_the_first_failure_message(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "junit.xml"
            path.write_text(
                junit(flaky("test_a", attempts=2, message="timed out")),
                encoding="utf-8",
            )
            found = summary.collect_flaky(path)
        self.assertEqual(found[0].first_failure, "timed out #1")

    def test_prefers_the_reason_over_the_panic_location(self) -> None:
        # Real nextest output: the `message` attribute carries only the panic
        # location, so the reason has to come from the element body.
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "junit.xml"
            path.write_text(
                junit(
                    flaky(
                        "test_a",
                        message=(
                            "thread 'always_fails' (102064) panicked at "
                            "tests\\probe.rs:3:5"
                        ),
                        text=(
                            "thread 'always_fails' (102064) panicked at "
                            "tests\\probe.rs:3:5:\n"
                            "assertion `left == right` failed\n"
                            "  left: 1\n"
                            " right: 2\n"
                            "note: run with `RUST_BACKTRACE=1` ..."
                        ),
                    )
                ),
                encoding="utf-8",
            )
            found = summary.collect_flaky(path)
        self.assertEqual(
            found[0].first_failure,
            "assertion `left == right` failed left: 1 right: 2 "
            "(at tests\\probe.rs:3:5)",
        )

    def test_skips_a_location_header_that_is_not_in_the_attribute(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "junit.xml"
            path.write_text(
                junit(
                    flaky(
                        "test_a",
                        message="",
                        text=(
                            "thread 'main' panicked at tests/foo.rs:24:9:\n"
                            "explicit panic\n"
                            "note: run with `RUST_BACKTRACE=1` ..."
                        ),
                    )
                ),
                encoding="utf-8",
            )
            found = summary.collect_flaky(path)
        self.assertEqual(found[0].first_failure, "explicit panic")

    def test_keeps_a_non_location_attribute_as_detail(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "junit.xml"
            path.write_text(
                junit(
                    flaky(
                        "test_a",
                        message="timed out",
                        text=(
                            "command exited after 120s\n"
                            "stdout:\n"
                            "  (empty)"
                        ),
                    )
                ),
                encoding="utf-8",
            )
            found = summary.collect_flaky(path)
        self.assertEqual(
            found[0].first_failure,
            "command exited after 120s stdout: (empty) (timed out #1)",
        )

    def test_drops_an_attribute_that_repeats_the_body(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "junit.xml"
            path.write_text(
                junit(flaky("test_a", message="boom", text="boom\nnote: rerun")),
                encoding="utf-8",
            )
            found = summary.collect_flaky(path)
        self.assertEqual(found[0].first_failure, "boom")

    def test_caps_a_long_failure_body(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "junit.xml"
            path.write_text(
                junit(
                    flaky(
                        "test_a",
                        message="",
                        text="\n".join(f"line {n}" for n in range(1, 10)),
                    )
                ),
                encoding="utf-8",
            )
            found = summary.collect_flaky(path)
        self.assertEqual(found[0].first_failure, "line 1 line 2 line 3")

    def test_orders_results_by_binary_then_test(self) -> None:
        xml = (
            '<?xml version="1.0" encoding="UTF-8"?>\n<testsuites>\n'
            '  <testsuite name="s">\n'
            '    <testcase name="z" classname="vx::b">'
            '<flakyFailure message="m" /></testcase>\n'
            '    <testcase name="a" classname="vx::b">'
            '<flakyFailure message="m" /></testcase>\n'
            '    <testcase name="q" classname="vx::a">'
            '<flakyFailure message="m" /></testcase>\n'
            "  </testsuite>\n</testsuites>\n"
        )
        self.assertEqual(
            self._collect(xml), ["vx::a q", "vx::b a", "vx::b z"]
        )


class RenderTests(unittest.TestCase):
    def test_clean_run_says_no_failure_was_absorbed(self) -> None:
        markdown = summary.render([])
        self.assertIn("(0)", markdown)
        self.assertIn("absorbed no failure", markdown)

    def test_renders_one_table_row_per_flaky_test(self) -> None:
        markdown = summary.render(
            [summary.FlakyTest("vx-cli::rust", "test_a", 2, "boom")]
        )
        self.assertIn("(1)", markdown)
        self.assertIn("| `vx-cli::rust test_a` | 2 | boom |", markdown)

    def test_keeps_the_table_contiguous(self) -> None:
        # A blank line between the separator row and the first data row makes
        # GitHub render the table as plain text.
        markdown = summary.render(
            [summary.FlakyTest("vx-cli::rust", "test_a", 2, "boom")]
        )
        lines = markdown.splitlines()
        start = lines.index("| Test | Attempts | First failure |")
        header, separator, row = lines[start : start + 3]
        self.assertTrue(header.startswith("| Test |"))
        self.assertEqual(separator, "| ---- | -------- | ------------- |")
        self.assertTrue(row.startswith("| `vx-cli::rust test_a` |"))

    def test_escapes_pipes_in_a_failure_message(self) -> None:
        markdown = summary.render(
            [summary.FlakyTest("vx-cli::rust", "test_a", 2, "a | b")]
        )
        self.assertIn(r"a \| b", markdown)

    def test_collapses_a_multiline_failure_message(self) -> None:
        markdown = summary.render(
            [summary.FlakyTest("vx-cli::rust", "test_a", 2, "one\n  two")]
        )
        self.assertIn("| one two |", markdown)

    def test_warning_precedes_the_heading(self) -> None:
        markdown = summary.render([], "no report")
        self.assertIn("> [!WARNING]", markdown)
        self.assertLess(markdown.index("[!WARNING]"), markdown.index("## Flaky"))


class MainTests(unittest.TestCase):
    def _run(self, xml: str | None) -> tuple[int, str, str]:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            report = root / "junit.xml"
            if xml is not None:
                report.write_text(xml, encoding="utf-8")
            summary_path = root / "summary.md"
            code = summary.main(
                ["--junit", str(report), "--summary-file", str(summary_path)]
            )
            return code, summary_path.read_text(encoding="utf-8"), str(report)

    def test_appends_the_report_to_the_summary_file(self) -> None:
        code, written, _ = self._run(junit(flaky("test_a")))
        self.assertEqual(code, 0)
        self.assertIn("| `vx-cli::rust test_a` |", written)

    def test_appends_rather_than_overwrites(self) -> None:
        with TemporaryDirectory() as tmp:
            summary_path = Path(tmp) / "summary.md"
            summary_path.write_text("existing\n", encoding="utf-8")
            report = Path(tmp) / "junit.xml"
            report.write_text(junit(passing("test_a")), encoding="utf-8")
            summary.main(
                ["--junit", str(report), "--summary-file", str(summary_path)]
            )
            self.assertTrue(
                summary_path.read_text(encoding="utf-8").startswith("existing")
            )

    def test_missing_report_warns_without_failing(self) -> None:
        code, written, report = self._run(None)
        self.assertEqual(code, 0)
        self.assertIn("No nextest JUnit report", written)
        self.assertIn(report.replace("\\", "/"), written.replace("\\", "/"))

    def test_unparseable_report_warns_without_failing(self) -> None:
        code, written, _ = self._run("<not xml")
        self.assertEqual(code, 0)
        self.assertIn("could not be read", written)

    def test_os_error_while_reading_warns_without_failing(self) -> None:
        # A summary helper that raises here would turn a green job red, so
        # every OSError - permission denied, I/O failure, a path that is
        # really a directory - has to degrade to the same warning.
        with TemporaryDirectory() as tmp:
            report = Path(tmp) / "junit.xml"
            report.write_text(junit(passing("test_a")), encoding="utf-8")
            with patch.object(summary.ET, "parse", side_effect=OSError("denied")):
                markdown = summary.summarize(report)
        self.assertIn("could not be read", markdown)
        self.assertIn("denied", markdown)

    def test_permission_error_on_stat_warns_without_failing(self) -> None:
        with TemporaryDirectory() as tmp:
            report = Path(tmp) / "junit.xml"
            report.write_text(junit(passing("test_a")), encoding="utf-8")
            with patch.object(
                Path, "is_file", side_effect=PermissionError("denied")
            ):
                markdown = summary.summarize(report)
        self.assertIn("could not be read", markdown)
        self.assertIn("denied", markdown)


if __name__ == "__main__":
    unittest.main()
