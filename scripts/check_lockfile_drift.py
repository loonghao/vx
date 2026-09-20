#!/usr/bin/env python3
"""Report dependency versions a pull request would move backwards.

Why this exists
---------------
`Cargo.lock` in this repository resolves some dependency edges to one of
several versions of the same crate that are all present in the lockfile:
`workspace-hack` (cargo-hakari) pins both `windows-sys 0.52` and
`windows-sys 0.61` because different third-party crates genuinely require
each of them, and the same happens for `getrandom`. Crates whose own
requirement accepts either candidate (`colored`, `errno`, `is-terminal`,
`rustix`, `rustls-platform-verifier`, `tempfile`, `winapi-util`) are then
free edges, and the resolver may point them at either version.

That resolution is not stable. Re-running an identical
`cargo update -p <crate> --precise <version>` on an unchanged tree flips the
`windows-sys` edges between `0.52.0` and `0.61.2` on every pass, so no
lockfile state prevents a bump from also moving an unrelated edge to an
older version. Rebasing the branch does not help either: the drift is
present in branches that are already rebased onto the current `main`.

What can be checked is the outcome. This script compares the `Cargo.lock`
of the **real merge result** against the one on the base branch, using
`git merge-tree` so that a branch lagging behind the base does not report
the base's own progress as if the pull request had made it. Any edge whose
resolved version goes down is reported.

The same automation that produces that instability can also swallow a bump
entirely. `cargo hakari generate` runs in `Code Quality (via vx)`, its commit
is pushed back onto the pull request branch, and the re-resolution it
performs has been observed removing the very package the branch was opened
to bump: the lockfile ends up identical to the base, the pull request shows
a net change of zero files, and the bump it advertises was never delivered.
So a second check is applied when the title claims a bump: the claimed
version has to be present in the merged lockfile.

That check is scoped to titles that claim a bump, so a pull request that
changes no dependencies at all is not reported: there is nothing it claimed
and failed to deliver.

No version numbers are hardcoded here: the comparison is semver on whatever
the two lockfiles contain.

Exit codes
----------
0
    No dependency moved backwards, and every claimed bump was delivered.
1
    At least one dependency moved backwards, or (with `--strict`) at least
    one claimed bump was not delivered.
2
    The comparison could not be made (missing ref, unreadable lockfile,
    conflicting merge, ...).
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable, Sequence

DEFAULT_LOCKFILE = "Cargo.lock"
DEFAULT_BASE = "origin/main"


class LockfileError(RuntimeError):
    """Raised when a lockfile cannot be read or parsed."""


@dataclass(frozen=True)
class Edge:
    """One resolved dependency edge: `package version -> dependency version`."""

    package: str
    package_version: str
    dependency: str
    version: str

    def as_dict(self) -> dict[str, str]:
        """Return the report entry for this edge."""

        return {
            "package": self.package,
            "package_version": self.package_version,
            "dependency": self.dependency,
            "version": self.version,
        }


@dataclass(frozen=True)
class Drift:
    """A dependency edge that the head would move backwards."""

    package: str
    package_version: str
    dependency: str
    base_version: str
    head_version: str

    def describe(self) -> str:
        """Return the one-line human readable form."""

        return (
            f"{self.package} {self.package_version}: "
            f"{self.dependency} {self.base_version} -> {self.head_version}"
        )

    def as_dict(self) -> dict[str, str]:
        """Return the report entry for this drift."""

        return {
            "package": self.package,
            "package_version": self.package_version,
            "dependency": self.dependency,
            "base_version": self.base_version,
            "head_version": self.head_version,
            "direction": "downgrade",
        }


@dataclass(frozen=True)
class BumpClaim:
    """A crate bump a pull request title says it delivers."""

    crate: str
    version: str


@dataclass(frozen=True)
class UnfulfilledBump:
    """A claimed bump the merged lockfile does not contain."""

    crate: str
    claimed_version: str
    found_versions: list[str]

    def describe(self) -> str:
        """Return the one-line human readable form."""

        found = ", ".join(self.found_versions) if self.found_versions else "not in the lockfile"
        return f"{self.crate} {self.claimed_version} is not delivered (lockfile has: {found})"

    def as_dict(self) -> dict[str, object]:
        """Return the report entry for this missed bump."""

        return {
            "crate": self.crate,
            "claimed_version": self.claimed_version,
            "found_versions": self.found_versions,
        }


def lockfile_packages(parsed: dict[str, dict[str, str]]) -> list[tuple[str, str]]:
    """Return `(name, version)` for every package in a parsed lockfile."""

    return [tuple(package.split(" ", 1)) for package in parsed]


def parse_version(version: str) -> tuple[int, ...]:
    """Return a comparable key for a semver-ish version string.

    Only the numeric release part is compared; pre-release and build metadata
    are dropped, which is enough to order the versions cargo resolves.
    """

    core = version.split("+", 1)[0].split("-", 1)[0]
    parts: list[int] = []
    for chunk in core.split("."):
        digits = "".join(char for char in chunk if char.isdigit())
        parts.append(int(digits) if digits else 0)
    return tuple(parts) if parts else (0,)


def parse_lockfile(text: str) -> dict[str, dict[str, str]]:
    """Parse a `Cargo.lock` into `{package version: {dependency: version}}`.

    Lockfile v4 writes a dependency as `"name"` when the name is unambiguous
    and as `"name version"` when several versions of that crate exist, which
    is exactly the case this script cares about. Bare names are resolved
    against the versions declared in the same file.
    """

    versions: dict[str, list[str]] = {}
    raw_edges: dict[str, set[str]] = {}
    name: str | None = None
    current: str | None = None
    reading = False
    seen_package = False

    for line in text.splitlines():
        if line.startswith("[[package]]"):
            name, current, reading, seen_package = None, None, False, True
            continue

        match = _match(r'^name = "(.*)"$', line)
        if match and current is None and seen_package:
            name = match
            continue

        match = _match(r'^version = "(.*)"$', line)
        if match and name is not None and current is None:
            current = f"{name} {match}"
            versions.setdefault(name, []).append(match)
            raw_edges[current] = set()
            continue

        if line.startswith("dependencies = ["):
            reading = True
            continue

        if reading:
            if line.strip() == "]":
                reading = False
                continue
            match = _match(r'^\s+"(.*)",?$', line)
            if match and current is not None:
                raw_edges[current].add(match)

    resolved: dict[str, dict[str, str]] = {}
    for package, deps in raw_edges.items():
        edges: dict[str, str] = {}
        for dep in deps:
            if " " in dep:
                dep_name, _, dep_version = dep.partition(" ")
                edges[dep_name] = dep_version
                continue
            candidates = versions.get(dep, [])
            if len(candidates) == 1:
                edges[dep] = candidates[0]
        resolved[package] = edges
    return resolved


def _match(pattern: str, line: str) -> str | None:
    """Return the first capture group of `pattern` in `line`."""

    found = re.match(pattern, line)
    return found.group(1) if found else None


def find_drift(
    base: dict[str, dict[str, str]],
    head: dict[str, dict[str, str]],
) -> list[Drift]:
    """Return the edges `head` resolves to an older version than `base`."""

    drift: list[Drift] = []
    for package, base_edges in sorted(base.items()):
        head_edges = head.get(package)
        if not head_edges:
            continue

        package_name, _, package_version = package.partition(" ")
        for dependency, base_version in sorted(base_edges.items()):
            head_version = head_edges.get(dependency)
            if not head_version or head_version == base_version:
                continue
            if parse_version(head_version) < parse_version(base_version):
                drift.append(
                    Drift(
                        package=package_name,
                        package_version=package_version,
                        dependency=dependency,
                        base_version=base_version,
                        head_version=head_version,
                    )
                )
    return drift


# Renovate states the bump it is making in the pull request title. Only
# cargo-shaped claims are matched, so an Actions or Docker update is not held
# to a lockfile it does not touch.
BUMP_CLAIM = re.compile(
    r"^update\s+(?:rust\s+crate\s+)?(?P<crate>[\w.-]+?)"
    r"(?:\s+monorepo)?\s+to\s+v?(?P<version>[0-9][0-9A-Za-z.+-]*)",
    re.IGNORECASE,
)


def parse_bump_claims(title: str) -> list[BumpClaim]:
    """Return the crate bumps a pull request title claims to deliver.

    Returns nothing for a title that claims no bump, which is what keeps a
    pull request that legitimately touches no dependencies out of the report.
    """

    # Titles are conventional commits: `fix(deps): update rust crate x to 1`.
    _, _, description = title.partition(":")
    return [
        BumpClaim(crate=match["crate"], version=match["version"])
        for match in BUMP_CLAIM.finditer(description.strip())
    ]


def crate_matches(package: str, crate: str) -> bool:
    """Return whether a lockfile package belongs to a claimed crate.

    A monorepo claim covers the crates it publishes, so `serde` matches
    `serde_derive` and `zstd` matches `zstd-sys`.
    """

    return package == crate or package.startswith((f"{crate}_", f"{crate}-"))


def version_matches(actual: str, claimed: str) -> bool:
    """Return whether a resolved version is the claimed one.

    A claim may be shorter than the resolved version: `0.7` is delivered by
    `0.7.1`.
    """

    return actual == claimed or actual.startswith(f"{claimed}.")


def find_unfulfilled_bumps(
    claims: Sequence[BumpClaim],
    packages: Iterable[tuple[str, str]],
) -> list[UnfulfilledBump]:
    """Return the claimed bumps the merged lockfile does not contain."""

    packages = list(packages)
    unfulfilled: list[UnfulfilledBump] = []

    for claim in claims:
        found = sorted(
            {version for name, version in packages if crate_matches(name, claim.crate)}
        )
        if any(version_matches(version, claim.version) for version in found):
            continue
        unfulfilled.append(
            UnfulfilledBump(
                crate=claim.crate,
                claimed_version=claim.version,
                found_versions=found,
            )
        )
    return unfulfilled


def git(*args: str) -> str:
    """Run `git` and return its stdout."""

    result = subprocess.run(
        ["git", *args],
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        raise LockfileError(f"git {' '.join(args)} failed: {result.stderr.strip()}")
    return result.stdout


def merge_tree(base: str, head: str) -> str | None:
    """Return the tree of merging `head` into `base`, or None on conflict."""

    result = subprocess.run(
        ["git", "merge-tree", "--write-tree", base, head],
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        return None
    return result.stdout.strip() or None


def read_lockfile(ref: str, lockfile: str) -> str:
    """Return the contents of `lockfile` at `ref`."""

    if ref == "WORKTREE":
        path = Path(lockfile)
        if not path.is_file():
            raise LockfileError(f"{lockfile} not found in the working tree")
        return path.read_text(encoding="utf-8")
    return git("show", f"{ref}:{lockfile}")


def merged_lockfile(base: str, head: str, lockfile: str) -> tuple[str, str]:
    """Return the merged lockfile text, and what it was derived from."""

    tree = merge_tree(base, head)
    if tree:
        return git("show", f"{tree}:{lockfile}"), "merge-tree"

    # A conflict means the merge cannot be represented; comparing the head
    # alone would report the base's own progress as drift, so say so.
    return read_lockfile(head, lockfile), "head (merge conflict)"


def render_text(
    drift: Sequence[Drift],
    missed: Sequence[UnfulfilledBump],
    source: str,
    base: str,
    head: str,
) -> str:
    """Return the human readable report."""

    lines = [
        "Lockfile drift check",
        f"  base:  {base}",
        f"  head:  {head}",
        f"  merged from: {source}",
        "",
    ]

    if not drift:
        lines.append("No dependency moves backwards. ✅")
    else:
        lines.append(f"{len(drift)} dependency edge(s) move backwards: ❌")
        lines.append("")
        for item in drift:
            lines.append(f"  - {item.describe()}")
        lines.append("")
        lines.append(
            "These are unrelated to the bumped crate: the resolver re-points free\n"
            "edges onto another version of a crate that is already in the lockfile.\n"
            "Regenerating the lockfile can land on a resolution without them."
        )

    lines.append("")

    if not missed:
        lines.append("Every bump the title claims is present in the merged lockfile. ✅")
        return "\n".join(lines) + "\n"

    lines.append(f"{len(missed)} claimed bump(s) were not delivered: ⚠️")
    lines.append("")
    for item in missed:
        lines.append(f"  - {item.describe()}")
    lines.append("")
    lines.append(
        "The title advertises a bump the merge result does not contain, usually\n"
        "because a later commit on the branch re-resolved the lockfile and undid\n"
        "it. Check the branch commits before trusting the title."
    )
    return "\n".join(lines) + "\n"


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    """Parse the command line."""

    parser = argparse.ArgumentParser(
        description="Report dependency versions a pull request would move backwards."
    )
    parser.add_argument("--base", default=DEFAULT_BASE, help="Base ref (default: origin/main).")
    parser.add_argument("--head", default="HEAD", help="Head ref (default: HEAD).")
    parser.add_argument(
        "--lockfile",
        default=DEFAULT_LOCKFILE,
        help="Lockfile to compare (default: Cargo.lock).",
    )
    parser.add_argument(
        "--format",
        choices=("text", "json"),
        default="text",
        help="Report format.",
    )
    parser.add_argument(
        "--title",
        default="",
        help="Pull request title to check claimed bumps against.",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Fail on a claimed bump that was not delivered, not just warn.",
    )
    parser.add_argument(
        "--output",
        default="",
        help="Write the report to this path in addition to stdout.",
    )
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    """Compare the merged lockfile against the base and report on it."""

    args = parse_args(argv)

    try:
        base_text = read_lockfile(args.base, args.lockfile)
        head_text, source = merged_lockfile(args.base, args.head, args.lockfile)
    except LockfileError as error:
        print(f"::error::{error}")
        return 2

    try:
        base_parsed = parse_lockfile(base_text)
        head_parsed = parse_lockfile(head_text)
        drift = find_drift(base_parsed, head_parsed)
        missed = find_unfulfilled_bumps(
            parse_bump_claims(args.title), lockfile_packages(head_parsed)
        )
    except LockfileError as error:
        print(f"::error::{error}")
        return 2

    if args.format == "json":
        report: Any = json.dumps(
            {
                "base": args.base,
                "head": args.head,
                "merged_from": source,
                "drift": [item.as_dict() for item in drift],
                "unfulfilled_bumps": [item.as_dict() for item in missed],
                "clean": not drift and not missed,
            },
            indent=2,
        )
    else:
        report = render_text(drift, missed, source, args.base, args.head)

    print(report)

    if args.output:
        Path(args.output).write_text(report, encoding="utf-8")

    # A missed bump warns by default. Failing on it would block a merge over a
    # title that this parser misread, which is a worse outcome than a pull
    # request that simply did nothing; `--strict` opts into the failure.
    for item in missed:
        level = "error" if args.strict else "warning"
        print(f"::{level}::claimed bump not delivered: {item.describe()}")

    if drift:
        for item in drift:
            print(f"::error::dependency moved backwards: {item.describe()}")
        return 1

    return 1 if (missed and args.strict) else 0


if __name__ == "__main__":
    raise SystemExit(main())
