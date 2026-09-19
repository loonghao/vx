#!/usr/bin/env python3
"""Detect commits on a branch whose CI gate was silently bypassed.

GitHub suppresses *every* workflow run of a push event when the head commit
message contains a skip marker. When that marker arrives through a squash
merge it is invisible to whoever merged: it hides in the generated bullet list
of branch commits. The commit lands on the base branch with a clean looking
subject and zero workflow runs, so no check ever reports the gate is gone.

This sentinel walks the recent commits of a branch, asks the Actions API how
many push-event runs exist for each of them, and classifies the ones that have
none. It is the fallback signal for the case where the pull request guard was
not in place (pull requests raised before it existed, direct pushes, or a
marker that reached the branch through another path).

Feeding it data instead of letting it call the API keeps it testable offline:

    {
      "commits": [
        {"sha": "...", "message": "...", "author": "...", "committed_at": "..."}
      ],
      "workflow_runs": [{"head_sha": "...", "event": "push"}]
    }

``--input`` reads such a document (``-`` for stdin). Without it the document is
fetched with the GitHub CLI.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
from pathlib import Path

# Markers documented by GitHub as suppressing workflow runs for a push.
SKIP_MARKERS: tuple[str, ...] = (
    "[skip ci]",
    "[ci skip]",
    "[no ci]",
    "[skip actions]",
    "[actions skip]",
)

# GitHub renders the squash body as a bullet list of the branch commits. A
# marker inside that generated block was inherited from a branch commit, which
# is exactly the invisible case this sentinel exists to catch.
ENUMERATION_LINE = re.compile(r"^\*\s")

BOT_NAME_MARKERS = ("[bot]", "github-actions", "dependabot", "renovate")

# Severity of a finding. Only errors fail the run; warnings are reported and
# only fail with --strict, because a commit can legitimately have no push runs.
ERROR = "error"
WARNING = "warning"
INFO = "info"


@dataclass(frozen=True)
class Finding:
    """One commit that does not have the CI coverage the branch expects."""

    sha: str
    subject: str
    kind: str
    severity: str
    detail: str
    marker: str = ""

    def as_dict(self) -> dict[str, object]:
        return {
            "sha": self.sha,
            "subject": self.subject,
            "kind": self.kind,
            "severity": self.severity,
            "detail": self.detail,
            "marker": self.marker,
        }


@dataclass
class Report:
    """Findings for every inspected commit, most recent first."""

    branch: str
    findings: list[Finding] = field(default_factory=list)

    @property
    def errors(self) -> list[Finding]:
        return [item for item in self.findings if item.severity == ERROR]

    @property
    def warnings(self) -> list[Finding]:
        return [item for item in self.findings if item.severity == WARNING]

    def as_dict(self) -> dict[str, object]:
        return {
            "branch": self.branch,
            "ok": not self.errors,
            "findings": [item.as_dict() for item in self.findings],
        }

    def render(self, strict: bool = False) -> str:
        if not self.findings:
            return f"Every inspected commit on {self.branch} has push-event workflow runs."

        lines = [f"CI gate report for {self.branch}:", ""]

        for item in self.findings:
            label = f"{item.severity}:{item.kind}"
            lines.append(f"  [{label}] {item.sha} {item.subject}")
            lines.append(f"      {item.detail}")

        lines.extend(
            [
                "",
                f"errors: {len(self.errors)}  warnings: {len(self.warnings)}"
                + ("  (strict: warnings fail)" if strict else ""),
            ]
        )
        return "\n".join(lines)


def is_bot_author(author: str) -> bool:
    """Whether a commit was authored by an automation account.

    Commits pushed with the repository ``GITHUB_TOKEN`` never start a workflow
    run, so a bot authored commit without push runs is expected, not a bypass.
    """

    lowered = (author or "").lower()
    return any(marker in lowered for marker in BOT_NAME_MARKERS)


def find_marker(message: str) -> tuple[str, bool] | None:
    """Return ``(marker, inherited)`` for the first marker in ``message``.

    ``inherited`` marks a marker that sits in the generated bullet list of a
    squash merge rather than in a message a human wrote.
    """

    lines = message.splitlines()

    enumeration_start = None
    for index, line in enumerate(lines):
        if ENUMERATION_LINE.match(line):
            enumeration_start = index
            break

    pattern = re.compile("|".join(re.escape(m) for m in SKIP_MARKERS), re.IGNORECASE)

    for index, line in enumerate(lines):
        match = pattern.search(line)
        if match is None:
            continue

        inherited = (
            enumeration_start is not None
            and index >= enumeration_start
            and (bool(ENUMERATION_LINE.match(line)) or line.startswith((" ", "\t")))
        )
        return match.group(0), inherited

    return None


def parse_timestamp(value: str) -> datetime | None:
    """Parse an ISO-8601 timestamp as produced by the GitHub API."""

    if not value:
        return None

    text = value.strip().replace("Z", "+00:00")
    try:
        parsed = datetime.fromisoformat(text)
    except ValueError:
        return None

    return parsed if parsed.tzinfo else parsed.replace(tzinfo=timezone.utc)


def push_run_counts(document: dict) -> dict[str, int]:
    """Count push-event runs per head SHA from an Actions API response list."""

    counts: dict[str, int] = {}

    for run in document.get("workflow_runs") or []:
        if (run or {}).get("event") != "push":
            continue
        sha = run.get("head_sha")
        if sha:
            counts[sha] = counts.get(sha, 0) + 1

    return counts


def evaluate(
    document: dict,
    branch: str = "main",
    now: datetime | None = None,
    min_age: timedelta = timedelta(0),
) -> Report:
    """Classify every commit in ``document`` by its push-event run coverage."""

    now = now or datetime.now(timezone.utc)
    counts = push_run_counts(document)
    report = Report(branch=branch)

    # Newest first, matching the order the commits API returns.
    for commit in document.get("commits") or []:
        sha = str(commit.get("sha", ""))
        message = commit.get("message") or ""
        subject = message.splitlines()[0] if message else ""
        author = commit.get("author") or ""

        committed_at = parse_timestamp(commit.get("committed_at", ""))
        if committed_at is not None and now - committed_at < min_age:
            report.findings.append(
                Finding(
                    sha=sha,
                    subject=subject,
                    kind="too-recent",
                    severity=INFO,
                    detail="Commit is younger than the grace period; runs may not exist yet.",
                )
            )
            continue

        runs = counts.get(sha, 0)
        marker = find_marker(message)

        # A marker decides the outcome before authorship does: a bot commit can
        # carry an inherited marker just like any other commit, and the missing
        # workflow runs are then still a bypass.
        if marker is not None and runs > 0:
            report.findings.append(
                Finding(
                    sha=sha,
                    subject=subject,
                    kind="marker-ineffective",
                    severity=WARNING,
                    detail=(
                        f"Commit message carries {marker[0]} but {runs} push run(s) "
                        "exist. The marker still suppresses the re-triggered runs."
                    ),
                    marker=marker[0],
                )
            )
            continue

        if marker is None and runs > 0:
            continue

        if marker is None:
            if is_bot_author(author):
                report.findings.append(
                    Finding(
                        sha=sha,
                        subject=subject,
                        kind="bot-commit",
                        severity=INFO,
                        detail=(
                            f"Authored by {author} with no push run; GITHUB_TOKEN pushes "
                            "start no workflow run, which is expected."
                        ),
                    )
                )
                continue

            report.findings.append(
                Finding(
                    sha=sha,
                    subject=subject,
                    kind="missing-runs",
                    severity=WARNING,
                    detail=(
                        "No push-event workflow run for this commit and no skip marker "
                        "in its message. It may be an intermediate commit of a multi-commit push."
                    ),
                )
            )
            continue

        text, inherited = marker
        if inherited:
            report.findings.append(
                Finding(
                    sha=sha,
                    subject=subject,
                    kind="bypass",
                    severity=ERROR,
                    detail=(
                        f"Commit message carries {text} inside the generated commit list of a "
                        "squash merge, so the marker reached the branch without being written "
                        "by the author. Every workflow run for this commit was suppressed."
                    ),
                    marker=text,
                )
            )
        else:
            report.findings.append(
                Finding(
                    sha=sha,
                    subject=subject,
                    kind="intentional-skip",
                    severity=WARNING,
                    detail=(
                        f"Commit message carries {text} in its own text, so the push event "
                        "was skipped deliberately. Confirm this was intended."
                    ),
                    marker=text,
                )
            )

    return report


def fetch(repo: str, branch: str, depth: int) -> dict:
    """Fetch the recent commits of ``branch`` and their push runs with ``gh``."""

    commits = json.loads(
        _gh(
            [
                "api",
                f"repos/{repo}/commits?sha={branch}&per_page={depth}",
                "--jq",
                "[.[] | {sha, message: .commit.message, author: .commit.author.name, committed_at: .commit.committer.date}]",
            ]
        )
    )

    workflow_runs: list[dict] = []
    for commit in commits:
        runs = json.loads(
            _gh(
                [
                    "api",
                    f"repos/{repo}/actions/runs?head_sha={commit['sha']}&event=push&per_page=1",
                    "--jq",
                    "[.workflow_runs[] | {head_sha, event}]",
                ]
            )
        )
        workflow_runs.extend(runs)

    return {"commits": commits, "workflow_runs": workflow_runs}


def _gh(arguments: list[str]) -> str:
    result = subprocess.run(
        ["gh", *arguments],
        capture_output=True,
        check=False,
        text=True,
    )

    if result.returncode != 0:
        raise RuntimeError(
            f"gh {' '.join(arguments)} failed: {result.stderr.strip() or result.stdout.strip()}"
        )

    return result.stdout


def write_summary(report: Report, strict: bool) -> None:
    """Append a markdown summary when running inside GitHub Actions."""

    summary_file = os.environ.get("GITHUB_STEP_SUMMARY", "").strip()
    if not summary_file:
        return

    summary_path = Path(summary_file)

    lines = ["### CI gate sentinel", "", f"Branch: `{report.branch}`", ""]

    if not report.findings:
        lines.append("Every inspected commit has push-event workflow runs. ✅")
    else:
        lines.append("| Commit | Subject | Severity | Kind |")
        lines.append("| --- | --- | --- | --- |")
        for item in report.findings:
            lines.append(
                f"| `{item.sha[:12]}` | {item.subject} | {item.severity} | {item.kind} |"
            )

        lines.append("")
        for item in report.findings:
            if item.severity == INFO:
                continue
            lines.append(f"- **{item.severity}** `{item.sha[:12]}` {item.kind}: {item.detail}")

    # A summary is a convenience, never a reason to lose the exit code.
    try:
        summary_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    except OSError as error:
        print(f"::warning::Could not write the job summary: {error}")


def emit_annotations(report: Report) -> None:
    """Emit GitHub workflow commands so failures surface in the run UI."""

    for item in report.findings:
        if item.severity == ERROR:
            print(f"::error::{item.sha[:12]} {item.kind}: {item.detail}")
        elif item.severity == WARNING:
            print(f"::warning::{item.sha[:12]} {item.kind}: {item.detail}")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        description="Detect commits on a branch whose push-event CI runs were suppressed."
    )
    parser.add_argument("--input", help="JSON document with commits and workflow_runs")
    parser.add_argument("--branch", default="main", help="Branch to inspect (default: main)")
    parser.add_argument("--repo", help="owner/repo for the GitHub API (default: current repo)")
    parser.add_argument("--depth", type=int, default=25, help="Commits to inspect")
    parser.add_argument(
        "--min-age-minutes",
        type=int,
        default=20,
        help="Ignore commits younger than this (default: 20)",
    )
    parser.add_argument(
        "--strict", action="store_true", help="Treat warnings as failures as well"
    )
    parser.add_argument(
        "--now",
        help="ISO-8601 timestamp used as 'now' instead of the clock (tests)",
    )
    parser.add_argument("--json", action="store_true", dest="as_json", help="Emit JSON")
    parser.add_argument("--output", help="Write the JSON report to this file")
    args = parser.parse_args(argv[1:])

    if args.input:
        text = sys.stdin.read() if args.input == "-" else Path(args.input).read_text(encoding="utf-8")
        try:
            document = json.loads(text)
        except json.JSONDecodeError as error:
            sys.stderr.write(f"Invalid JSON input: {error}\n")
            return 2
    else:
        try:
            repo = args.repo or _gh(
                ["repo", "view", "--json", "nameWithOwner", "--jq", ".nameWithOwner"]
            ).strip()
            document = fetch(repo, args.branch, args.depth)
        except (RuntimeError, OSError) as error:
            # An API failure or a missing gh is not evidence of a bypass. Exit
            # with a distinct code so the caller can report it without filing
            # an issue.
            print(f"::error::Could not inspect {args.branch}: {error}")
            sys.stderr.write(f"Could not inspect {args.branch}: {error}\n")
            return 2

    now = parse_timestamp(args.now) if args.now else None
    if args.now and now is None:
        sys.stderr.write(f"Invalid --now timestamp: {args.now}\n")
        return 2

    report = evaluate(
        document,
        branch=args.branch,
        now=now,
        min_age=timedelta(minutes=args.min_age_minutes),
    )

    if args.output:
        Path(args.output).write_text(json.dumps(report.as_dict(), indent=2), encoding="utf-8")

    failed = bool(report.errors) or (args.strict and bool(report.warnings))

    if args.as_json:
        print(json.dumps(report.as_dict(), indent=2))
    else:
        emit_annotations(report)
        if failed:
            sys.stderr.write(report.render(strict=args.strict) + "\n")
        else:
            print(report.render(strict=args.strict))

    write_summary(report, args.strict)

    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
