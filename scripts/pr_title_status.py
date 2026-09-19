#!/usr/bin/env python3
"""Publish the ``PR Title`` required status check on pull request heads.

Why this script exists
----------------------
``PR Title`` is a required status check on ``main``. It used to be published by
a single ``pull_request_target`` run, which works for a head pushed by a human
or by Renovate: the ``synchronize`` event arrives and the run reports on the
new head.

It does not work for a head pushed with the repository ``GITHUB_TOKEN``. The
``Code Quality (via vx)`` job regenerates ``workspace-hack`` with
``cargo hakari`` and pushes the result with that token. GitHub starts the
plain ``pull_request`` workflows for such a push, but not
``pull_request_target``, so the one workflow that owns the required status is
the one workflow that never sees the new head. The head then carries no
``PR Title`` status at all, the required check can never be satisfied, and the
pull request stays ``BLOCKED`` with fully green CI.

This script is the shared implementation for every path that can observe a
head which arrived without a ``pull_request_target`` event:

``pull_request_target``
    The interactive path. Unchanged: validate and publish on every event.

``workflow_run``
    Runs after ``CI`` completes for the head. ``CI`` is a plain
    ``pull_request`` workflow, so it does run for a token push.

``schedule``
    Hourly sweep over the open pull requests. It is what guarantees the status
    can never stay missing, whatever the event plumbing does next.

``workflow_dispatch``
    Manual backfill, optionally narrowed to one pull request number.

Every path executes from a trusted checkout (the base commit for
``pull_request_target``, the default branch otherwise) and reads pull request
metadata through the API. Pull request code is never checked out and never
executed.

Exit codes
----------
0
    Every evaluated title is valid (or there was nothing to evaluate).
1
    At least one evaluated title is invalid. The failure status is published,
    so the pull request is already blocked by the required check.
2
    The statuses could not be published at all: nothing was evaluated, so
    nothing may be treated as green.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from dataclasses import dataclass
from typing import Any, Callable, Iterable, Sequence
from urllib.parse import urlencode

from validate_pr_title import is_valid_title

STATUS_CONTEXT = "PR Title"

VALID_DESCRIPTION = "Conventional Commit title is valid"
INVALID_DESCRIPTION = "Use a Conventional Commit title"

EXIT_VALID = 0
EXIT_INVALID = 1
EXIT_UNAVAILABLE = 2


class GhError(RuntimeError):
    """Raised when a ``gh api`` call fails."""


GhRunner = Callable[[Sequence[str]], str]


def gh_runner(args: Sequence[str]) -> str:
    """Run ``gh api`` with the ambient ``GH_TOKEN`` and return its stdout."""

    result = subprocess.run(
        ["gh", "api", *args],
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        raise GhError(f"gh {' '.join(args)} failed: {result.stderr.strip()}")
    return result.stdout


@dataclass
class Evaluation:
    """The verdict for one pull request head."""

    number: int
    head_sha: str
    title: str
    valid: bool
    published: bool = False

    @property
    def state(self) -> str:
        """Return the commit status state for this verdict."""

        return "success" if self.valid else "failure"

    def as_dict(self) -> dict[str, Any]:
        """Return the report entry for this verdict."""

        return {
            "number": self.number,
            "head_sha": self.head_sha,
            "title": self.title,
            "valid": self.valid,
            "state": self.state,
            "published": self.published,
        }


class GitHub:
    """The small slice of the GitHub API this script needs."""

    def __init__(self, repository: str, runner: GhRunner = gh_runner) -> None:
        self.repository = repository
        self._runner = runner

    @property
    def owner(self) -> str:
        """Return the repository owner, used as the fallback head owner."""

        return self.repository.split("/", 1)[0]

    def json(self, *args: str) -> Any:
        """Run a read-only ``gh api`` call and decode its JSON body."""

        return json.loads(self._runner(list(args)))

    def pull(self, number: str) -> dict[str, Any]:
        """Return one pull request by number."""

        return self.json(f"repos/{self.repository}/pulls/{number}")

    def open_pulls(self, head: str | None = None) -> list[dict[str, Any]]:
        """Return the open pull requests, optionally filtered by ``head``.

        ``head`` uses the ``owner:branch`` form of the pulls API.
        """

        query = {"state": "open", "per_page": "100"}
        if head:
            query["head"] = head
        # `safe` keeps the `owner:branch` separator readable; the colon is
        # part of the API's filter syntax, not a separator to be escaped.
        return self.json(f"repos/{self.repository}/pulls?{urlencode(query, safe=':')}")

    def statuses(self, sha: str) -> list[dict[str, Any]]:
        """Return the commit statuses already published on ``sha``."""

        payload = self.json(f"repos/{self.repository}/commits/{sha}/status")
        return list(payload.get("statuses") or [])

    def publish_status(
        self,
        sha: str,
        state: str,
        description: str,
        target_url: str,
    ) -> None:
        """Publish the ``PR Title`` status on ``sha``."""

        self._runner(
            [
                "--method",
                "POST",
                f"repos/{self.repository}/statuses/{sha}",
                "--raw-field",
                f"state={state}",
                "--raw-field",
                f"context={STATUS_CONTEXT}",
                "--raw-field",
                f"description={description}",
                "--raw-field",
                f"target_url={target_url}",
            ]
        )


def needs_status(statuses: Iterable[dict[str, Any]], context: str = STATUS_CONTEXT) -> bool:
    """Return whether ``context`` is absent from a commit's status list."""

    return not any(status.get("context") == context for status in statuses)


