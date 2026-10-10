#!/usr/bin/env python3
"""Accept a built VX against the fixed Python 3.7 lifecycle contract.

Run with a modern harness interpreter, for example::

    vx python@3.12 scripts/test_legacy_python.py --vx-binary target/debug/vx.exe

Every run creates its own empty VX_HOME and UV_CACHE_DIR below --work-root.
Only its Python store directory is removed, for the archive-cache reinstall
gate. The run directory, command logs and receipt are retained for inspection.
This is a live acceptance check, separate from mocked unit tests.
"""

from __future__ import annotations

import argparse
import ast
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import time
from typing import Any, Callable


ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests" / "fixtures" / "legacy-python-build"
PYTHON_VERSION = "3.7.9"
UV_VERSION = "0.12.23"
# These last compatible backend releases are explicit so acceptance does not
# depend on a resolver selecting newer packages that dropped Python 3.7.
BUILD_REQUIREMENTS = ["setuptools==67.8.0", "wheel==0.41.3", "build==1.0.3"]
IDENTITY_CODE = """
import json, ssl, sys, sysconfig
print(json.dumps({
    'implementation': sys.implementation.name,
    'version': list(sys.version_info[:3]),
    'executable': sys.executable,
    'prefix': sys.prefix,
    'base_prefix': sys.base_prefix,
    'include': sysconfig.get_path('include'),
    'openssl': ssl.OPENSSL_VERSION,
}))
"""
WHEEL_CODE = """
import json, sys
import vx_legacy_probe
print(json.dumps({
    'version': list(sys.version_info[:3]),
    'prefix': sys.prefix,
    'base_prefix': sys.base_prefix,
    'module': vx_legacy_probe.__file__,
    'native_versions': vx_legacy_probe.versions(),
    'build': vx_legacy_probe.build_provenance(),
}))
"""


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def within(path: Path, directory: Path) -> bool:
    return path.resolve().is_relative_to(directory.resolve())


def platform_key() -> str:
    operating_system = {"Windows": "windows", "Linux": "linux", "Darwin": "macos"}
    architecture = {
        "AMD64": "x64",
        "x86_64": "x64",
        "aarch64": "arm64",
        "arm64": "arm64",
    }
    return "{}/{}".format(
        operating_system.get(platform.system(), platform.system().lower()),
        architecture.get(platform.machine(), platform.machine()),
    )


def artifact_contract(provider: Path) -> dict[str, Any]:
    """Read the checked-in literal lifecycle catalog without executing the DSL."""
    tree = ast.parse(provider.read_text(encoding="utf-8"), filename=str(provider))
    for node in tree.body:
        if isinstance(node, ast.Assign) and any(
            isinstance(target, ast.Name) and target.id == "_LEGACY_37_ASSETS"
            for target in node.targets
        ):
            catalog = ast.literal_eval(node.value)
            artifact = dict(catalog[platform_key()])
            require(
                re.fullmatch(r"[0-9a-fA-F]{64}", artifact.get("sha256") or "")
                is not None,
                "The lifecycle catalog must contain a trusted SHA256 before acceptance",
            )
            artifact["sha256"] = artifact["sha256"].lower()
            artifact["url"] = artifact.get("url") or (
                "https://github.com/astral-sh/python-build-standalone/releases/download/"
                + artifact["release"]
                + "/"
                + artifact["asset"]
            )
            return artifact
    raise AssertionError("Python provider has no literal _LEGACY_37_ASSETS catalog")


