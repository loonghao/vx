"""Unit tests for the Renovate configuration checker."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

SCRIPTS_DIR = Path(__file__).resolve().parents[1]
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import check_renovate_config as crc  # noqa: E402


def schema(properties: dict, definitions: dict | None = None) -> dict:
    return {
        "properties": properties,
        "definitions": definitions or {},
    }


class DereferenceTests(unittest.TestCase):
    def test_follows_a_local_reference(self) -> None:
        doc = schema(
            {"recreateWhen": {"$ref": "#/definitions/recreateWhen"}},
            {"recreateWhen": {"enum": ["auto", "always", "never"]}},
        )

        resolved = crc.dereference(doc["properties"]["recreateWhen"], doc)

        self.assertEqual(resolved, {"enum": ["auto", "always", "never"]})

    def test_leaves_a_plain_node_alone(self) -> None:
        node = {"type": "string"}
        self.assertEqual(crc.dereference(node, schema({})), node)

    # A self-referential definition must not hang: the guard trips and the
    # node is returned unresolved, which callers treat as "no enumeration".
    def test_stops_instead_of_looping(self) -> None:
        doc = schema({}, {"loop": {"$ref": "#/definitions/loop"}})
        node = {"$ref": "#/definitions/loop"}

        resolved = crc.dereference(node, doc)

        self.assertEqual(resolved, node)
        self.assertIsNone(crc.accepted_values(node, doc))


class AcceptedValuesTests(unittest.TestCase):
    def test_reads_a_direct_enum(self) -> None:
        doc = schema({"a": {"enum": ["x", "y"]}})
        self.assertEqual(crc.accepted_values(doc["properties"]["a"], doc), ["x", "y"])

    def test_reads_an_enum_behind_a_reference(self) -> None:
        doc = schema(
            {"a": {"$ref": "#/definitions/a"}},
            {"a": {"enum": ["x", "y"]}},
        )
        self.assertEqual(crc.accepted_values(doc["properties"]["a"], doc), ["x", "y"])

    def test_merges_branches_of_oneOf(self) -> None:
        doc = schema({"a": {"oneOf": [{"enum": ["x"]}, {"enum": ["y"]}]}})
        self.assertEqual(sorted(crc.accepted_values(doc["properties"]["a"], doc)), ["x", "y"])

    def test_returns_none_when_values_are_not_enumerated(self) -> None:
        doc = schema({"a": {"type": "array"}})
        self.assertIsNone(crc.accepted_values(doc["properties"]["a"], doc))


class FindProblemsTests(unittest.TestCase):
    def build(self) -> dict:
        return schema(
            {
                "recreateWhen": {
                    "$ref": "#/definitions/recreateWhen",
                },
                "extends": {"type": "array"},
            },
            {"recreateWhen": {"enum": ["auto", "always", "never"]}},
        )

    def test_an_accepted_value_is_not_reported(self) -> None:
        doc = self.build()
        self.assertEqual(crc.find_problems({"recreateWhen": "always"}, doc), [])

    # The gap this closes: the official validator accepts any string here.
    def test_a_value_outside_the_enumeration_is_reported(self) -> None:
        doc = self.build()

        problems = crc.find_problems({"recreateWhen": "alwyas"}, doc)

        self.assertEqual(len(problems), 1)
        self.assertIn("recreateWhen", problems[0])
        self.assertIn("alwyas", problems[0])

    def test_an_unknown_option_is_reported(self) -> None:
        doc = self.build()
        self.assertEqual(
            crc.find_problems({"recreateWhenn": "always"}, doc),
            ["unknown option `recreateWhenn`"],
        )

    def test_the_schema_key_is_ignored(self) -> None:
        doc = self.build()
        self.assertEqual(crc.find_problems({"$schema": "https://example.test"}, doc), [])

    def test_an_option_without_enumerated_values_is_not_value_checked(self) -> None:
        doc = self.build()
        self.assertEqual(crc.find_problems({"extends": ["config:recommended"]}, doc), [])

    def test_every_problem_is_reported(self) -> None:
        doc = self.build()

        problems = crc.find_problems(
            {"recreateWhen": "nope", "rebaseWhen": "also-nope", "extends": ["x"]}, doc
        )

        self.assertEqual(len(problems), 2)


if __name__ == "__main__":
    unittest.main()