def pulls_without_status(
    pulls: Iterable[dict[str, Any]],
    statuses_by_sha: dict[str, list[dict[str, Any]]],
    context: str = STATUS_CONTEXT,
) -> list[dict[str, Any]]:
    """Return the pull requests whose head carries no ``context`` status."""

    return [
        pull
        for pull in pulls
        if needs_status(statuses_by_sha.get(pull["head"]["sha"], []), context)
    ]


def backfill_targets(
    github: GitHub,
    pulls: Iterable[dict[str, Any]],
    context: str = STATUS_CONTEXT,
) -> list[dict[str, Any]]:
    """Return the pull requests in ``pulls`` that still need a status.

    A backfill must not overwrite a verdict published by another run, so only
    heads with no ``context`` status at all are selected.
    """

    pulls = list(pulls)
    statuses_by_sha = {pull["head"]["sha"]: github.statuses(pull["head"]["sha"]) for pull in pulls}
    return pulls_without_status(pulls, statuses_by_sha, context)


def head_filter_for_workflow_run(event: dict[str, Any], default_owner: str) -> str | None:
    """Return the ``owner:branch`` filter for a ``workflow_run`` event.

    The run reports the branch it ran on. Returning ``None`` means the event
    carries no branch (a push to the default branch, for example), so there is
    no pull request head to look up.
    """

    run = event.get("workflow_run") or {}
    branch = run.get("head_branch")
    if not branch:
        return None

    repository = run.get("head_repository") or {}
    owner = (repository.get("owner") or {}).get("login") or default_owner
    return f"{owner}:{branch}"


def resolve_targets(
    github: GitHub,
    event_name: str,
    event: dict[str, Any],
    requested_pr: str = "",
    context: str = STATUS_CONTEXT,
) -> list[dict[str, Any]]:
    """Return the pull requests that need a ``context`` status published."""

    if requested_pr:
        # An explicit request must be able to replace a stale verdict, so it
        # bypasses the backfill filter.
        return [github.pull(requested_pr)]

    if event_name == "pull_request_target":
        pull = event.get("pull_request")
        return [pull] if pull else []

    if event_name == "workflow_run":
        head = head_filter_for_workflow_run(event, github.owner)
        if not head:
            return []
        return backfill_targets(github, github.open_pulls(head), context)

    return backfill_targets(github, github.open_pulls(), context)


def evaluate(
    github: GitHub,
    pull: dict[str, Any],
    target_url: str,
    dry_run: bool = False,
) -> Evaluation:
    """Validate one pull request title and publish the status on its head."""

    title = pull.get("title") or ""
    result = Evaluation(
        number=int(pull.get("number") or 0),
        head_sha=pull["head"]["sha"],
        title=title,
        valid=is_valid_title(title),
    )

    if not dry_run:
        github.publish_status(
            result.head_sha,
            result.state,
            VALID_DESCRIPTION if result.valid else INVALID_DESCRIPTION,
            target_url,
        )
        result.published = True

    return result


