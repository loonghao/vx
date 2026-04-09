# provider.star - DuckDB provider
#
# DuckDB is an in-process SQL OLAP database management system.
# The CLI tool is released as a compressed single binary:
#
# Asset naming: duckdb_cli-{os}-{arch}.{ext}
#   - Linux:   duckdb_cli-linux-amd64.zip, duckdb_cli-linux-arm64.zip
#   - macOS:   duckdb_cli-osx-universal.gz  (universal binary)
#   - Windows: duckdb_cli-windows-amd64.zip, duckdb_cli-windows-arm64.zip
#
# Note: macOS uses "osx" (not "darwin") and "universal" architecture.
# Note: Linux uses .zip (not .gz) for amd64/arm64 variants.
# Archive/gz contains: duckdb[.exe] binary

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "duckdb"
description = "DuckDB - In-process SQL OLAP database management system"
homepage    = "https://duckdb.org"
repository  = "https://github.com/duckdb/duckdb"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("duckdb",
        version_cmd     = "{executable} --version",
        version_pattern = "v\\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — from duckdb/duckdb releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("duckdb", "duckdb")

# ---------------------------------------------------------------------------
# Platform helpers
# DuckDB uses "osx" for macOS and "universal" for macOS arch
# ---------------------------------------------------------------------------

def _duckdb_asset(ctx):
    """Return (asset_name, is_gz) for the CLI on the current platform."""
    if ctx.platform.os == "macos":
        # macOS provides a universal binary in .gz format
        return "duckdb_cli-osx-universal.gz", True
    elif ctx.platform.os == "linux":
        arch_map = {"x64": "amd64", "arm64": "arm64"}
        arch_str = arch_map.get(ctx.platform.arch, "amd64")
        return "duckdb_cli-linux-{}.zip".format(arch_str), False
    elif ctx.platform.os == "windows":
        arch_map = {"x64": "amd64", "arm64": "arm64"}
        arch_str = arch_map.get(ctx.platform.arch, "amd64")
        return "duckdb_cli-windows-{}.zip".format(arch_str), False
    return None, None

# ---------------------------------------------------------------------------
# download_url — GitHub releases
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    asset, _is_gz = _duckdb_asset(ctx)
    if not asset:
        return None
    return "https://github.com/duckdb/duckdb/releases/download/v{}/{}".format(version, asset)

# ---------------------------------------------------------------------------
# install_layout — compressed binary (zip or gz)
# After extraction the binary is named "duckdb" (or "duckdb.exe" on Windows)
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "duckdb.exe" if ctx.platform.os == "windows" else "duckdb"
    return {
        "type":             "archive",
        "executable_paths": [exe, "duckdb"],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/duckdb"


def get_execute_path(ctx, _version):
    exe = "duckdb.exe" if ctx.platform.os == "windows" else "duckdb"
    return ctx.install_dir + "/" + exe


def post_install(_ctx, _version):
    return None


def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]


def deps(_ctx, _version):
    return []