def existing_compiler_environment(
    environment: dict[str, str],
) -> tuple[dict[str, str], str | None]:
    """Bind an already installed VS compiler when the invoking shell did not.

    vswhere and vcvars64 come from the existing Visual Studio installation.
    This probe neither downloads a compiler nor changes persistent environment.
    """
    if os.name != "nt" or shutil.which("cl", path=environment.get("PATH")):
        return environment, None
    installer = Path(environment.get("ProgramFiles(x86)", "C:/Program Files (x86)"))
    vswhere = installer / "Microsoft Visual Studio" / "Installer" / "vswhere.exe"
    if not vswhere.is_file():
        return environment, None
    location = subprocess.run(
        [
            str(vswhere),
            "-latest",
            "-products",
            "*",
            "-requires",
            "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
            "-property",
            "installationPath",
        ],
        env=environment,
        capture_output=True,
        text=True,
        timeout=30,
        check=False,
    )
    if location.returncode or not location.stdout.strip():
        return environment, None
    vcvars = (
        Path(location.stdout.strip()) / "VC" / "Auxiliary" / "Build" / "vcvars64.bat"
    )
    if not vcvars.is_file():
        return environment, None
    # Pass cmd its original command line: list2cmdline escapes the batch file's
    # nested quotes, which cmd interprets as a UNC path instead of a script.
    command = '"{}" /d /s /c ""{}" >nul && set"'.format(
        environment.get("COMSPEC", "cmd.exe"), vcvars
    )
    bound = subprocess.run(
        command,
        env=environment,
        capture_output=True,
        text=True,
        errors="replace",
        timeout=60,
        check=False,
    )
    if bound.returncode:
        raise AssertionError(
            "Existing MSVC environment failed: " + bound.stderr[-1000:]
        )
    result = dict(environment)
    for line in bound.stdout.splitlines():
        if "=" in line and not line.startswith("="):
            name, value = line.split("=", 1)
            # Windows names are case-insensitive; avoid retaining both Path/PATH.
            for previous in list(result):
                if previous.casefold() == name.casefold():
                    del result[previous]
            result[name.upper() if name.casefold() == "path" else name] = value
    result.update(DISTUTILS_USE_SDK="1", MSSdk="1")
    return result, str(vcvars)


