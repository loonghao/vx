# provider.star - ruff provider
#
# ruff: An extremely fast Python linter and code formatter, written in Rust
# Releases: https://github.com/astral-sh/ruff/releases
# Asset format: ruff-{triple}.{ext}  (no version in filename)
# Tag format:   {version}  (NO 'v' prefix)
#
# Uses Rust target triples for platform naming.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "archive_layout", "path_fns")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "ruff"
description = "ruff - An extremely fast Python linter and code formatter, written in Rust"
homepage    = "https://docs.astral.sh/ruff"
repository  = "https://github.com/astral-sh/ruff"
license     = "MIT"
ecosystem   = "python"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("ruff",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "ruff \\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — ruff uses plain version tags (no v prefix)
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("astral-sh", "ruff")

# ---------------------------------------------------------------------------
# Platform helpers
# ruff uses Rust target triples
# ---------------------------------------------------------------------------

_RUFF_TRIPLES = {
    "windows/x64":   "x86_64-pc-windows-msvc",
    "windows/arm64": "aarch64-pc-windows-msvc",
    "macos/x64":     "x86_64-apple-darwin",
    "macos/arm64":   "aarch64-apple-darwin",
    "linux/x64":     "x86_64-unknown-linux-gnu",
    "linux/arm64":   "aarch64-unknown-linux-gnu",
}

def _ruff_triple(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _RUFF_TRIPLES.get(key)

# ---------------------------------------------------------------------------
# download_url — ruff-{triple}.{ext}, tag = "{version}" (no v prefix)
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    triple = _ruff_triple(ctx)
    if not triple:
        return None
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    asset = "ruff-{}.{}".format(triple, ext)
    return github_asset_url("astral-sh", "ruff", version, asset)

# ---------------------------------------------------------------------------
# install_layout — archive contains ruff binary at root
# ---------------------------------------------------------------------------

install_layout = archive_layout("ruff")

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

paths            = path_fns("ruff")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []
