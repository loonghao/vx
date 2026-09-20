#!/usr/bin/env python3
"""Check Renovate configuration option names and values against the schema.

`renovate-config-validator` parses and type-checks the configuration, but it
does not compare string values against the set the option accepts: setting
`recreateWhen` to `"whenever"` validates cleanly and exits 0. Names and values
are the easiest thing to typo, and a value Renovate does not recognise is
silently ignored rather than reported, so they are checked here.

Only top-level options are checked. `packageRules` entries are validated by
the official validator, which catches structural and type errors inside them.

Exit codes
----------
0
    Every option name is documented and every value is accepted.
1
    At least one unknown option or unaccepted value.
2
    The check could not be made (schema unavailable, unreadable config).
"""

from __future__ import annotations

import json
import sys
import urllib.request
from typing import Any
from urllib.error import URLError

DEFAULT_SCHEMA_URL = "https://docs.renovatebot.com/renovate-schema.json"
DEFAULT_CONFIGS = (
    "renovate.json",
    "renovate.json5",
    ".renovaterc",
    ".renovaterc.json",
    ".renovaterc.json5",
    ".github/renovate.json",
    ".github/renovate.json5",
)


class ConfigError(RuntimeError):
    """Raised when the check cannot be made."""


def load_schema(url: str) -> dict[str, Any]:
    """Fetch the published Renovate schema."""

    try:
        with urllib.request.urlopen(url, timeout=60) as response:
            schema = json.load(response)
    except (URLError, TimeoutError, ValueError) as error:
        raise ConfigError(f"could not read the schema at {url}: {error}") from error
    return schema


def dereference(node: Any, schema: dict[str, Any], depth: int = 0) -> Any:
    """Follow local `$ref` pointers into the schema's definitions."""

    if depth > 10 or not isinstance(node, dict):
        return node

    ref = node.get("$ref")
    if isinstance(ref, str) and ref.startswith("#/definitions/"):
        definitions = schema.get("definitions", {})
        return dereference(definitions.get(ref.split("/")[-1], {}), schema, depth + 1)
    return node


def accepted_values(node: Any, schema: dict[str, Any]) -> list[str] | None:
    """Return the values an option accepts, or None if it is not enumerated."""

    node = dereference(node, schema)
    if not isinstance(node, dict):
        return None

    if node.get("enum"):
        return list(node["enum"])

    values: list[str] = []
    for key in ("oneOf", "anyOf", "allOf"):
        for branch in node.get(key) or []:
            branch = dereference(branch, schema)
            if isinstance(branch, dict) and branch.get("enum"):
                values.extend(branch["enum"])
    return values or None


def find_problems(config: dict[str, Any], schema: dict[str, Any]) -> list[str]:
    """Return the unknown options and unaccepted values in one config."""

    properties = schema.get("properties", {})
    problems: list[str] = []

    for key in sorted(config):
        if key == "$schema":
            continue
        if key not in properties:
            problems.append(f"unknown option `{key}`")
            continue

        accepted = accepted_values(properties[key], schema)
        value = config[key]
        if accepted and isinstance(value, str) and value not in accepted:
            problems.append(
                f"`{key}` = {value!r} is not one of {sorted(set(accepted))}"
            )

    return problems


def main(argv: list[str] | None = None) -> int:
    """Check every Renovate configuration file present."""

    args = argv if argv is not None else sys.argv[1:]
    schema_url = DEFAULT_SCHEMA_URL
    paths = [arg for arg in args if not arg.startswith("--")]

    for arg in args:
        if arg.startswith("--schema="):
            schema_url = arg.split("=", 1)[1]

    try:
        schema = load_schema(schema_url)
    except ConfigError as error:
        print(f"::error::{error}")
        return 2

    print(f"schema: {schema_url}")
    print(f"documented options: {len(schema.get('properties', {}))}")

    candidates = paths or [
        path
        for path in DEFAULT_CONFIGS
        if _is_file(path)
    ]

    if not candidates:
        print("no renovate configuration file present; nothing to check")
        return 0

    problems: list[str] = []
    for path in candidates:
        try:
            with open(path, encoding="utf-8") as handle:
                config = json.load(handle)
        except (OSError, ValueError) as error:
            print(f"::error::{path}: could not be read as JSON: {error}")
            return 2

        print(f"checking {path}")
        problems.extend(f"{path}: {problem}" for problem in find_problems(config, schema))

    if problems:
        for problem in problems:
            print(f"::error::{problem}")
        return 1

    print("option names and values are consistent with the schema")
    return 0


def _is_file(path: str) -> bool:
    from pathlib import Path

    return Path(path).is_file()


if __name__ == "__main__":
    raise SystemExit(main())