def render_summary(event_name: str, results: Sequence[Evaluation]) -> str:
    """Return the Markdown job summary for one run."""

    lines = [
        "## PR Title status",
        "",
        f"- event: `{event_name}`",
        f"- evaluated: {len(results)}",
        "",
    ]

    if not results:
        lines.append("Nothing to do: every candidate head already reports this status.")
        return "\n".join(lines) + "\n"

    lines.append("| Pull request | Head | Title | State |")
    lines.append("| --- | --- | --- | --- |")
    for result in results:
        marker = "✅" if result.valid else "❌"
        lines.append(
            f"| #{result.number} | `{result.head_sha[:8]}` | {result.title} | {marker} {result.state} |"
        )
    return "\n".join(lines) + "\n"


def render_report(event_name: str, results: Sequence[Evaluation]) -> dict[str, Any]:
    """Return the JSON report for one run."""

    return {
        "context": STATUS_CONTEXT,
        "event": event_name,
        "valid": all(result.valid for result in results),
        "evaluated": [result.as_dict() for result in results],
    }


def write_report(path: str, event_name: str, results: Sequence[Evaluation]) -> None:
    """Write the JSON report, when a path was requested."""

    if not path:
        return

    with open(path, "w", encoding="utf-8") as handle:
        json.dump(render_report(event_name, results), handle, indent=2)
        handle.write("\n")


def write_summary(event_name: str, results: Sequence[Evaluation]) -> None:
    """Append the Markdown summary to the GitHub step summary, when there is one."""

    summary_path = os.environ.get("GITHUB_STEP_SUMMARY")
    if not summary_path:
        return

    with open(summary_path, "a", encoding="utf-8") as handle:
        handle.write(render_summary(event_name, results))


def load_event(path: str) -> dict[str, Any]:
    """Return the webhook payload, or an empty dict when there is none."""

    if not path or not os.path.isfile(path):
        return {}

    with open(path, encoding="utf-8") as handle:
        payload = json.load(handle)

    return payload if isinstance(payload, dict) else {}


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    """Parse the command line."""

    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--event-name",
        default=os.environ.get("GITHUB_EVENT_NAME", ""),
        help="GitHub event that started the run.",
    )
    parser.add_argument(
        "--event-path",
        default=os.environ.get("GITHUB_EVENT_PATH", ""),
        help="Path to the webhook payload.",
    )
    parser.add_argument(
        "--repository",
        default=os.environ.get("GITHUB_REPOSITORY", ""),
        help="`owner/repo` to publish statuses on.",
    )
    parser.add_argument(
        "--pr",
        default="",
        help="Validate only this pull request number, even if it already has a status.",
    )
    parser.add_argument(
        "--target-url",
        default="",
        help="URL the published status links to.",
    )
    parser.add_argument(
        "--report",
        default="",
        help="Write a JSON report to this path.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Evaluate without publishing any status.",
    )
    return parser.parse_args(argv)


def run(
    github: GitHub,
    event_name: str,
    event: dict[str, Any],
    target_url: str,
    requested_pr: str = "",
    report: str = "",
    dry_run: bool = False,
) -> int:
    """Evaluate every target and publish its status; return the exit code.

    Separated from :func:`main` so the pull request resolution can be driven
    without a GitHub token.
    """

    try:
        targets = resolve_targets(github, event_name, event, requested_pr)
        results = [evaluate(github, pull, target_url, dry_run) for pull in targets]
    except GhError as error:
        print(f"::error::PR Title status could not be published: {error}")
        return EXIT_UNAVAILABLE

    write_report(report, event_name, results)
    write_summary(event_name, results)

    for result in results:
        if not result.valid:
            print(
                f"::error::Pull request #{result.number} title is not a Conventional Commit: {result.title}"
            )

    invalid = sum(1 for result in results if not result.valid)
    print(
        f"PR Title status: {len(results) - invalid} valid, {invalid} invalid, "
        f"of {len(results)} evaluated."
    )
    return EXIT_INVALID if invalid else EXIT_VALID


def main(argv: Sequence[str] | None = None) -> int:
    """Validate pull request titles and publish the required status."""

    args = parse_args(argv)

    if not args.event_name:
        print("::error::--event-name is required (it is unset for a local run).")
        return EXIT_UNAVAILABLE

    if not args.repository:
        print("::error::--repository is required (it is unset for a local run).")
        return EXIT_UNAVAILABLE

    github = GitHub(args.repository)
    event = load_event(args.event_path)

    return run(
        github,
        args.event_name,
        event,
        args.target_url,
        requested_pr=args.pr,
        report=args.report,
        dry_run=args.dry_run,
    )


if __name__ == "__main__":
    raise SystemExit(main())
