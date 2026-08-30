#!/usr/bin/env python3
"""Validate that a pull request title is release-please compatible."""

from __future__ import annotations

import re
import sys


CONVENTIONAL_TITLE = re.compile(
    r"(?:feat|fix|perf|refactor|docs|style|test|build|ci|chore|revert)"
    r"(?:\([^()\r\n]+\))?!?: [^\s\r\n][^\r\n]*"
)

ERROR_MESSAGE = """Pull request title must use the Conventional Commit format.
Expected: <type>[optional scope][!]: <description>
Example: fix(windows): preserve nested PATH resolution
Allowed types: feat, fix, perf, refactor, docs, style, test, build, ci, chore, revert
"""


def is_valid_title(title: str) -> bool:
    """Return whether ``title`` can be parsed as a supported Conventional Commit."""

    return CONVENTIONAL_TITLE.fullmatch(title) is not None


def main(argv: list[str]) -> int:
    """Validate one title supplied on the command line."""

    if len(argv) != 2 or not is_valid_title(argv[1]):
        sys.stderr.write(ERROR_MESSAGE)
        return 1

    print("Pull request title is release-please compatible.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
