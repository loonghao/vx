"""Installed-wheel probes for legacy Python acceptance."""

import json
from pathlib import Path

from ._native import versions as versions


def build_provenance():
    return json.loads(
        (Path(__file__).parent / "build_provenance.json").read_text(encoding="utf-8")
    )
