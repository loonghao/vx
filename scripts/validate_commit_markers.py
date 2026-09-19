#!/usr/bin/env python3
"""Reject CI-skip markers in commit messages before they can reach ``main``.

GitHub suppresses *every* workflow run of a push event when the head commit
message contains one of its documented skip markers (``[skip ci]``,
``[ci skip]``, ``[no ci]``, ``[skip actions]``, ``[actions skip]``).

A squash merge composes the merge commit message from the pull request title
plus a generated bullet list of every commit on the branch, so a marker on an
otherwise harmless bot commit survives into the merge commit and silently
disables CI for that commit on ``main``. The marker is invisible to whoever
performs the merge because it hides in the generated body.

This validator scans pull request titles and commit messages for those markers
so the guard can fail the pull request *before* the merge happens.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field

# Markers documented by GitHub as suppressing workflow runs for a push.
# https://docs.github.com/actions/managing-workflow-runs/skipping-workflow-runs
SKIP_MARKERS: tuple[str, ...] = (
    "[skip ci]",
    "[ci skip]",
    "[no ci]",
    "[skip actions]",
    "[actions skip]",
)

# Matches any of the markers above, case-insensitively. Brackets are escaped so
# they are literal characters, not a character class.
MARKER_PATTERN = re.compile(
    "|".join(re.escape(marker) for marker in SKIP_MARKERS), re.IGNORECASE
)


@dataclass(frozen=True)
class Finding:
    """One marker occurrence inside one commit message."""

    ref: str
    subject: str
    line_number: int
    marker: str

    def as_dict(self) -> dict[str, object]:
        return {
            "ref": self.ref,
            "subject": self.subject,
            "line_number": self.line_number,
            "marker": self.marker,
        }


@dataclass(frozen=True)
class Commit:
    """A commit message together with the identifier that produced it."""

    ref: str
    message: str

    @property
    def subject(self) -> str:
        """First line of the message, which is what squash merges list first."""

        return self.message.splitlines()[0] if self.message else ""


@dataclass
class Report:
    """Result of scanning a set of commit messages."""

    findings: list[Finding] = field(default_factory=list)

    @property
    def ok(self) -> bool:
        return not self.findings

    def as_dict(self) -> dict[str, object]:
        return {
            "ok": self.ok,
            "findings": [finding.as_dict() for finding in self.findings],
        }

    def render(self) -> str:
        if self.ok:
            return "No CI-skip markers found in commit messages."

        lines = [
            f"Found {len(self.findings)} CI-skip marker(s) in commit messages.",
            "",
            "GitHub suppresses every workflow run for a commit whose message",
            "contains a skip marker. A squash merge carries these markers into",
            "the merge commit on the base branch, which disables CI there.",
            "",
        ]

        for finding in self.findings:
            lines.append(
                f"  {finding.ref} line {finding.line_number}: "
                f"{finding.marker} in {finding.subject!r}"
            )

        lines.extend(
            [
                "",
                "Rewrite the commit message without the marker, or amend the",
                "commit before merging.",
            ]
        )
        return "\n".join(lines)


def find_markers(message: str, ref: str = "") -> list[Finding]:
    """Return every marker occurrence in ``message`` with its line number."""

    subject = message.splitlines()[0] if message else ""
    findings: list[Finding] = []

    for index, line in enumerate(message.splitlines(), start=1):
        for match in MARKER_PATTERN.finditer(line):
            findings.append(
                Finding(
                    ref=ref,
                    subject=subject,
                    line_number=index,
                    marker=match.group(0),
                )
            )

    return findings


def validate_commits(commits: list[Commit]) -> Report:
    """Scan every commit and collect the findings in order."""

    report = Report()

    for commit in commits:
        report.findings.extend(find_markers(commit.message, commit.ref))

    return report


def parse_records(text: str) -> list[Commit]:
    """Parse ``<ref><TAB><json-encoded message>`` lines.

    Messages are multi-line, so the workflow encodes them with ``jq``'s
    ``tojson`` and keeps one commit per physical line. Lines without a tab are
    treated as a single-line message, which keeps the validator usable from
    hooks and tests without any encoding step.
    """

    commits: list[Commit] = []

    for number, line in enumerate(text.splitlines(), start=1):
        if not line.strip():
            continue

        if "\t" in line:
            ref, _, encoded = line.partition("\t")
            try:
                message = json.loads(encoded)
            except json.JSONDecodeError:
                message = encoded
            commits.append(Commit(ref=ref or f"line-{number}", message=message))
        else:
            commits.append(Commit(ref=f"line-{number}", message=line))

    return commits


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        description="Fail when commit messages contain CI-skip markers."
    )
    parser.add_argument(
        "path",
        nargs="?",
        default="-",
        help=(
            "File with one commit per line, '<ref><TAB><json-encoded message>' "
            "(lines without a tab are read as a single-line message). "
            "Reads stdin when omitted."
        ),
    )
    parser.add_argument(
        "--message",
        help="Validate a single commit message supplied on the command line.",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        dest="as_json",
        help="Emit findings as JSON instead of human readable text.",
    )
    args = parser.parse_args(argv[1:])

    if args.message is not None:
        commits = [Commit(ref="message", message=args.message)]
    else:
        if args.path == "-":
            text = sys.stdin.read()
        else:
            try:
                with open(args.path, encoding="utf-8") as handle:
                    text = handle.read()
            except OSError as error:
                sys.stderr.write(f"Cannot read {args.path}: {error}\n")
                return 2
        commits = parse_records(text)

    report = validate_commits(commits)

    if args.as_json:
        print(json.dumps(report.as_dict(), indent=2))
    elif report.ok:
        print(report.render())
    else:
        sys.stderr.write(report.render() + "\n")

    return 0 if report.ok else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