class Acceptance:
    def __init__(self, arguments: argparse.Namespace) -> None:
        self.vx = arguments.vx_binary.resolve(strict=True)
        work_root = arguments.work_root.resolve()
        work_root.mkdir(parents=True, exist_ok=True)
        self.directory = Path(tempfile.mkdtemp(prefix="run-", dir=work_root)).resolve()
        self.marker = self.directory / ".vx-legacy-acceptance-owner"
        self.marker.write_text(str(self.directory), encoding="utf-8")
        self.vx_home = self.directory / "vx-home"
        self.uv_cache = self.directory / "uv-cache"
        self.logs = self.directory / "logs"
        self.logs.mkdir()
        self.cwd = self.directory / "project"
        self.cwd.mkdir()
        self.uv = "uv@" + arguments.uv_version
        self.timeout = arguments.command_timeout
        self.receipt_path = (
            arguments.receipt or self.directory / "receipt.json"
        ).resolve()
        self.environment = dict(os.environ)
        for name in list(self.environment):
            if (
                name.startswith(("UV_", "VX_", "PYTHON", "PIP_"))
                or name == "VIRTUAL_ENV"
            ):
                del self.environment[name]
        self.environment.update(
            {
                "VX_HOME": str(self.vx_home),
                "UV_CACHE_DIR": str(self.uv_cache),
                "UV_PYTHON_DOWNLOADS": "never",
                "UV_NO_CONFIG": "1",
                "UV_LINK_MODE": "copy",
                "PYTHONNOUSERSITE": "1",
                "RUST_LOG": "vx_runtime_http=debug,vx_runtime=info",
                "NO_COLOR": "1",
            }
        )
        self.environment, compiler_source = existing_compiler_environment(
            self.environment
        )
        self.provider = arguments.provider.resolve()
        self.state: dict[str, Any] = {}
        self.receipt: dict[str, Any] = {
            "schema_version": 1,
            "started_at": datetime.now(timezone.utc).isoformat(),
            "platform": platform_key(),
            "host": platform.platform(),
            "python_request": PYTHON_VERSION,
            "uv_request": arguments.uv_version,
            "vx_binary": str(self.vx),
            "vx_binary_sha256": sha256(self.vx),
            "source_revision": getattr(arguments, "source_revision", None),
            "run_directory": str(self.directory),
            "vx_home": str(self.vx_home),
            "uv_cache": str(self.uv_cache),
            "compiler": {
                "executable": shutil.which(
                    "cl" if os.name == "nt" else "cc", path=self.environment.get("PATH")
                ),
                "environment_source": compiler_source,
                "CC": self.environment.get("CC"),
                "DISTUTILS_USE_SDK": self.environment.get("DISTUTILS_USE_SDK"),
            },
            "gates": [],
            "commands": [],
        }

    def save(self) -> None:
        self.receipt_path.parent.mkdir(parents=True, exist_ok=True)
        data = json.dumps(self.receipt, indent=2) + "\n"
        self.receipt_path.write_text(data, encoding="utf-8")
        local_receipt = self.directory / "receipt.json"
        if local_receipt != self.receipt_path:
            local_receipt.write_text(data, encoding="utf-8")

    def run(self, label: str, *arguments: str, offline: bool = False) -> str:
        command = [str(self.vx), *arguments]
        if offline and arguments and arguments[0] == self.uv:
            command.insert(2, "--offline")
        environment = dict(self.environment)
        if offline:
            environment.update(
                {
                    "UV_OFFLINE": "1",
                    "HTTP_PROXY": "http://127.0.0.1:9",
                    "HTTPS_PROXY": "http://127.0.0.1:9",
                    "ALL_PROXY": "http://127.0.0.1:9",
                    "http_proxy": "http://127.0.0.1:9",
                    "https_proxy": "http://127.0.0.1:9",
                    "all_proxy": "http://127.0.0.1:9",
                    "NO_PROXY": "",
                    "no_proxy": "",
                }
            )
        log = self.logs / ("{:02d}-{}.log".format(len(self.receipt["commands"]), label))
        started = time.monotonic()
        record: dict[str, Any] = {"argv": command, "log": str(log), "offline": offline}
        self.receipt["commands"].append(record)
        self.save()
        try:
            with log.open("w", encoding="utf-8") as stream:
                completed = subprocess.run(
                    command,
                    cwd=self.cwd,
                    env=environment,
                    stdout=stream,
                    stderr=subprocess.STDOUT,
                    timeout=self.timeout,
                    check=False,
                )
            record["returncode"] = completed.returncode
        except subprocess.TimeoutExpired:
            record["returncode"] = None
            record["timed_out"] = True
        output = log.read_text(encoding="utf-8", errors="replace")
        record["duration_seconds"] = round(time.monotonic() - started, 3)
        self.save()
        require(
            record["returncode"] == 0,
            "{} failed; inspect {}\n{}".format(label, log, output[-2500:]),
        )
        return output

    def gate(
        self,
        name: str,
        action: Callable[[], dict[str, Any]],
        needs: tuple[str, ...] = (),
    ) -> None:
        started = time.monotonic()
        previous = {gate["name"]: gate["status"] for gate in self.receipt["gates"]}
        missing = [
            dependency for dependency in needs if previous.get(dependency) != "passed"
        ]
        record: dict[str, Any] = {"name": name}
        if missing:
            record.update(
                status="blocked", reason="Failed prerequisite: " + ", ".join(missing)
            )
        else:
            try:
                record.update(status="passed", evidence=action())
            except (AssertionError, OSError, ValueError, KeyError) as error:
                record.update(status="failed", reason=str(error))
        record["duration_seconds"] = round(time.monotonic() - started, 3)
        self.receipt["gates"].append(record)
        self.save()
        print("{}: {}".format(name, record["status"]), flush=True)

    def python(
        self, label: str, code: str, venv: Path | None = None, offline: bool = False
    ) -> dict[str, Any]:
        if venv is None:
            arguments = ["python@" + PYTHON_VERSION, "-I", "-c", code]
        else:
            arguments = [
                self.uv,
                "run",
                "--no-project",
                "--no-sync",
                "--no-python-downloads",
                "--python",
                str(self.venv_python(venv)),
                "python",
                "-I",
                "-c",
                code,
            ]
        output = self.run(label, *arguments, offline=offline)
        for line in reversed(output.splitlines()):
            if line.startswith("{"):
                return json.loads(line)
        raise AssertionError("Python command did not emit JSON: " + output[-1000:])

    @staticmethod
    def venv_python(venv: Path) -> Path:
        return venv / ("Scripts/python.exe" if os.name == "nt" else "bin/python")

    def check_identity(self, data: dict[str, Any], venv: Path | None = None) -> None:
        require(
            data["implementation"] == "cpython",
            "Expected CPython, got " + data["implementation"],
        )
        require(
            data["version"] == [3, 7, 9],
            "Interpreter version differs from the explicit request",
        )
        base = self.vx_home / "store" / "python" / PYTHON_VERSION
        require(
            within(Path(data["base_prefix"]), base),
            "Base interpreter escaped the owned VX store",
        )
        require(
            Path(data["prefix"]).resolve() == (venv or base).resolve(),
            "Incorrect Python prefix",
        )
        # On Unix uv normally symlinks venv/bin/python to the managed base.
        # sys.prefix binds the environment; sys.executable must name its entry point.
        executable = Path(data["executable"]).absolute()
        require(
            executable.is_relative_to((venv or base).absolute()),
            "Interpreter executable escaped its environment",
        )

    def contract(self) -> dict[str, Any]:
        self.state["artifact"] = artifact_contract(self.provider)
        return {"provider": str(self.provider), **self.state["artifact"]}

    def fresh_install(self) -> dict[str, Any]:
        require(
            not self.vx_home.exists() and not self.uv_cache.exists(),
            "Acceptance must start with empty caches",
        )
        self.run("vx-version", "--version")
        output = self.run("fresh-python-install", "install", "python@" + PYTHON_VERSION)
        expected = self.state["artifact"]["sha256"]
        require(
            "Verified artifact SHA256" in output and expected in output,
            "VX did not attest the expected digest during fresh acquisition",
        )
        return {
            "empty_vx_home": True,
            "empty_uv_cache": True,
            "verified_sha256": expected,
        }

    def archive_integrity(self) -> dict[str, Any]:
        artifact = self.state["artifact"]
        # DownloadCache::file_path uses the full URL SHA256 under its two-byte shard.
        key = hashlib.sha256(artifact["url"].encode()).hexdigest()
        archive = self.vx_home / "cache" / "downloads" / key[:2] / key
        require(
            archive.is_file(), "Verified origin archive missing from VX download cache"
        )
        actual = sha256(archive)
        require(
            actual == artifact["sha256"],
            "Cached bytes differ from the trusted lifecycle SHA256",
        )
        self.state["archive"] = archive
        return {
            "archive": str(archive),
            "size": archive.stat().st_size,
            "sha256": actual,
        }

    def identity(self) -> dict[str, Any]:
        data = self.python("python-identity", IDENTITY_CODE)
        self.check_identity(data)
        require(
            Path(data["include"]).joinpath("Python.h").is_file(),
            "Python headers required for native builds are absent",
        )
        return data

    def install_uv(self) -> dict[str, Any]:
        output = self.run("uv-version", self.uv, "--version")
        require("uv " + self.uv.split("@", 1)[1] in output, "Wrong uv version")
        return {"version_output": output.strip()}

    def create_venv(self, offline: bool = False) -> dict[str, Any]:
        venv = self.directory / ("venv-offline" if offline else "venv-online")
        self.run(
            "venv-offline" if offline else "venv-online",
            self.uv,
            "venv",
            "--python",
            PYTHON_VERSION,
            "--no-python-downloads",
            str(venv),
            offline=offline,
        )
        data = self.python(
            "venv-identity-offline" if offline else "venv-identity",
            IDENTITY_CODE,
            venv,
            offline,
        )
        self.check_identity(data, venv)
        self.state["venv-offline" if offline else "venv-online"] = venv
        return data

    def packages(self, offline: bool = False) -> dict[str, Any]:
        venv = self.state["venv-offline" if offline else "venv-online"]
        self.run(
            "packages-offline" if offline else "packages-online",
            self.uv,
            "pip",
            "install",
            "--python",
            str(self.venv_python(venv)),
            "--only-binary",
            ":all:",
            *BUILD_REQUIREMENTS,
            offline=offline,
        )
        versions = self.python(
            "backend-versions-offline" if offline else "backend-versions",
            """
import json, pkg_resources
print(json.dumps({name: pkg_resources.get_distribution(name).version
                  for name in ['setuptools', 'wheel', 'build']}))
""",
            venv,
            offline,
        )
        return {"requirements": BUILD_REQUIREMENTS, "installed": versions}

    def build(self, offline: bool = False) -> dict[str, Any]:
        mode = "offline" if offline else "online"
        venv = self.state["venv-" + mode]
        source = self.directory / ("source-" + mode)
        shutil.copytree(
            FIXTURE, source, ignore=shutil.ignore_patterns("__pycache__", "*.pyc")
        )
        destination = self.directory / ("wheels-" + mode)
        self.run(
            "build-" + mode,
            self.uv,
            "build",
            "--python",
            str(self.venv_python(venv)),
            "--no-python-downloads",
            "--no-build-isolation",
            "--wheel",
            "--out-dir",
            str(destination),
            str(source),
            offline=offline,
        )
        wheels = list(destination.glob("*.whl"))
        require(len(wheels) == 1, "Expected exactly one native wheel")
        require("-cp37-" in wheels[0].name, "Wheel was not built for CPython 3.7")
        self.state["wheel-" + mode] = wheels[0]
        return {
            "wheel": str(wheels[0]),
            "sha256": sha256(wheels[0]),
            "no_build_isolation": True,
        }

    def verify_wheel(self, offline: bool = False) -> dict[str, Any]:
        mode = "offline" if offline else "online"
        venv = self.state["venv-" + mode]
        self.run(
            "wheel-install-" + mode,
            self.uv,
            "pip",
            "install",
            "--python",
            str(self.venv_python(venv)),
            str(self.state["wheel-" + mode]),
            offline=offline,
        )
        data = self.python("wheel-execution-" + mode, WHEEL_CODE, venv, offline)
        require(
            data["version"] == [3, 7, 9], "Wheel executed on a different Python version"
        )
        require(
            within(Path(data["module"]), venv),
            "Imported source tree instead of the installed wheel",
        )
        require(
            all(
                version.startswith(PYTHON_VERSION)
                for version in data["native_versions"]
            ),
            "Native extension compile-time/runtime versions differ from CPython 3.7.9",
        )
        self.check_identity(data["build"], venv)
        return data

    def remove_owned_store(self) -> Path:
        store = self.vx_home / "store" / "python" / PYTHON_VERSION
        require(
            self.marker.read_text(encoding="utf-8") == str(self.directory),
            "Missing run ownership marker",
        )
        require(
            within(store, self.directory) and store.resolve() != self.directory,
            "Unsafe removal target",
        )
        require(not store.is_symlink(), "Refusing to remove a symlinked Python store")
        shutil.rmtree(store)
        return store

    def cached_reinstall(self) -> dict[str, Any]:
        store = self.remove_owned_store()
        output = self.run(
            "cached-python-reinstall",
            "install",
            "python@" + PYTHON_VERSION,
            offline=True,
        )
        require(
            "Served from download cache" in output,
            "Reinstall did not attest archive-cache reuse",
        )
        require(
            "Verified artifact SHA256" in output
            and self.state["artifact"]["sha256"] in output,
            "Cached payload was not verified again before extraction",
        )
        actual = sha256(self.state["archive"])
        require(
            actual == self.state["artifact"]["sha256"],
            "Reinstall changed the trusted archive bytes",
        )
        data = self.python("cached-python-identity", IDENTITY_CODE, offline=True)
        self.check_identity(data)
        return {
            "deleted_store": str(store),
            "network_policy": "unreachable_proxy",
            "archive_sha256": actual,
            "identity": data,
        }

    def partial_installation_repair(self) -> dict[str, Any]:
        store = self.vx_home / "store" / "python" / PYTHON_VERSION
        relative = Path("python.exe" if os.name == "nt" else "bin/python3")
        backup = self.directory / "partial-python-executable"
        shutil.copy2(store / relative, backup)
        self.remove_owned_store()
        partial = store / relative
        partial.parent.mkdir(parents=True)
        shutil.copy2(backup, partial)
        output = self.run(
            "partial-install-repair",
            "install",
            "python@" + PYTHON_VERSION,
            offline=True,
        )
        require(
            "Served from download cache" in output,
            "VX reused the partial executable instead of repairing the installation",
        )
        require(
            "Verified artifact SHA256" in output
            and self.state["artifact"]["sha256"] in output,
            "Repair did not verify the fixed lifecycle payload",
        )
        data = self.python("repaired-python-identity", IDENTITY_CODE, offline=True)
        self.check_identity(data)
        require(
            Path(data["include"]).joinpath("Python.h").is_file(),
            "Repair did not restore development headers",
        )
        return {
            "simulated_partial_install": str(partial),
            "identity": data,
            "network_policy": "unreachable_proxy",
        }

    def execute(self) -> int:
        self.gate("artifact_contract", self.contract)
        self.gate("fresh_acquisition", self.fresh_install, ("artifact_contract",))
        self.gate("archive_integrity", self.archive_integrity, ("fresh_acquisition",))
        self.gate("interpreter_identity", self.identity, ("fresh_acquisition",))
        self.gate("uv_installation", self.install_uv)
        self.gate(
            "uv_venv", self.create_venv, ("interpreter_identity", "uv_installation")
        )
        self.gate("compatible_packages", self.packages, ("uv_venv",))
        self.gate("native_pep517_build", self.build, ("compatible_packages",))
        self.gate(
            "installed_wheel_execution", self.verify_wheel, ("native_pep517_build",)
        )
        self.gate(
            "cached_archive_reinstall",
            self.cached_reinstall,
            ("archive_integrity", "interpreter_identity"),
        )
        self.gate(
            "partial_installation_repair",
            self.partial_installation_repair,
            ("cached_archive_reinstall",),
        )
        self.gate(
            "offline_venv",
            lambda: self.create_venv(True),
            ("partial_installation_repair", "uv_installation"),
        )
        self.gate(
            "offline_packages",
            lambda: self.packages(True),
            ("offline_venv", "compatible_packages"),
        )
        self.gate(
            "offline_native_build", lambda: self.build(True), ("offline_packages",)
        )
        self.gate(
            "offline_wheel_execution",
            lambda: self.verify_wheel(True),
            ("offline_native_build",),
        )
        passed = all(gate["status"] == "passed" for gate in self.receipt["gates"])
        self.receipt.update(
            status="passed" if passed else "failed",
            finished_at=datetime.now(timezone.utc).isoformat(),
        )
        self.save()
        print("Receipt: " + str(self.receipt_path), flush=True)
        return 0 if passed else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--vx-binary", type=Path, required=True)
    parser.add_argument(
        "--work-root",
        type=Path,
        default=Path(tempfile.gettempdir()) / "vx-legacy-python",
    )
    parser.add_argument("--receipt", type=Path)
    parser.add_argument(
        "--source-revision", help="Source revision used to build the tested binary"
    )
    parser.add_argument(
        "--provider",
        type=Path,
        default=ROOT / "crates/vx-providers/python/provider.star",
    )
    parser.add_argument("--uv-version", default=UV_VERSION)
    parser.add_argument("--command-timeout", type=int, default=600)
    return Acceptance(parser.parse_args()).execute()


if __name__ == "__main__":
    sys.exit(main())
