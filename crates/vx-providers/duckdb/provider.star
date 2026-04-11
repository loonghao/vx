# provider.star - DuckDB provider
#
# DuckDB is an in-process SQL OLAP database management system.
# The CLI tool is released as a compressed single binary.
#
# Asset naming: duckdb_cli-{os}-{arch}.{ext}
#   - Linux:   duckdb_cli-linux-amd64.zip, duckdb_cli-linux-arm64.zip
#   - macOS:   duckdb_cli-osx-amd64.zip, duckdb_cli-osx-arm64.zip
#              (also universal: duckdb_cli-osx-universal.zip)
#   - Windows: duckdb_cli-windows-amd64.zip, duckdb_cli-windows-arm64.zip
#
# NOTE: macOS also ships a .gz single-binary, but we use .zip for
#       consistent archive extraction across all platforms.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "path_fns")
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
# fetch_versions - from duckdb/duckdb releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("duckdb", "duckdb")

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

def _duckdb_asset(ctx):
    """Return the asset filename for the DuckDB CLI on the current platform."""
    if ctx.platform.os == "macos":
        # Use arch-specific zip assets (available since v1.1.x)
        arch_map = {"x64": "amd64", "arm64": "arm64"}
        arch_str = arch_map.get(ctx.platform.arch, "amd64")
        return "duckdb_cli-osx-{}.zip".format(arch_str)
    elif ctx.platform.os == "linux":
        arch_map = {"x64": "amd64", "arm64": "arm64"}
        arch_str = arch_map.get(ctx.platform.arch, "amd64")
        return "duckdb_cli-linux-{}.zip".format(arch_str)
    elif ctx.platform.os == "windows":
        arch_map = {"x64": "amd64", "arm64": "arm64"}
        arch_str = arch_map.get(ctx.platform.arch, "amd64")
        return "duckdb_cli-windows-{}.zip".format(arch_str)
    return None

# ---------------------------------------------------------------------------
# download_url - GitHub releases
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    asset = _duckdb_asset(ctx)
    if not asset:
        return None
    return "https://github.com/duckdb/duckdb/releases/download/v{}/{}".format(version, asset)

# ---------------------------------------------------------------------------
# install_layout - compressed binary (zip or gz)
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "duckdb.exe" if ctx.platform.os == "windows" else "duckdb"
    return {
        "__type":           "archive",
        "executable_paths": [exe, "duckdb"],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

paths            = path_fns("duckdb")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
