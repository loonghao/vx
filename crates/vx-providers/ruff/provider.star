# provider.star - ruff provider
#
# ruff: An extremely fast Python linter and code formatter, written in Rust
# Releases: https://github.com/astral-sh/ruff/releases
# Asset format: ruff-{triple}.{ext}  (no version in filename)
# Tag format:   {version}  (NO 'v' prefix)
# Linux uses GNU libc (not musl).

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "github_rust_provider")

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
# Provider template - github_rust_provider
#
# Asset:      ruff-{triple}.{ext}  (no version in filename)
# Tag format: {version}  (no v prefix, so tag_prefix = "")
# Linux:      gnu libc
# ---------------------------------------------------------------------------

_p = github_rust_provider(
    "astral-sh", "ruff",
    asset      = "ruff-{triple}.{ext}",
    tag_prefix = "",
    linux_libc = "gnu",
)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]
