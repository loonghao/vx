"""Safety and evidence tests for the live legacy Python acceptance harness."""

import argparse
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import test_legacy_python as harness


class LegacyPythonHarnessTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="legacy-harness-test-")
        self.root = Path(self.temporary.name)
        with patch.object(
            harness,
            "existing_compiler_environment",
            side_effect=lambda env: (env, None),
        ):
            self.acceptance = harness.Acceptance(
                argparse.Namespace(
                    vx_binary=Path(sys.executable),
                    work_root=self.root,
                    receipt=None,
                    provider=self.root / "provider.star",
                    uv_version=harness.UV_VERSION,
                    command_timeout=10,
                )
            )

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def identity(self) -> dict:
        base = self.acceptance.vx_home / "store" / "python" / harness.PYTHON_VERSION
        return {
            "implementation": "cpython",
            "version": [3, 7, 9],
            "prefix": str(base),
            "base_prefix": str(base),
            "executable": str(base / "python.exe"),
        }

    def test_identity_refuses_substituted_implementation(self) -> None:
        data = self.identity()
        data["implementation"] = "pypy"
        with self.assertRaisesRegex(AssertionError, "Expected CPython"):
            self.acceptance.check_identity(data)

    def test_identity_refuses_substituted_version(self) -> None:
        data = self.identity()
        data["version"] = [3, 7, 17]
        with self.assertRaisesRegex(AssertionError, "version differs"):
            self.acceptance.check_identity(data)

    def test_identity_refuses_external_base_interpreter(self) -> None:
        data = self.identity()
        data["base_prefix"] = str(self.root / "external-python")
        with self.assertRaisesRegex(AssertionError, "escaped the owned VX store"):
            self.acceptance.check_identity(data)

    def test_identity_accepts_venv_entry_point(self) -> None:
        data = self.identity()
        venv = self.acceptance.directory / "venv"
        data.update(prefix=str(venv), executable=str(venv / "bin" / "python"))
        self.acceptance.check_identity(data, venv)

    def test_catalog_refuses_unverified_archive(self) -> None:
        self.acceptance.provider.write_text(
            '_LEGACY_37_ASSETS = {"linux/x64": {"sha256": None}}', encoding="utf-8"
        )
        with patch.object(harness, "platform_key", return_value="linux/x64"):
            with self.assertRaisesRegex(AssertionError, "trusted SHA256"):
                self.acceptance.contract()

    def test_catalog_reads_fixed_source_and_digest(self) -> None:
        self.acceptance.provider.write_text(
            '_LEGACY_37_ASSETS = {"linux/x64": {"sha256": "'
            + "a" * 64
            + '", "release": "historical", "asset": "fixed.tar.zst"}}',
            encoding="utf-8",
        )
        with patch.object(harness, "platform_key", return_value="linux/x64"):
            artifact = self.acceptance.contract()
        self.assertEqual(artifact["sha256"], "a" * 64)
        self.assertTrue(artifact["url"].endswith("/historical/fixed.tar.zst"))

    def test_reinstall_refuses_invalid_ownership_marker(self) -> None:
        store = self.acceptance.vx_home / "store" / "python" / harness.PYTHON_VERSION
        store.mkdir(parents=True)
        self.acceptance.marker.write_text("another-run", encoding="utf-8")
        with self.assertRaisesRegex(AssertionError, "ownership marker"):
            self.acceptance.cached_reinstall()
        self.assertTrue(
            store.exists(), "Guard must run before deleting the installation"
        )

    def test_failed_gate_blocks_only_its_dependents(self) -> None:
        def failure():
            raise AssertionError("expected acquisition failure")

        dependent = unittest.mock.Mock(return_value={})
        self.acceptance.gate("fresh", failure)
        self.acceptance.gate("dependent", dependent, ("fresh",))
        self.acceptance.gate("independent", lambda: {"checked": True})
        self.assertEqual(
            [gate["status"] for gate in self.acceptance.receipt["gates"]],
            ["failed", "blocked", "passed"],
        )
        dependent.assert_not_called()
        self.assertTrue(self.acceptance.receipt_path.is_file())


if __name__ == "__main__":
    unittest.main()
