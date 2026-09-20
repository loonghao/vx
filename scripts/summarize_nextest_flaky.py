#!/usr/bin/env python3
"""Surface nextest retry-absorbed flaky tests in the GitHub job summary.

`.config/nextest.toml` gives the environment-dependent test groups
`retries = 2`. nextest counts a test that fails at least once and then passes
as FLAKY and still exits 0, so an intermittent regression - a race, a network
stall, a tool download that times out once - lands as a green required check.
The job log does show a FLAKY line, but it is one row among thousands and the
step summary says nothing.

The JUnit report is the only output that records *why* a test passed: a flaky
test carries one `<flakyFailure>` child per failed attempt, while a test that
failed every attempt carries `<failure>`. This script turns those back into a
table, which keeps the retry behaviour - the point of the setting - while
restoring the visibility it costs.

It never fails the run. A missing or unreadable report becomes a warning in the
summary instead, because the job verdict belongs to nextest, and a summary
helper that turns a green job red would be worse than no helper.

Usage:

    summarize_nextest_flaky.py --junit target/nextest/default/junit.xml

The `--junit` path is resolved relative to the profile store directory, so the
default profile writes to `target/nextest/default/junit.xml` whenever
`CARGO_TARGET_DIR=target`.
"""

from __future__ import annotations

import argparse
import os
import sys
import xml.etree.ElementTree as ET
from dataclasses import dataclass
from pathlib import Path

# nextest writes one <flakyFailure> per failed attempt of a test that
# eventually passed, and <failure> (with <rerunFailure> siblings) for a test
# that never passed. Only the former is absorbed by the retry policy.
FLAKY_TAG = "flakyFailure"

NO_REPORT = "No nextest JUnit report was found at `{path}`."
UNREADABLE = "The nextest JUnit report at `{path}` could not be parsed: {error}"

EXPLANATION = """\
`retries` in `.config/nextest.toml` re-runs a failed test, and nextest reports
a test that only passes on a later attempt as FLAKY with exit code 0. These
tests passed *and* failed on this run - treat them as intermittent regressions
until proven otherwise rather than as a green light."""

CLEAN_LINE = (
    "No test needed a retry, so the `retries` setting absorbed no failure."
)


@dataclass(frozen=True)
class FlakyTest:
    """One test that failed at least one attempt and then passed."""

    binary_id: str
    name: str
    attempts: int
    first_failure: str

    @property
    def display(self) -> str:
        """The `binary::suite test` form nextest prints in its own output."""

        return f"{self.binary_id} {self.name}"


def _one_line(text: str) -> str:
    """Collapse a JUnit message to something that fits in a table cell."""

    collapsed = " ".join((text or "").split())
    # A pipe inside a cell would break the markdown table row.
    return collapsed.replace("|", "\\|")


def collect_flaky(path: Path) -> list[FlakyTest]:
    """Return every test in `path` that carries at least one flaky attempt."""

    root = ET.parse(path).getroot()
    flaky: list[FlakyTest] = []
    for testcase in root.iter("testcase"):
        failures = [child for child in testcase if child.tag == FLAKY_TAG]
        if not failures:
            continue
        flaky.append(
            FlakyTest(
                binary_id=testcase.get("classname", "") or testcase.get("name", ""),
                name=testcase.get("name", ""),
                # One <flakyFailure> per failed attempt, plus the final pass.
                attempts=len(failures) + 1,
                first_failure=failures[0].get("message", ""),
            )
        )
    return sorted(flaky, key=lambda test: (test.binary_id, test.name))


def render(flaky: list[FlakyTest], warning: str | None = None) -> str:
    """Render the job-summary markdown for `flaky` and an optional `warning`."""

    blocks: list[str] = []
    if warning:
        blocks.append(f"> [!WARNING]\n> {warning}")

    blocks.append(f"## Flaky tests absorbed by nextest retries ({len(flaky)})")
    blocks.append(EXPLANATION)

    if not flaky:
        blocks.append(CLEAN_LINE)
        return "\n\n".join(blocks) + "\n"

    rows = [
        "| Test | Attempts | First failure |",
        "| ---- | -------- | ------------- |",
    ]
    rows.extend(
        f"| `{test.display}` | {test.attempts} | {_one_line(test.first_failure)} |"
        for test in flaky
    )
    blocks.append("\n".join(rows))
    return "\n\n".join(blocks) + "\n"


def summarize(path: Path) -> str:
    """Render the summary for `path`, degrading to a warning if unreadable."""

    if not path.is_file():
        return render([], NO_REPORT.format(path=path))
    try:
        flaky = collect_flaky(path)
    except ET.ParseError as error:
        return render([], UNREADABLE.format(path=path, error=error))
    return render(flaky)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--junit",
        required=True,
        type=Path,
        help="Path to the nextest JUnit report.",
    )
    parser.add_argument(
        "--summary-file",
        type=Path,
        default=None,
        help=(
            "Markdown file to append to. Defaults to $GITHUB_STEP_SUMMARY when "
            "it is set, so the report shows up in the job summary."
        ),
    )
    args = parser.parse_args(argv)

    markdown = summarize(args.junit)
    print(markdown)

    summary_file = args.summary_file or os.environ.get("GITHUB_STEP_SUMMARY")
    if summary_file:
        with open(summary_file, "a", encoding="utf-8") as handle:
            handle.write(markdown)
            handle.write("\n")

    # Deliberately 0 even when the report is missing or flaky tests exist: this
    # job's verdict is nextest's, and the point here is only visibility.
    return 0


if __name__ == "__main__":
    sys.exit(main())
