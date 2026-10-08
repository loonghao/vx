"""A native PEP 517 build that records the interpreter running its backend."""

import json
from pathlib import Path
import sys
import sysconfig

from setuptools import Extension, setup
from setuptools.command.build_py import build_py


class BuildWithProvenance(build_py):
    def run(self):
        build_py.run(self)
        provenance = {
            "implementation": sys.implementation.name,
            "version": list(sys.version_info[:3]),
            "executable": sys.executable,
            "prefix": sys.prefix,
            "base_prefix": sys.base_prefix,
            "compiler": sysconfig.get_config_var("CC"),
        }
        target = Path(self.build_lib) / "vx_legacy_probe" / "build_provenance.json"
        target.write_text(json.dumps(provenance, indent=2), encoding="utf-8")


setup(
    name="vx-legacy-python-probe",
    version="0.0.1",
    description="VX legacy Python native build acceptance fixture",
    python_requires=">=3.7",
    packages=["vx_legacy_probe"],
    package_data={"vx_legacy_probe": ["build_provenance.json"]},
    ext_modules=[Extension("vx_legacy_probe._native", ["native.c"])],
    cmdclass={"build_py": BuildWithProvenance},
)
