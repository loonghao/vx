#!/usr/bin/env python3
"""Validate metadata and changed files for a Release Please pull request."""

from __future__ import annotations

import argparse
import sys


EXPECTED_BASE_REF = "main"
EXPECTED_HEAD_PREFIX = "release-please--"
EXPECTED_AUTHORS = frozenset(("github-actions[bot]", "release-please[bot]"))
REQUIRED_FILES = frozenset(
    (
        ".release-please-manifest.json",
        "CHANGELOG.md",
        "Cargo.toml",
    )
)
OPTIONAL_FILES = frozenset(("Cargo.lock",))


def validation_errors(
    *, head_ref: str, base_ref: str, author: str, files: list[str]
) -> list[str]:
    """Return contract violations for a proposed Release Please pull request."""

    errors: list[str] = []
    changed_files = set(files)

    if not head_ref.startswith(EXPECTED_HEAD_PREFIX):
        errors.append(f"unexpected head branch: {head_ref}")
    if base_ref != EXPECTED_BASE_REF:
        errors.append(f"unexpected base branch: {base_ref}")
    if author not in EXPECTED_AUTHORS:
        errors.append(f"unexpected author: {author}")

    missing = sorted(REQUIRED_FILES - changed_files)
    if missing:
        errors.append(f"missing required files: {', '.join(missing)}")

    unexpected = sorted(changed_files - REQUIRED_FILES - OPTIONAL_FILES)
    if unexpected:
        errors.append(f"unexpected changed files: {', '.join(unexpected)}")

    return errors


def parse_args(argv: list[str]) -> argparse.Namespace:
    """Parse command-line arguments."""

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--head-ref", required=True)
    parser.add_argument("--base-ref", required=True)
    parser.add_argument("--author", required=True)
    parser.add_argument("files", nargs="+")
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    """Validate one Release Please pull request."""

    arguments = parse_args(argv)
    errors = validation_errors(
        head_ref=arguments.head_ref,
        base_ref=arguments.base_ref,
        author=arguments.author,
        files=arguments.files,
    )
    if errors:
        for error in errors:
            print(f"Release PR validation failed: {error}", file=sys.stderr)
        return 1

    print("Release Please pull request metadata and files are valid.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
